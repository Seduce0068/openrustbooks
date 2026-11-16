//! EPUB の本文読み出し。
//!
//! [`ArchiveReader`] の上に、`META-INF/container.xml` → OPF（メタデータ・
//! manifest・spine）→ 章本文とリソースの読み出しまでを載せる。ファイル I/O に
//! 依存せず、ネイティブと `wasm32-unknown-unknown` の両方でビルドできる。
//!
//! `docs/requirements.md` の REQ-002（電子書籍の閲覧）のうち、本文・章・目次
//! （EPUB3 の nav と EPUB2 の NCX）の読み出しに加え、表示用の XHTML
//! （[`EpubBook::chapter_html`]）とリソースの読み出し
//! （[`EpubBook::chapter_resource`]）、表紙画像（[`EpubBook::cover_image`]）と
//! 章の表示名（[`EpubBook::chapter_title`]）、landmarks と page-list
//! （[`EpubBook::landmarks`]、[`EpubBook::page_list`]）、本文内の相対リンクの
//! 解決（[`EpubBook::resolve_chapter_link`]）の取得を提供する。
//! 表示位置のスクロール、画像のデコード・表示、リソース URL の書き換え、
//! サニタイズ、`Document` 型との統合は範囲外。読み取り専用であり、EPUB の
//! 作成・書き換え・DRM 解除は提供しない。
//!
//! 失敗はすべて [`CoreResult`] で返し、panic しない。不正な入力は
//! 黙って読み飛ばさず、`CoreError::Archive` / `CoreError::Validation` として
//! 報告する（fail-closed）。

use std::collections::HashMap;

use quick_xml::events::{BytesStart, Event};
use quick_xml::{Reader, XmlVersion};

use crate::archive::{ArchiveLimits, ArchiveReader};
use crate::{CoreError, CoreResult};

/// EPUB コンテナ文書のパス。
const CONTAINER_PATH: &str = "META-INF/container.xml";
/// コンテナ文書で OPF を指す要素のローカル名。
const ROOTFILE_ELEMENT: &str = "rootfile";
/// OPF のパスを持つ属性のローカル名。
const FULL_PATH_ATTRIBUTE: &str = "full-path";
/// `<meta property="dcterms:modified">` の `property` 値。
const MODIFIED_PROPERTY: &str = "dcterms:modified";
/// 本文抽出で内容を除外する要素のローカル名。
const SKIPPED_ELEMENTS: [&str; 3] = ["head", "script", "style"];
/// EPUB3 nav を目次として選ぶ `properties` のトークン。
const NAV_PROPERTY: &str = "nav";
/// EPUB3 nav を目次として選ぶ `epub:type` のトークン。
const TOC_TYPE: &str = "toc";
/// EPUB3 nav の landmarks 種別。
const LANDMARKS_TYPE: &str = "landmarks";
/// EPUB3 nav の page-list 種別。
const PAGE_LIST_TYPE: &str = "page-list";
/// EPUB2 NCX の media-type。
const NCX_MEDIA_TYPE: &str = "application/x-dtbncx+xml";
/// 表紙画像を指す `properties` のトークン（EPUB3）。
const COVER_IMAGE_PROPERTY: &str = "cover-image";
/// 表紙画像の `id` を持つ `<meta>` の `name` 属性値（EPUB2）。
const COVER_META_NAME: &str = "cover";
/// XHTML の文書タイトルを持つ要素のローカル名。
const TITLE_ELEMENT: &str = "title";
/// 本文の開始を示す要素のローカル名。
const BODY_ELEMENT: &str = "body";
/// nav の要素のローカル名。
const NAV_ELEMENT: &str = "nav";
/// 目次エントリ数の上限。
const MAX_TOC_ENTRIES: usize = 10_000;

/// 本文抽出で改行を挿入する要素のローカル名。
const BLOCK_ELEMENTS: [&str; 16] = [
    "p",
    "div",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "li",
    "blockquote",
    "section",
    "article",
    "pre",
    "tr",
    "td",
    "th",
];

/// EPUB の書誌メタデータ。
///
/// 値が無い項目は `None`、複数あり得る項目は文書順の [`Vec`] で表す。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EpubMetadata {
    /// `dc:title`（最初の空でない値）。
    pub title: Option<String>,
    /// `dc:creator`（文書順）。
    pub creators: Vec<String>,
    /// `dc:language`（最初の空でない値）。
    pub language: Option<String>,
    /// `dc:identifier`（`package` の `unique-identifier` と一致する `id` を優先）。
    pub identifier: Option<String>,
    /// `dc:publisher`（最初の空でない値）。
    pub publisher: Option<String>,
    /// `dc:date`（最初の空でない値）。
    pub date: Option<String>,
    /// `<meta property="dcterms:modified">` の内容。
    pub modified: Option<String>,
    /// `package` 要素の `version` 属性。
    pub version: Option<String>,
}

/// spine の順序に対応する 1 章。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpubChapter {
    /// manifest の `id`。
    pub id: String,
    /// OPF の位置を基準に解決済みの、アーカイブ内パス。
    pub href: String,
    /// manifest の `media-type`（そのままの値）。
    pub media_type: String,
    /// `itemref` の `linear` 属性。既定は `true`。
    pub linear: bool,
}

/// 本文内の相対リンクの解決結果。
///
/// 同じ本の spine にある章へのリンクだけを表す。外部リンクや本の外を指す
/// リンクは [`EpubBook::resolve_chapter_link`] が `None` を返す。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChapterLink {
    /// 移動先の spine の index。
    pub chapter_index: usize,
    /// 移動先のフラグメント（`#` の後ろ）。無ければ `None`。
    pub fragment: Option<String>,
}

/// 表紙画像。
///
/// 画像のバイト列まで含む。デコード・縮小・URL の書き換えは行わない。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpubCoverImage {
    /// manifest の `id`。
    pub id: String,
    /// OPF の位置を基準に解決済みの、アーカイブ内パス。
    pub href: String,
    /// manifest の `media-type`（そのままの値。未宣言なら空文字列）。
    pub media_type: String,
    /// 画像のバイト列。
    pub bytes: Vec<u8>,
}

/// 目次の 1 項目。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpubTocEntry {
    /// 表示名（実体参照を戻し、空白を 1 つにまとめたもの）。
    pub title: String,
    /// 本文と同じ規則で解決済みのアーカイブ内パス（フラグメントは含まない）。
    pub href: String,
    /// 生の `href` の最初の `#` の後ろ（`#` を除き、前後の空白を除いたもの）。
    /// 元の綴りのまま保持し、パーセント復号しない。`#` が無い・空なら `None`。
    pub fragment: Option<String>,
    /// 入れ子の深さ。最上位を 0 とする。
    pub depth: usize,
}

/// ZIP バイト列・解析済み OPF・章一覧を所有する EPUB ビュー。
///
/// 入力バイト列を所有するため `open` の戻り値以降は借用を持たず、内部可変性を
/// 使わないため `Send + Sync` を満たす。
pub struct EpubBook {
    /// 原本のアーカイブ。
    archive: ArchiveReader,
    /// OPF から読んだ書誌メタデータ。
    metadata: EpubMetadata,
    /// spine の順に並べた章。
    chapters: Vec<EpubChapter>,
    /// 目次の情報源を選ぶために保持する manifest。
    manifest: Vec<ManifestItem>,
    /// 目次の `href` を解決する基準のディレクトリ（OPF の位置）。
    base_dir: String,
    /// `<meta name="cover">` が指す表紙画像の `id`（EPUB2）。
    cover_id: Option<String>,
}

impl EpubBook {
    /// 既定の [`ArchiveLimits`] で EPUB を開く。
    pub fn open(bytes: &[u8]) -> CoreResult<Self> {
        Self::with_limits(bytes, ArchiveLimits::default())
    }

    /// リミットを指定して EPUB を開く。
    ///
    /// `mimetype` を持たないアーカイブは [`CoreError::UnsupportedFormat`]、
    /// コンテナ文書や OPF の読み出し・解析の失敗は [`CoreError::Archive`]、
    /// パス逸脱やリミット超過は [`CoreError::Validation`] を返す。
    pub fn with_limits(bytes: &[u8], limits: ArchiveLimits) -> CoreResult<Self> {
        Self::from_archive(ArchiveReader::with_limits(bytes, limits)?)
    }

    /// 所有権を持つバイト列から EPUB を開く。
    ///
    /// 呼び出し側が既にバイト列を所有している場合の二重コピーを避ける。
    /// 既定のリミットを使い、バリデーションとエラーは `open` と同一。
    pub fn open_owned(bytes: Vec<u8>) -> CoreResult<Self> {
        Self::from_archive(ArchiveReader::from_owned(bytes, ArchiveLimits::default())?)
    }

    /// 開いたアーカイブからメタデータと章を読み出す共通処理。
    fn from_archive(archive: ArchiveReader) -> CoreResult<Self> {
        if !archive.is_epub() {
            return Err(CoreError::UnsupportedFormat(
                "EPUB ではありません (mimetype が application/epub+zip ではない)".to_string(),
            ));
        }

        let container = read_text_entry(&archive, CONTAINER_PATH)?;
        let opf_path = parse_container(&container)?;
        let opf = read_text_entry(&archive, &opf_path)?;
        let parsed = parse_opf(&opf, &opf_path)?;

        Ok(Self {
            archive,
            metadata: parsed.metadata,
            chapters: parsed.chapters,
            manifest: parsed.manifest,
            base_dir: parsed.base_dir,
            cover_id: parsed.cover_id,
        })
    }

    /// 書誌メタデータを返す。
    pub fn metadata(&self) -> &EpubMetadata {
        &self.metadata
    }

    /// spine の順に並んだ章を返す。
    pub fn chapters(&self) -> &[EpubChapter] {
        &self.chapters
    }

    /// 目次を返す。情報源が無い場合は空。
    ///
    /// manifest の `properties` に `nav` を含む item の XHTML を EPUB3 nav として
    /// 読み、無ければ media-type が `application/x-dtbncx+xml` の item を EPUB2
    /// NCX として読む。情報源のエントリがアーカイブに無い場合も空を返す。
    ///
    /// 目次の XML の構文エラーは [`CoreError::Archive`]、`href` の逸脱と
    /// エントリ数の上限超過は [`CoreError::Validation`]。
    ///
    /// [`EpubTocEntry::href`] はフラグメントを落としたパス、`fragment` は生の
    /// `href` の `#` の後ろを保持する。パス部が空の項目（`href` が無い・
    /// フラグメントのみ・空白のみ）はエラーにせず読み飛ばす。
    pub fn toc(&self) -> CoreResult<Vec<EpubTocEntry>> {
        let nav = self
            .manifest
            .iter()
            .find(|item| has_property(&item.properties, NAV_PROPERTY));
        if let Some(item) = nav {
            let Some(xml) = self.read_toc_source(&item.href)? else {
                return Ok(Vec::new());
            };
            return parse_nav(&xml, &self.base_dir, &item.href);
        }

        let ncx = self
            .manifest
            .iter()
            .find(|item| item.media_type.trim() == NCX_MEDIA_TYPE);
        if let Some(item) = ncx {
            let Some(xml) = self.read_toc_source(&item.href)? else {
                return Ok(Vec::new());
            };
            return parse_ncx(&xml, &self.base_dir, &item.href);
        }

        Ok(Vec::new())
    }

    /// EPUB3 nav の landmarks を返す。情報源が無い場合は空。
    ///
    /// エントリの `href`・`fragment`・`depth` は [`EpubBook::toc`] と同じ規則で
    /// 解決する。目次の nav が無い場合も landmarks 用 nav を検索する。
    pub fn landmarks(&self) -> CoreResult<Vec<EpubTocEntry>> {
        self.navigation_list(LANDMARKS_TYPE)
    }

    /// EPUB3 nav の page-list を返す。情報源が無い場合は空。
    ///
    /// エントリの `href`・`fragment`・`depth` は [`EpubBook::toc`] と同じ規則で
    /// 解決する。目次の nav が無い場合も page-list 用 nav を検索する。
    pub fn page_list(&self) -> CoreResult<Vec<EpubTocEntry>> {
        self.navigation_list(PAGE_LIST_TYPE)
    }

    /// 章の内容をバイト列で返す。
    ///
    /// 範囲外の `index` は [`CoreError::DocumentNotFound`]。`href` のエントリが
    /// 無い場合は、`%XX` を復号した名前でも試す。
    pub fn chapter_bytes(&self, index: usize) -> CoreResult<Vec<u8>> {
        let href = self.chapter(index)?.href.clone();
        match self.archive.read_entry(&href) {
            Ok(data) => Ok(data),
            Err(first) => self.read_decoded_entry(&href).ok_or(first),
        }
    }

    /// 章の本文をテキストとして返す。
    ///
    /// 先頭の BOM を除去して UTF-8 として復号し、XML として解析する。不正な
    /// UTF-8 と XML の構文エラーは [`CoreError::Archive`]。`head`・`script`・
    /// `style` の内側は含めず、ブロック要素と `br` で改行する。
    pub fn chapter_text(&self, index: usize) -> CoreResult<String> {
        let (href, body) = self.chapter_source(index)?;
        extract_text(&body, &href)
    }

    /// 章の XHTML を先頭の BOM を除いた UTF-8 文字列で返す。
    ///
    /// 内容はそのまま返し、XML 解析・タグ除去・整形はしない。範囲外の
    /// `index` は [`CoreError::DocumentNotFound`]、読み出しと復号の失敗は
    /// [`CoreError::Archive`]。
    pub fn chapter_html(&self, index: usize) -> CoreResult<String> {
        let (_, html) = self.chapter_source(index)?;
        Ok(html)
    }

    /// 章から見た相対参照 `href` のリソースをバイト列で返す。
    ///
    /// 基準ディレクトリは章の `href` の所属ディレクトリ。`#` 以降の
    /// フラグメントは落とす。範囲外の `index` と存在しないパスは
    /// [`CoreError::DocumentNotFound`]、空のパスとスキーム付き URL・
    /// 先頭 `/` は [`CoreError::Archive`]、ルートを越える解決は
    /// [`CoreError::Validation`]。
    pub fn chapter_resource(&self, index: usize, href: &str) -> CoreResult<Vec<u8>> {
        let chapter_href = self.chapter(index)?.href.clone();
        let path = resolve_reference(path_dir(&chapter_href), href, "章リソースの href")?;
        match self.archive.read_entry(&path) {
            Ok(data) => Ok(data),
            Err(first) => self.read_decoded_entry(&path).ok_or(first),
        }
    }

    /// 章 `index` の本文にある相対リンク `href` を、同じ本の章へ解決する。
    ///
    /// `#fragment` だけのリンクは同じ章を返す。スキーム付き URL・先頭 `/`・
    /// 空のリンク・spine に無いパス・ルートを越える参照は `None` を返し、
    /// 外部や本の外へは移動しない（fail-closed）。範囲外の `index` は
    /// [`CoreError::DocumentNotFound`]。`%XX` は復号して比較する。
    pub fn resolve_chapter_link(
        &self,
        index: usize,
        href: &str,
    ) -> CoreResult<Option<ChapterLink>> {
        let chapter_href = self.chapter(index)?.href.clone();
        let (path, fragment) = split_fragment(href);
        if path.is_empty() {
            return Ok(fragment.map(|fragment| ChapterLink {
                chapter_index: index,
                fragment: Some(fragment),
            }));
        }
        if path.starts_with('/') || has_scheme(path) {
            return Ok(None);
        }
        let Ok(target) = resolve_reference(path_dir(&chapter_href), path, "章リンクの href")
        else {
            return Ok(None);
        };
        let Some(chapter_index) = self.chapter_index_for_path(&target) else {
            return Ok(None);
        };
        Ok(Some(ChapterLink {
            chapter_index,
            fragment,
        }))
    }

    /// アーカイブ内パスに対応する章の index を返す。
    ///
    /// spine の `href` と比較するとき、`%XX` を復号した形でも一致を調べる。
    fn chapter_index_for_path(&self, target: &str) -> Option<usize> {
        let target = normalized_chapter_path(target);
        self.chapters
            .iter()
            .position(|chapter| normalized_chapter_path(&chapter.href) == target)
    }

    /// 章の表示名を返す。情報源が無い場合は `None`。
    ///
    /// 目次（[`EpubBook::toc`]）で `href` が一致する最初のエントリの表示名を
    /// 優先し、一致するエントリが無ければ章の XHTML の `<head>` の `<title>` を
    /// 使う。実体参照は戻し、前後の空白と連続する空白はまとめる。
    ///
    /// 範囲外の `index` は [`CoreError::DocumentNotFound`]。目次または章の
    /// 読み出し・解析の失敗は [`CoreError::Archive`] / [`CoreError::Validation`]
    /// としてそのまま報告し、`<title>` へは落とさない（目次が壊れている本で
    /// 別の名前を返さないため）。呼び出しごとに目次を解析し直し、必要なら章を
    /// 読み直す（[`EpubBook::toc`] と同じ方針で、内部にキャッシュしない）。
    pub fn chapter_title(&self, index: usize) -> CoreResult<Option<String>> {
        let href = self.chapter(index)?.href.clone();
        if let Some(title) = self.toc_title(&href)? {
            return Ok(Some(title));
        }
        let html = self.chapter_html(index)?;
        extract_title(&html, &href)
    }

    /// 表紙画像を返す。表紙の宣言が無い場合は `None`。
    ///
    /// manifest の `properties` に `cover-image` を含む item（EPUB3）を優先し、
    /// 無ければ `<meta name="cover">` の `content` が指す `id` の item（EPUB2）
    /// を使う。EPUB3 の宣言がある場合は EPUB2 へは落ちない。`media-type` が
    /// `image/` 以外と明示されている item と、宣言されたエントリがアーカイブに
    /// 無い場合は表紙なしとして `None` を返す（表紙は補助情報であり、無いことで
    /// 本文の読み出しを妨げない）。それ以外の読み出しの失敗は
    /// [`CoreError::Archive`] / [`CoreError::Validation`] として報告する。
    pub fn cover_image(&self) -> CoreResult<Option<EpubCoverImage>> {
        let item = self
            .manifest
            .iter()
            .find(|item| has_property(&item.properties, COVER_IMAGE_PROPERTY))
            .or_else(|| {
                let id = self.cover_id.as_deref()?;
                self.manifest.iter().find(|item| item.id == id)
            });
        let Some(item) = item else {
            return Ok(None);
        };
        if !is_image_media_type(&item.media_type) {
            return Ok(None);
        }
        let bytes = match self.archive.read_entry(&item.href) {
            Ok(data) => data,
            // エントリが無い場合は `%XX` を復号した名前でも試す
            // （`chapter_bytes` と同じフォールバック）。それでも無ければ表紙なし。
            Err(CoreError::DocumentNotFound(_)) => match self.read_decoded_entry(&item.href) {
                Some(data) => data,
                None => return Ok(None),
            },
            Err(err) => return Err(err),
        };
        Ok(Some(EpubCoverImage {
            id: item.id.clone(),
            href: item.href.clone(),
            media_type: item.media_type.clone(),
            bytes,
        }))
    }

    /// 範囲を検証して章を取り出す。
    fn chapter(&self, index: usize) -> CoreResult<&EpubChapter> {
        self.chapters
            .get(index)
            .ok_or_else(|| CoreError::DocumentNotFound(format!("章の index が範囲外です: {index}")))
    }

    /// 目次で `href` が一致する最初の表示名（空でない）を返す。
    fn toc_title(&self, href: &str) -> CoreResult<Option<String>> {
        Ok(self
            .toc()?
            .into_iter()
            .find(|entry| entry.href == href && !entry.title.is_empty())
            .map(|entry| entry.title))
    }

    /// 章の `href` と、復号して先頭の BOM を 1 個だけ除いた本文を返す。
    ///
    /// 範囲検証と `%XX` の復号フォールバックは [`EpubBook::chapter_bytes`] に
    /// 委ねる。不正な UTF-8 は [`CoreError::Archive`]。
    fn chapter_source(&self, index: usize) -> CoreResult<(String, String)> {
        let href = self.chapter(index)?.href.clone();
        let data = self.chapter_bytes(index)?;
        let decoded = String::from_utf8(data).map_err(|err| {
            CoreError::archive(format!("章を UTF-8 として復号できません: {href} ({err})"))
        })?;
        // 先頭の BOM は 1 個だけ除き、2 個目以降は本文として残す。
        let body = match decoded.strip_prefix('\u{FEFF}') {
            Some(stripped) => stripped.to_string(),
            None => decoded,
        };
        Ok((href, body))
    }

    /// `href` の `%XX` を復号した名前で読めたらその内容を返す。
    fn read_decoded_entry(&self, href: &str) -> Option<Vec<u8>> {
        let decoded = percent_decode(href)?;
        if decoded == href {
            return None;
        }
        self.archive.read_entry(&decoded).ok()
    }

    /// 目次の情報源のテキストを読む。エントリが無い場合は `None`。
    ///
    /// [`read_text_entry`] はエントリの欠落も [`CoreError::Archive`] にするため、
    /// 目次の情報源では欠落を空の目次として扱えるように `read_entry_text` を
    /// 直接使う。
    fn read_toc_source(&self, href: &str) -> CoreResult<Option<String>> {
        match self.archive.read_entry_text(href) {
            Ok(text) => Ok(Some(text)),
            Err(CoreError::DocumentNotFound(_)) => Ok(None),
            Err(err) => Err(err),
        }
    }

    /// EPUB3 nav 文書から指定された種類の navigation list を読む。
    fn navigation_list(&self, navigation_type: &str) -> CoreResult<Vec<EpubTocEntry>> {
        let Some(item) = self
            .manifest
            .iter()
            .find(|item| has_property(&item.properties, NAV_PROPERTY))
        else {
            return Ok(Vec::new());
        };
        let Some(xml) = self.read_toc_source(&item.href)? else {
            return Ok(Vec::new());
        };
        let Some(target) = select_nav_type(&xml, &item.href, navigation_type)? else {
            return Ok(Vec::new());
        };
        collect_nav_entries(&xml, target, &self.base_dir, &item.href)
    }
}

/// OPF の解析結果。
struct ParsedOpf {
    /// 書誌メタデータ。
    metadata: EpubMetadata,
    /// spine の順の章。
    chapters: Vec<EpubChapter>,
    /// manifest の `item`（文書順）。
    manifest: Vec<ManifestItem>,
    /// OPF の位置（相対 `href` の基準）。
    base_dir: String,
    /// `<meta name="cover">` が指す表紙画像の `id`。
    cover_id: Option<String>,
}

/// manifest の `item`。
struct ManifestItem {
    /// manifest の `id`。
    id: String,
    /// OPF の位置を基準に解決済みのアーカイブ内パス。
    href: String,
    /// `media-type` の値（無い場合は空文字列）。
    media_type: String,
    /// `properties` の値（空白区切りのトークン列。無い場合は空文字列）。
    properties: String,
}

/// spine の `itemref`。
struct SpineRef {
    /// manifest の `id` への参照。
    idref: String,
    /// 本文の読み上げ順に含めるかどうか。
    linear: bool,
}

/// メタデータ収集で対象にする項目。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MetadataField {
    /// `dc:title`。
    Title,
    /// `dc:creator`。
    Creator,
    /// `dc:language`。
    Language,
    /// `dc:identifier`。
    Identifier,
    /// `dc:publisher`。
    Publisher,
    /// `dc:date`。
    Date,
    /// `<meta property="dcterms:modified">`。
    Modified,
}

impl MetadataField {
    /// 要素のローカル名から対応する項目を返す。
    fn from_element(name: &str) -> Option<Self> {
        match name {
            "title" => Some(Self::Title),
            "creator" => Some(Self::Creator),
            "language" => Some(Self::Language),
            "identifier" => Some(Self::Identifier),
            "publisher" => Some(Self::Publisher),
            "date" => Some(Self::Date),
            _ => None,
        }
    }
}

/// 収集中のメタデータ要素。
struct Capture {
    /// 対応する要素のローカル名。終了タグとの対応付けに使う。
    element: String,
    /// 収集する項目。
    field: MetadataField,
    /// `dc:identifier` の `id` 属性。
    id: Option<String>,
    /// 収集したテキスト。
    text: String,
}

/// 収集し終えたメタデータ要素。
struct CollectedField {
    /// 収集した項目。
    field: MetadataField,
    /// `dc:identifier` の `id` 属性。
    id: Option<String>,
    /// 収集したテキスト（正規化前）。
    text: String,
}

/// アーカイブから必須のテキストエントリを読む。
///
/// 読み出しの失敗は理由を問わず [`CoreError::Archive`] にする。
fn read_text_entry(archive: &ArchiveReader, path: &str) -> CoreResult<String> {
    archive
        .read_entry_text(path)
        .map_err(|err| CoreError::archive(format!("{path} を読み出せません: {err}")))
}

/// コンテナ文書から最初の `rootfile` の `full-path` を取り出す。
fn parse_container(xml: &str) -> CoreResult<String> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);

    let mut depth: usize = 0;
    loop {
        match reader.read_event() {
            Ok(Event::Start(element)) => {
                depth += 1;
                if let Some(path) = rootfile_path(&element)? {
                    return normalize_container_path(&path);
                }
            }
            Ok(Event::Empty(element)) => {
                if let Some(path) = rootfile_path(&element)? {
                    return normalize_container_path(&path);
                }
            }
            Ok(Event::End(_)) => depth = depth.saturating_sub(1),
            Ok(Event::Eof) => {
                if depth != 0 {
                    return Err(CoreError::archive(format!(
                        "{CONTAINER_PATH} の要素が閉じられていません"
                    )));
                }
                break;
            }
            Ok(_) => {}
            Err(err) => {
                return Err(CoreError::archive(format!(
                    "{CONTAINER_PATH} を解析できません: {err}"
                )));
            }
        }
    }

    Err(CoreError::archive(format!(
        "{CONTAINER_PATH} に rootfile 要素がありません"
    )))
}

/// `rootfile` 要素から `full-path` 属性を取り出す。
fn rootfile_path(element: &BytesStart<'_>) -> CoreResult<Option<String>> {
    if element.local_name().into_inner() != ROOTFILE_ELEMENT {
        return Ok(None);
    }
    for attr in element.attributes().flatten() {
        if attr.key.local_name().into_inner() == FULL_PATH_ATTRIBUTE {
            return Ok(Some(unescape_attribute(&attr, FULL_PATH_ATTRIBUTE)?));
        }
    }
    Ok(None)
}

/// `full-path` をアーカイブ内パスとして正規化する。
///
/// 先頭の `/` と `.` コンポーネントは落とす。`..` と空のパスは不正な
/// コンテナ文書として [`CoreError::Archive`] を返す。
fn normalize_container_path(raw: &str) -> CoreResult<String> {
    let trimmed = raw.trim();
    if trimmed.contains('\0') {
        return Err(CoreError::archive(
            "container.xml の full-path に NUL 文字が含まれています",
        ));
    }

    let mut components: Vec<&str> = Vec::new();
    for component in trimmed.split('/') {
        match component {
            "" | "." => {}
            ".." => {
                return Err(CoreError::archive(format!(
                    "container.xml の full-path が親ディレクトリを参照しています: {raw:?}"
                )));
            }
            other => components.push(other),
        }
    }

    if components.is_empty() {
        return Err(CoreError::archive("container.xml の full-path が空です"));
    }
    Ok(components.join("/"))
}

/// パスから、相対参照の基準となる所属ディレクトリ（最後の `/` まで）を返す。
///
/// `/` を含まない場合はルート直下とみなして空文字列を返す。
fn path_dir(path: &str) -> &str {
    match path.rfind('/') {
        Some(index) => &path[..index],
        None => "",
    }
}

/// 生の `href` / `src` を「最初の `#` の前」と「後ろ」に分ける。
///
/// パス部は前後の空白を除いたもの、フラグメントは `#` を除いて前後の空白を
/// 除いたもの（空なら `None`）を返す。2 つ目以降の `#` は区切りにせず、
/// `%XX` はそのまま残す。属性の実体参照は復号済みの文字列を渡す。
fn split_fragment(raw: &str) -> (&str, Option<String>) {
    let Some((path, fragment)) = raw.split_once('#') else {
        return (raw.trim(), None);
    };
    let fragment = fragment.trim();
    let fragment = if fragment.is_empty() {
        None
    } else {
        Some(fragment.to_string())
    };
    (path.trim(), fragment)
}

/// `href` を OPF のディレクトリ基準でアーカイブ内パスへ解決する。
///
/// フラグメント (`#...`) を落とし、`.` と `..` を字句的に解決する。スキーム付き
/// URL と先頭 `/` は不正な OPF として [`CoreError::Archive`]、ルートを越える
/// 解決は [`CoreError::Validation`] を返す。
fn resolve_href(opf_dir: &str, href: &str) -> CoreResult<String> {
    resolve_reference(opf_dir, href, "manifest の href")
}

/// 相対参照 `href` を `base_dir` 基準でアーカイブ内パスへ解決する。
///
/// 最初の `#` 以降（フラグメント）を落として前後の空白を除き、`.` と `..` を
/// 字句的に解決する。空のパス・スキーム付き URL・先頭 `/` は対象の文書が
/// 不正であるとして [`CoreError::Archive`]、ルートを越える解決は
/// [`CoreError::Validation`] を返す。`subject` はエラーメッセージの主語
/// （例: `manifest の href`）に使う。
fn resolve_reference(base_dir: &str, href: &str, subject: &str) -> CoreResult<String> {
    let target = href.split('#').next().unwrap_or_default().trim();
    if target.is_empty() {
        return Err(CoreError::archive(format!("{subject} が空です")));
    }
    if target.starts_with('/') {
        return Err(CoreError::archive(format!(
            "{subject} が絶対パスです: {href:?}"
        )));
    }
    if has_scheme(target) {
        return Err(CoreError::archive(format!(
            "{subject} がスキーム付き URL です: {href:?}"
        )));
    }

    let mut components: Vec<&str> = Vec::new();
    for component in base_dir.split('/').filter(|part| !part.is_empty()) {
        components.push(component);
    }
    for component in target.split('/') {
        match component {
            "" | "." => {}
            ".." => {
                if components.pop().is_none() {
                    return Err(CoreError::validation(format!(
                        "{subject} がルートを越えて解決されます: {href:?}"
                    )));
                }
            }
            other => components.push(other),
        }
    }

    if components.is_empty() {
        return Err(CoreError::archive(format!(
            "{subject} が空のパスに解決されます: {href:?}"
        )));
    }
    Ok(components.join("/"))
}

/// `http://` や `mailto:` のようなスキーム付き URL かどうか。
fn has_scheme(value: &str) -> bool {
    let Some((prefix, _)) = value.split_once(':') else {
        return false;
    };
    let mut chars = prefix.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_alphabetic() {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
}

/// 要素の属性値を実体参照ごと復号して取り出す。
fn attribute_value(element: &BytesStart<'_>, name: &str) -> CoreResult<Option<String>> {
    for attr in element.attributes().flatten() {
        if attr.key.local_name().into_inner() == name {
            return Ok(Some(unescape_attribute(&attr, name)?));
        }
    }
    Ok(None)
}

/// 属性値を実体参照ごと復号する。
///
/// 0.42 では `unescape_value()` は非推奨で、`normalized_value()` が同じ
/// 復号（定義済み実体の解決と改行の正規化）を行う。
fn unescape_attribute(
    attr: &quick_xml::events::attributes::Attribute<'_>,
    name: &str,
) -> CoreResult<String> {
    attr.normalized_value(XmlVersion::Implicit1_0)
        .map(|value| value.into_owned())
        .map_err(|err| CoreError::archive(format!("属性 {name} を復号できません: {err}")))
}

/// OPF を解析してメタデータと章一覧を作る。
fn parse_opf(xml: &str, opf_path: &str) -> CoreResult<ParsedOpf> {
    let base_dir = path_dir(opf_path);

    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);

    let mut version: Option<String> = None;
    let mut unique_identifier: Option<String> = None;
    let mut captures: Vec<Capture> = Vec::new();
    let mut collected: Vec<CollectedField> = Vec::new();
    let mut manifest: Vec<ManifestItem> = Vec::new();
    let mut manifest_index: HashMap<String, usize> = HashMap::new();
    let mut spine: Vec<SpineRef> = Vec::new();
    let mut cover_id: Option<String> = None;
    let mut depth: usize = 0;

    loop {
        match reader.read_event() {
            Ok(Event::Start(element)) => {
                depth += 1;
                let name = element.local_name().into_inner();
                match name {
                    "package" => {
                        read_package_attributes(&element, &mut version, &mut unique_identifier)?;
                    }
                    "item" => {
                        push_manifest_item(&element, base_dir, &mut manifest, &mut manifest_index)?;
                    }
                    "itemref" => push_spine_ref(&element, &mut spine)?,
                    "meta" if cover_id.is_none() => cover_id = cover_meta_id(&element)?,
                    _ => {}
                }
                if let Some(field) = metadata_field(name, &element)? {
                    captures.push(Capture {
                        element: name.to_string(),
                        field,
                        id: attribute_value(&element, "id")?,
                        text: String::new(),
                    });
                }
            }
            Ok(Event::Empty(element)) => {
                let name = element.local_name().into_inner();
                match name {
                    "item" => {
                        push_manifest_item(&element, base_dir, &mut manifest, &mut manifest_index)?;
                    }
                    "itemref" => push_spine_ref(&element, &mut spine)?,
                    "meta" if cover_id.is_none() => cover_id = cover_meta_id(&element)?,
                    _ => {}
                }
            }
            Ok(Event::End(element)) => {
                depth = depth.saturating_sub(1);
                let name = element.local_name().into_inner();
                if captures.last().map(|c| c.element.as_str()) == Some(name)
                    && let Some(capture) = captures.pop()
                {
                    collected.push(CollectedField {
                        field: capture.field,
                        id: capture.id,
                        text: capture.text,
                    });
                }
            }
            Ok(Event::Text(text)) => {
                if let Some(capture) = captures.last_mut() {
                    capture.text.push_str(&text.into_inner());
                }
            }
            Ok(Event::CData(data)) => {
                if let Some(capture) = captures.last_mut() {
                    capture.text.push_str(&data.into_inner());
                }
            }
            Ok(Event::GeneralRef(reference)) => {
                let Some(capture) = captures.last_mut() else {
                    continue;
                };
                if reference.is_char_ref() {
                    if let Ok(Some(ch)) = reference.resolve_char_ref() {
                        capture.text.push(ch);
                    }
                } else if let Some(ch) = named_reference(&reference.into_inner()) {
                    capture.text.push(ch);
                }
            }
            Ok(Event::Eof) => {
                if depth != 0 {
                    return Err(CoreError::archive(format!(
                        "{opf_path} の要素が閉じられていません"
                    )));
                }
                break;
            }
            Ok(_) => {}
            Err(err) => {
                return Err(CoreError::archive(format!(
                    "{opf_path} を解析できません: {err}"
                )));
            }
        }
    }

    let metadata = build_metadata(&collected, unique_identifier, version);
    let chapters = build_chapters(&manifest, &manifest_index, &spine)?;

    Ok(ParsedOpf {
        metadata,
        chapters,
        manifest,
        base_dir: base_dir.to_string(),
        cover_id,
    })
}

/// `<meta name="cover">` の `content`（表紙画像の manifest の `id`）を返す。
///
/// EPUB2 の表紙指定。`name` 属性を持たない `<meta>`（`property` による
/// メタデータ）と、`content` が空の `<meta>` は `None`。
fn cover_meta_id(element: &BytesStart<'_>) -> CoreResult<Option<String>> {
    let Some(name) = attribute_value(element, "name")? else {
        return Ok(None);
    };
    if !name.trim().eq_ignore_ascii_case(COVER_META_NAME) {
        return Ok(None);
    }
    Ok(attribute_value(element, "content")?
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty()))
}

/// メタデータ収集の対象になる要素かどうか。
fn metadata_field(name: &str, element: &BytesStart<'_>) -> CoreResult<Option<MetadataField>> {
    if let Some(field) = MetadataField::from_element(name) {
        return Ok(Some(field));
    }
    if name == "meta"
        && let Some(property) = attribute_value(element, "property")?
        && property.trim() == MODIFIED_PROPERTY
    {
        return Ok(Some(MetadataField::Modified));
    }
    Ok(None)
}

/// `package` 要素の属性を読む。
fn read_package_attributes(
    element: &BytesStart<'_>,
    version: &mut Option<String>,
    unique_identifier: &mut Option<String>,
) -> CoreResult<()> {
    if version.is_none() {
        *version = attribute_value(element, "version")?
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
    }
    if unique_identifier.is_none() {
        *unique_identifier =
            attribute_value(element, "unique-identifier")?.filter(|value| !value.trim().is_empty());
    }
    Ok(())
}

/// `item` を manifest に追加する。
fn push_manifest_item(
    element: &BytesStart<'_>,
    base_dir: &str,
    manifest: &mut Vec<ManifestItem>,
    index: &mut HashMap<String, usize>,
) -> CoreResult<()> {
    let Some(id) = attribute_value(element, "id")? else {
        return Ok(());
    };
    let Some(href) = attribute_value(element, "href")? else {
        return Err(CoreError::archive(format!(
            "manifest の item に href がありません: {id}"
        )));
    };
    let media_type = attribute_value(element, "media-type")?.unwrap_or_default();
    let properties = attribute_value(element, "properties")?.unwrap_or_default();
    let resolved = resolve_href(base_dir, &href)?;

    index.entry(id.clone()).or_insert(manifest.len());
    manifest.push(ManifestItem {
        id,
        href: resolved,
        media_type,
        properties,
    });
    Ok(())
}

/// `itemref` を spine の順に追加する。
fn push_spine_ref(element: &BytesStart<'_>, spine: &mut Vec<SpineRef>) -> CoreResult<()> {
    let Some(idref) = attribute_value(element, "idref")? else {
        return Err(CoreError::archive("spine の itemref に idref がありません"));
    };
    let linear = attribute_value(element, "linear")?.is_none_or(|value| value.trim() != "no");
    spine.push(SpineRef { idref, linear });
    Ok(())
}

/// 収集した要素から [`EpubMetadata`] を組み立てる。
fn build_metadata(
    collected: &[CollectedField],
    unique_identifier: Option<String>,
    version: Option<String>,
) -> EpubMetadata {
    let mut metadata = EpubMetadata {
        version,
        ..EpubMetadata::default()
    };
    let mut identifiers: Vec<(Option<String>, String)> = Vec::new();

    for entry in collected {
        let text = normalize_text(&entry.text);
        if text.is_empty() {
            continue;
        }
        match entry.field {
            MetadataField::Title => {
                if metadata.title.is_none() {
                    metadata.title = Some(text);
                }
            }
            MetadataField::Creator => metadata.creators.push(text),
            MetadataField::Language => {
                if metadata.language.is_none() {
                    metadata.language = Some(text);
                }
            }
            MetadataField::Identifier => identifiers.push((entry.id.clone(), text)),
            MetadataField::Publisher => {
                if metadata.publisher.is_none() {
                    metadata.publisher = Some(text);
                }
            }
            MetadataField::Date => {
                if metadata.date.is_none() {
                    metadata.date = Some(text);
                }
            }
            MetadataField::Modified => {
                if metadata.modified.is_none() {
                    metadata.modified = Some(text);
                }
            }
        }
    }

    // `unique-identifier` と一致する `id` を優先し、無ければ最初の値を使う。
    let selected = unique_identifier.as_deref().and_then(|want| {
        identifiers
            .iter()
            .find(|(id, _)| id.as_deref() == Some(want))
    });
    metadata.identifier = selected
        .or_else(|| identifiers.first())
        .map(|(_, text)| text.clone());

    metadata
}

/// spine の順に章を組み立てる。
fn build_chapters(
    manifest: &[ManifestItem],
    index: &HashMap<String, usize>,
    spine: &[SpineRef],
) -> CoreResult<Vec<EpubChapter>> {
    let mut chapters = Vec::with_capacity(spine.len());
    for reference in spine {
        let Some(&position) = index.get(&reference.idref) else {
            return Err(CoreError::archive(format!(
                "spine の idref が manifest にありません: {}",
                reference.idref
            )));
        };
        let Some(item) = manifest.get(position) else {
            return Err(CoreError::archive(format!(
                "manifest の索引が壊れています: {}",
                reference.idref
            )));
        };
        chapters.push(EpubChapter {
            id: item.id.clone(),
            href: item.href.clone(),
            media_type: item.media_type.clone(),
            linear: reference.linear,
        });
    }
    Ok(chapters)
}

/// `properties`（空白区切りのトークン列）に指定のトークンが含まれるかどうか。
fn has_property(properties: &str, token: &str) -> bool {
    properties.split_whitespace().any(|value| value == token)
}

/// 表紙に使える画像の media-type かどうか。
///
/// `image/` 以外と明示されている item は表紙にしない（表紙ページの XHTML を
/// 指す `id` を返さないため）。未宣言（空文字列）は判定材料が無いため許容する。
fn is_image_media_type(media_type: &str) -> bool {
    let value = media_type.trim();
    value.is_empty() || value.to_ascii_lowercase().starts_with("image/")
}

/// 目次のエントリを追加する。上限を超える場合は [`CoreError::Validation`]。
fn push_toc_entry(entries: &mut Vec<EpubTocEntry>, entry: EpubTocEntry) -> CoreResult<()> {
    if entries.len() >= MAX_TOC_ENTRIES {
        return Err(CoreError::validation(format!(
            "目次のエントリ数が上限を超えています: {MAX_TOC_ENTRIES}"
        )));
    }
    entries.push(entry);
    Ok(())
}

/// EPUB3 の nav 文書から目次を取り出す。
fn parse_nav(xml: &str, base_dir: &str, label: &str) -> CoreResult<Vec<EpubTocEntry>> {
    let Some(target) = select_nav(xml, label)? else {
        return Ok(Vec::new());
    };
    collect_nav_entries(xml, target, base_dir, label)
}

/// 目次として読む `<nav>` の索引（文書順、0 始まり）を選ぶ。
///
/// キーに `:` を含みローカル名が `type` の属性（`epub:type` など）の値に
/// `toc` を含む最初の `<nav>` を優先し、無ければ最初の `<nav>` を使う。
/// `<nav>` が 1 つも無い場合は `None`。
fn select_nav(xml: &str, label: &str) -> CoreResult<Option<usize>> {
    select_nav_matching(xml, label, TOC_TYPE, true)
}

/// 指定された EPUB type を持つ `<nav>` の索引を選ぶ。
fn select_nav_type(xml: &str, label: &str, navigation_type: &str) -> CoreResult<Option<usize>> {
    select_nav_matching(xml, label, navigation_type, false)
}

/// 指定された EPUB type の `<nav>` を選び、必要なら最初の nav にフォールバックする。
fn select_nav_matching(
    xml: &str,
    label: &str,
    navigation_type: &str,
    fallback_to_first: bool,
) -> CoreResult<Option<usize>> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);

    let mut count: usize = 0;
    let mut first: Option<usize> = None;
    let mut toc: Option<usize> = None;
    let mut depth: usize = 0;

    loop {
        match reader.read_event() {
            Ok(Event::Start(element)) => {
                depth += 1;
                if element.local_name().into_inner() == NAV_ELEMENT {
                    if first.is_none() {
                        first = Some(count);
                    }
                    if toc.is_none() && has_epub_type(&element, navigation_type)? {
                        toc = Some(count);
                    }
                    count += 1;
                }
            }
            Ok(Event::Empty(element)) => {
                if element.local_name().into_inner() == NAV_ELEMENT {
                    if first.is_none() {
                        first = Some(count);
                    }
                    if toc.is_none() && has_epub_type(&element, navigation_type)? {
                        toc = Some(count);
                    }
                    count += 1;
                }
            }
            Ok(Event::End(_)) => depth = depth.saturating_sub(1),
            Ok(Event::Eof) => {
                if depth != 0 {
                    return Err(CoreError::archive(format!(
                        "{label} の要素が閉じられていません"
                    )));
                }
                break;
            }
            Ok(_) => {}
            Err(err) => {
                return Err(CoreError::archive(format!(
                    "{label} を解析できません: {err}"
                )));
            }
        }
    }

    Ok(toc.or_else(|| fallback_to_first.then_some(first).flatten()))
}

/// `<nav>` の接頭辞付き `type` 属性（`epub:type` など）に指定値があるかどうか。
///
/// 接頭辞の無い `type` 属性は EPUB navigation 指定ではないため対象にしない。
fn has_epub_type(element: &BytesStart<'_>, expected: &str) -> CoreResult<bool> {
    for attr in element.attributes().flatten() {
        if !attr.key.as_ref().contains(':') {
            continue;
        }
        if attr.key.local_name().into_inner() != "type" {
            continue;
        }
        let value = unescape_attribute(&attr, "type")?;
        if value.split_whitespace().any(|token| token == expected) {
            return Ok(true);
        }
    }
    Ok(false)
}

/// 収集中の `<a>`。
struct NavAnchor {
    /// 表示名（正規化前）。
    title: String,
    /// `<a>` の内側で開いている要素の数。
    nesting: usize,
    /// 祖先 `<ol>` の数から決まる深さ。
    depth: usize,
    /// 解決済みのリンク先（フラグメントを含まない）。
    href: String,
    /// 生の `href` の `#` の後ろ。
    fragment: Option<String>,
}

/// 選んだ `<nav>` の入れ子を文書順に辿って目次を取り出す。
///
/// `<a href="...">` ごとに 1 エントリを作り、深さは祖先 `<ol>` の数 − 1 とする。
/// `<a>` を持たない `<li>`（`<span>` だけの見出し）はエントリにしない。`href` が
/// 無い、またはパス部が空の `<a>` もエントリにしない（解析は続ける）。
fn collect_nav_entries(
    xml: &str,
    target: usize,
    base_dir: &str,
    label: &str,
) -> CoreResult<Vec<EpubTocEntry>> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);

    let mut entries: Vec<EpubTocEntry> = Vec::new();
    let mut nav_index: usize = 0;
    let mut in_target = false;
    let mut nested_nav: usize = 0;
    let mut ol_depth: usize = 0;
    let mut anchor: Option<NavAnchor> = None;
    let mut depth: usize = 0;

    loop {
        match reader.read_event() {
            Ok(Event::Start(element)) => {
                depth += 1;
                let name = element.local_name().into_inner();
                if !in_target {
                    if name == NAV_ELEMENT {
                        if nav_index == target {
                            in_target = true;
                        }
                        nav_index += 1;
                    }
                    continue;
                }
                // 収集中の `<a>` の内側で開いた要素は 1 段として数える。
                if let Some(state) = anchor.as_mut() {
                    state.nesting += 1;
                }
                match (name, anchor.is_none()) {
                    (NAV_ELEMENT, _) => nested_nav += 1,
                    ("ol", true) => ol_depth += 1,
                    ("a", true) => {
                        // フラグメントの分割は `<a>` の開始時に 1 回だけ行う。
                        // パス部が空の `<a>` はエントリにせず読み飛ばす。
                        if let Some(raw) = attribute_value(&element, "href")? {
                            let (path, fragment) = split_fragment(&raw);
                            if !path.is_empty() {
                                anchor = Some(NavAnchor {
                                    title: String::new(),
                                    nesting: 0,
                                    depth: ol_depth.saturating_sub(1),
                                    href: resolve_href(base_dir, path)?,
                                    fragment,
                                });
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Empty(element)) => {
                if !in_target && element.local_name().into_inner() == NAV_ELEMENT {
                    // 自己終了した `<nav>` は内容を持たない。
                    nav_index += 1;
                }
            }
            Ok(Event::End(element)) => {
                depth = depth.saturating_sub(1);
                let name = element.local_name().into_inner();
                if !in_target {
                    continue;
                }
                if let Some(state) = anchor.as_mut()
                    && state.nesting > 0
                {
                    state.nesting -= 1;
                    continue;
                }
                if let Some(state) = anchor.take() {
                    push_toc_entry(
                        &mut entries,
                        EpubTocEntry {
                            title: normalize_text(&state.title),
                            href: state.href,
                            fragment: state.fragment,
                            depth: state.depth,
                        },
                    )?;
                    continue;
                }
                match name {
                    NAV_ELEMENT => {
                        if nested_nav > 0 {
                            nested_nav -= 1;
                        } else {
                            break;
                        }
                    }
                    "ol" => ol_depth = ol_depth.saturating_sub(1),
                    _ => {}
                }
            }
            Ok(Event::Text(text)) => {
                if let Some(state) = anchor.as_mut() {
                    state.title.push_str(&text.into_inner());
                }
            }
            Ok(Event::CData(data)) => {
                if let Some(state) = anchor.as_mut() {
                    state.title.push_str(&data.into_inner());
                }
            }
            Ok(Event::GeneralRef(reference)) => {
                let Some(state) = anchor.as_mut() else {
                    continue;
                };
                if reference.is_char_ref() {
                    if let Ok(Some(ch)) = reference.resolve_char_ref() {
                        state.title.push(ch);
                    }
                } else if let Some(ch) = named_reference(&reference.into_inner()) {
                    state.title.push(ch);
                }
            }
            Ok(Event::Eof) => {
                if depth != 0 {
                    return Err(CoreError::archive(format!(
                        "{label} の要素が閉じられていません"
                    )));
                }
                break;
            }
            Ok(_) => {}
            Err(err) => {
                return Err(CoreError::archive(format!(
                    "{label} を解析できません: {err}"
                )));
            }
        }
    }

    Ok(entries)
}

/// 収集中の `navPoint`。
struct NcxNavPoint {
    /// 祖先 `navPoint` の数。
    depth: usize,
    /// `<navLabel>` の `<text>` の内容。
    title: Option<String>,
    /// `<content>` の `src` のパス部（解決前。フラグメントを含まない）。
    href: Option<String>,
    /// `<content>` の `src` の `#` の後ろ。
    fragment: Option<String>,
}

/// EPUB2 の NCX 文書から目次を取り出す。
///
/// `<navMap>` の下の入れ子 `navPoint` を文書順に辿る（`playOrder` は無視する）。
/// `<text>` と `<content>` の両方を持ち、`src` のパス部が空でない `navPoint`
/// だけをエントリにする。
fn parse_ncx(xml: &str, base_dir: &str, label: &str) -> CoreResult<Vec<EpubTocEntry>> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);

    // 確定した `navPoint`。開始タグの順（親が子より先）に並ぶ。
    let mut points: Vec<NcxNavPoint> = Vec::new();
    // 開いている `navPoint` の `points` 索引。
    let mut open: Vec<usize> = Vec::new();
    // `<navLabel>` の内側にいるかどうか。
    let mut in_label = false;
    // 収集中の `<text>` の内容と、その内側で開いている要素の数。
    let mut text: Option<String> = None;
    let mut text_nesting: usize = 0;
    let mut depth: usize = 0;

    loop {
        match reader.read_event() {
            Ok(Event::Start(element)) => {
                depth += 1;
                let name = element.local_name().into_inner();
                // 収集中の `<text>` の内側で開いた要素は 1 段として数える。
                if text.is_some() {
                    text_nesting += 1;
                }
                match name {
                    "navPoint" => {
                        points.push(NcxNavPoint {
                            depth: open.len(),
                            title: None,
                            href: None,
                            fragment: None,
                        });
                        open.push(points.len().saturating_sub(1));
                    }
                    "navLabel" => in_label = true,
                    "text" => {
                        if in_label && text.is_none() {
                            text = Some(String::new());
                        }
                    }
                    "content" => {
                        if let Some(&index) = open.last() {
                            set_ncx_href(&mut points, index, &element)?;
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Empty(element)) => match element.local_name().into_inner() {
                "content" => {
                    if let Some(&index) = open.last() {
                        set_ncx_href(&mut points, index, &element)?;
                    }
                }
                "text" => {
                    if in_label && let Some(&index) = open.last() {
                        set_ncx_title(&mut points, index, String::new());
                    }
                }
                _ => {}
            },
            Ok(Event::End(element)) => {
                depth = depth.saturating_sub(1);
                let name = element.local_name().into_inner();
                if text.is_some() && text_nesting > 0 {
                    text_nesting -= 1;
                    continue;
                }
                if let Some(contents) = text.take() {
                    if let Some(&index) = open.last() {
                        set_ncx_title(&mut points, index, contents);
                    }
                    continue;
                }
                match name {
                    "navPoint" => {
                        open.pop();
                    }
                    "navLabel" => in_label = false,
                    _ => {}
                }
            }
            Ok(Event::Text(value)) => {
                if let Some(contents) = text.as_mut() {
                    contents.push_str(&value.into_inner());
                }
            }
            Ok(Event::CData(value)) => {
                if let Some(contents) = text.as_mut() {
                    contents.push_str(&value.into_inner());
                }
            }
            Ok(Event::GeneralRef(reference)) => {
                let Some(contents) = text.as_mut() else {
                    continue;
                };
                if reference.is_char_ref() {
                    if let Ok(Some(ch)) = reference.resolve_char_ref() {
                        contents.push(ch);
                    }
                } else if let Some(ch) = named_reference(&reference.into_inner()) {
                    contents.push(ch);
                }
            }
            Ok(Event::Eof) => {
                if depth != 0 {
                    return Err(CoreError::archive(format!(
                        "{label} の要素が閉じられていません"
                    )));
                }
                break;
            }
            Ok(_) => {}
            Err(err) => {
                return Err(CoreError::archive(format!(
                    "{label} を解析できません: {err}"
                )));
            }
        }
    }

    let mut entries: Vec<EpubTocEntry> = Vec::with_capacity(points.len());
    for point in &points {
        let (Some(title), Some(href)) = (&point.title, &point.href) else {
            continue;
        };
        // パス部が空の `navPoint` はエントリにせず読み飛ばす。
        if href.is_empty() {
            continue;
        }
        push_toc_entry(
            &mut entries,
            EpubTocEntry {
                title: normalize_text(title),
                href: resolve_href(base_dir, href)?,
                fragment: point.fragment.clone(),
                depth: point.depth,
            },
        )?;
    }
    Ok(entries)
}

/// `<content>` の `src` をパス部とフラグメントに分けて最初の値だけ記録する。
fn set_ncx_href(
    points: &mut [NcxNavPoint],
    index: usize,
    element: &BytesStart<'_>,
) -> CoreResult<()> {
    let Some(point) = points.get_mut(index) else {
        return Ok(());
    };
    if point.href.is_none()
        && let Some(raw) = attribute_value(element, "src")?
    {
        let (path, fragment) = split_fragment(&raw);
        point.href = Some(path.to_string());
        point.fragment = fragment;
    }
    Ok(())
}

/// `<text>` の内容を最初の値だけ `navPoint` に記録する。
fn set_ncx_title(points: &mut [NcxNavPoint], index: usize, value: String) {
    if let Some(point) = points.get_mut(index)
        && point.title.is_none()
    {
        point.title = Some(value);
    }
}

/// XHTML の `<head>` の最初の `<title>` の内容を正規化して返す。無ければ `None`。
///
/// 実体参照は定義済みの 5 つと数値参照だけを文字に戻す（本文抽出と同じ規則）。
/// 空白だけの `<title>` は情報が無いものとして `None` にする。`<body>` に入った
/// 後に現れる `<title>`（インライン SVG のアクセシブル名など）は章の表示名では
/// ないため対象にしない。要素が閉じられていない・解析できない場合は
/// [`CoreError::Archive`]。
fn extract_title(xml: &str, label: &str) -> CoreResult<Option<String>> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);

    // 収集中の `<title>` の内容。`None` なら未収集または収集済み。
    let mut text: Option<String> = None;
    // 収集中の `<title>` の内側で開いている要素の数。
    let mut nesting: usize = 0;
    // `<body>` に入ったかどうか。
    let mut in_body = false;
    let mut depth: usize = 0;

    loop {
        match reader.read_event() {
            Ok(Event::Start(element)) => {
                depth += 1;
                if text.is_some() {
                    nesting += 1;
                    continue;
                }
                match element.local_name().into_inner() {
                    BODY_ELEMENT => in_body = true,
                    TITLE_ELEMENT if !in_body => text = Some(String::new()),
                    _ => {}
                }
            }
            Ok(Event::Empty(element)) => {
                // 自己終了した `<title/>` は内容を持たない。
                if text.is_none() && !in_body && element.local_name().into_inner() == TITLE_ELEMENT
                {
                    return Ok(None);
                }
            }
            Ok(Event::End(_)) => {
                depth = depth.saturating_sub(1);
                if text.is_some() && nesting > 0 {
                    nesting -= 1;
                    continue;
                }
                if let Some(contents) = text.take() {
                    let title = normalize_text(&contents);
                    return Ok(if title.is_empty() { None } else { Some(title) });
                }
            }
            Ok(Event::Text(value)) => {
                if let Some(contents) = text.as_mut() {
                    contents.push_str(&value.into_inner());
                }
            }
            Ok(Event::CData(value)) => {
                if let Some(contents) = text.as_mut() {
                    contents.push_str(&value.into_inner());
                }
            }
            Ok(Event::GeneralRef(reference)) => {
                let Some(contents) = text.as_mut() else {
                    continue;
                };
                if reference.is_char_ref() {
                    if let Ok(Some(ch)) = reference.resolve_char_ref() {
                        contents.push(ch);
                    }
                } else if let Some(ch) = named_reference(&reference.into_inner()) {
                    contents.push(ch);
                }
            }
            Ok(Event::Eof) => {
                if depth != 0 {
                    return Err(CoreError::archive(format!(
                        "{label} の要素が閉じられていません"
                    )));
                }
                break;
            }
            Ok(_) => {}
            Err(err) => {
                return Err(CoreError::archive(format!(
                    "{label} を解析できません: {err}"
                )));
            }
        }
    }

    Ok(None)
}

/// XHTML から本文テキストを抽出して正規化する。
///
/// `head`・`script`・`style` の内側は含めない。ブロック要素の終了タグと `br`
/// で改行する。実体参照は定義済みの 5 つと数値参照だけを文字に戻し、未知の
/// 名前付き参照は無視する。
fn extract_text(xml: &str, label: &str) -> CoreResult<String> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);

    let mut raw = String::new();
    let mut skip_depth: usize = 0;
    let mut depth: usize = 0;

    loop {
        match reader.read_event() {
            Ok(Event::Start(element)) => {
                depth += 1;
                let name = element.local_name().into_inner();
                if skip_depth > 0 {
                    if is_skipped(name) {
                        skip_depth += 1;
                    }
                    continue;
                }
                if is_skipped(name) {
                    skip_depth = 1;
                }
            }
            Ok(Event::Empty(element)) => {
                let name = element.local_name().into_inner();
                if skip_depth > 0 || is_skipped(name) {
                    continue;
                }
                if name == "br" || is_block(name) {
                    raw.push('\n');
                }
            }
            Ok(Event::End(element)) => {
                depth = depth.saturating_sub(1);
                let name = element.local_name().into_inner();
                if skip_depth > 0 {
                    if is_skipped(name) {
                        skip_depth -= 1;
                    }
                    continue;
                }
                if name == "br" || is_block(name) {
                    raw.push('\n');
                }
            }
            Ok(Event::Text(text)) => {
                if skip_depth == 0 {
                    raw.push_str(&text.into_inner());
                }
            }
            Ok(Event::CData(data)) => {
                if skip_depth == 0 {
                    raw.push_str(&data.into_inner());
                }
            }
            Ok(Event::GeneralRef(reference)) => {
                if skip_depth > 0 {
                    continue;
                }
                if reference.is_char_ref() {
                    if let Ok(Some(ch)) = reference.resolve_char_ref() {
                        raw.push(ch);
                    }
                } else if let Some(ch) = named_reference(&reference.into_inner()) {
                    raw.push(ch);
                }
            }
            Ok(Event::Eof) => {
                if depth != 0 || skip_depth != 0 {
                    return Err(CoreError::archive(format!(
                        "{label} の要素が閉じられていません"
                    )));
                }
                break;
            }
            Ok(_) => {}
            Err(err) => {
                return Err(CoreError::archive(format!(
                    "{label} を解析できません: {err}"
                )));
            }
        }
    }

    Ok(normalize_lines(&raw))
}

/// 本文抽出で内容を除外する要素かどうか。
fn is_skipped(name: &str) -> bool {
    SKIPPED_ELEMENTS.contains(&name)
}

/// 本文抽出で改行を挿入する要素かどうか。
fn is_block(name: &str) -> bool {
    BLOCK_ELEMENTS.contains(&name)
}

/// 定義済みの名前付き参照を文字に戻す。未知の名前は `None`。
fn named_reference(name: &str) -> Option<char> {
    match name {
        "amp" => Some('&'),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "quot" => Some('"'),
        "apos" => Some('\''),
        _ => None,
    }
}

/// 空白の連続を 1 つの半角空白にまとめ、前後の空白を落とす。
fn normalize_text(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// 行ごとに空白を 1 つにまとめ、空行を落として改行で連結する。
fn normalize_lines(raw: &str) -> String {
    raw.split('\n')
        .map(normalize_text)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

/// `%XX`（16 進 2 桁）を 1 バイトに復号する。
///
/// `%` を含まない場合と、復号結果が不正な UTF-8 になる場合は `None` を返す
/// （呼び出し側は元の名前を使う）。`+` は空白にしない。
fn percent_decode(name: &str) -> Option<String> {
    if !name.contains('%') {
        return None;
    }

    let bytes = name.as_bytes();
    let mut decoded: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut position = 0;
    while position < bytes.len() {
        if bytes[position] == b'%'
            && position + 2 < bytes.len()
            && let (Some(high), Some(low)) = (
                hex_digit(bytes[position + 1]),
                hex_digit(bytes[position + 2]),
            )
        {
            decoded.push(high * 16 + low);
            position += 3;
            continue;
        }
        decoded.push(bytes[position]);
        position += 1;
    }

    String::from_utf8(decoded).ok()
}

/// 章の `href` を比較するための正規化したパスを返す。
///
/// `%XX` を復号した形にする。復号できなければ生の値をそのまま使う。
fn normalized_chapter_path(path: &str) -> String {
    percent_decode(path).unwrap_or_else(|| path.to_string())
}

/// 16 進 1 桁を数値にする。
fn hex_digit(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Write};
    use zip::CompressionMethod;
    use zip::ZipWriter;
    use zip::write::SimpleFileOptions;

    /// テストで使う `META-INF/container.xml`（OPF は `OEBPS/content.opf`）。
    const CONTAINER: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>
"#;

    /// テストで使う OPF。`linear="no"` の章と EPUB2 形式の `meta` を含む。
    const OPF: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="pub-id">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:title>サンプル本</dc:title>
    <dc:creator>Alice</dc:creator>
    <dc:creator>Bob</dc:creator>
    <dc:language>ja</dc:language>
    <dc:identifier id="other">urn:uuid:0000</dc:identifier>
    <dc:identifier id="pub-id">urn:isbn:9784000000000</dc:identifier>
    <dc:publisher>Example Press</dc:publisher>
    <dc:date>2024-05-01</dc:date>
    <meta property="dcterms:modified">2024-05-02T10:00:00Z</meta>
    <meta name="cover" content="cover-image"/>
  </metadata>
  <manifest>
    <item id="ch1" href="ch1.xhtml" media-type="application/xhtml+xml"/>
    <item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/>
    <item id="ch2" href="ch2.xhtml" media-type="application/xhtml+xml"/>
  </manifest>
  <spine>
    <itemref idref="ch1"/>
    <itemref idref="ch2" linear="no"/>
  </spine>
</package>
"#;

    /// 接頭辞付きの OPF。要素名のローカル名だけで解析できることを確認する。
    const OPF_PREFIXED: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<opf:package xmlns:opf="http://www.idpf.org/2007/opf"
             xmlns:dc="http://purl.org/dc/elements/1.1/"
             version="3.0" unique-identifier="pub-id">
  <opf:metadata>
    <dc:title>サンプル本</dc:title>
    <dc:creator>Alice</dc:creator>
    <dc:creator>Bob</dc:creator>
    <dc:language>ja</dc:language>
    <dc:identifier id="pub-id">urn:isbn:9784000000000</dc:identifier>
    <dc:publisher>Example Press</dc:publisher>
    <dc:date>2024-05-01</dc:date>
    <opf:meta property="dcterms:modified">2024-05-02T10:00:00Z</opf:meta>
  </opf:metadata>
  <opf:manifest>
    <opf:item id="ch1" href="ch1.xhtml" media-type="application/xhtml+xml"/>
    <opf:item id="ch2" href="ch2.xhtml" media-type="application/xhtml+xml"/>
  </opf:manifest>
  <opf:spine>
    <opf:itemref idref="ch1"/>
    <opf:itemref idref="ch2" linear="no"/>
  </opf:spine>
</opf:package>
"#;

    /// 本文抽出を検証する章。`head` の `title` と `script`・`style` を含む。
    const CH1: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head>
  <title>Head Title</title>
  <style>p { color: red; }</style>
</head>
<body>
  <script>var marker = "script-marker";</script>
  <h1>T</h1>
  <p>a &amp; b</p>
  <p>c &lt;d&gt; &#x30; &foo;</p>
</body>
</html>"#;

    /// 2 章目の内容（`chapter_bytes` の検証に使う）。
    const CH2: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<html xmlns="http://www.w3.org/1999/xhtml"><body><p>第二章</p></body></html>"#;

    /// 章本文の期待値。
    const CH1_TEXT: &str = "T\na & b\nc <d> 0";

    /// 入れ子の `<ol>` / `<li>` / `<a>` を持つ EPUB3 の nav。
    const NAV_NESTED: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops">
<head><title>目次</title></head>
<body>
  <nav epub:type="toc">
    <h1>目次</h1>
    <ol>
      <li><a href="ch1.xhtml">第一章 &amp; はじめに</a></li>
      <li><span>第二部</span>
        <ol>
          <li><a href="ch2.xhtml#s1">第二章 1 節</a></li>
          <li><span>見出しだけ</span></li>
        </ol>
      </li>
    </ol>
  </nav>
</body>
</html>
"#;

    /// `<a>` を持たない `<li>` を含む nav。
    const NAV_SPAN_ONLY: &str = r#"<html xmlns:epub="http://www.idpf.org/2007/ops"><body>
  <nav epub:type="toc">
    <ol>
      <li><span>見出しだけ</span></li>
      <li><a href="ch1.xhtml">実項目</a></li>
    </ol>
  </nav>
</body></html>
"#;

    /// `landmarks` が先、`toc` が後ろにある nav。
    const NAV_LANDMARKS_FIRST: &str = r#"<html xmlns:epub="http://www.idpf.org/2007/ops"><body>
  <nav epub:type="landmarks">
    <ol><li><a href="ch1.xhtml">表紙</a></li></ol>
  </nav>
  <nav epub:type="toc">
    <ol><li><a href="ch2.xhtml">目次の項目</a></li></ol>
  </nav>
</body></html>
"#;

    /// `epub:type` を持たない `<nav>` だけの nav。
    const NAV_WITHOUT_TYPE: &str = r#"<html><body>
  <nav><ol><li><a href="ch1.xhtml">唯一の nav</a></li></ol></nav>
</body></html>
"#;

    /// フラグメント付きと無しの `<a>` を混ぜた入れ子の nav。
    const NAV_FRAGMENT_MIXED: &str = r##"<html xmlns:epub="http://www.idpf.org/2007/ops"><body>
  <nav epub:type="toc">
    <ol>
      <li><a href="ch1.xhtml#s1">第一章</a>
        <ol><li><a href="ch2.xhtml">第二章</a></li></ol>
      </li>
    </ol>
  </nav>
</body></html>
"##;

    /// パス部が空の `<a>` を挟む入れ子の nav。
    const NAV_WITH_EMPTY_PATHS: &str = r##"<html xmlns:epub="http://www.idpf.org/2007/ops"><body>
  <nav epub:type="toc">
    <ol>
      <li><a href="#only-fragment">位置だけ</a>
        <ol><li><a href="ch1.xhtml">第一章</a></li></ol>
      </li>
      <li><a href="  ">空白だけ</a></li>
      <li><a href="ch2.xhtml#s1">第二章</a></li>
    </ol>
  </nav>
</body></html>
"##;

    /// 入れ子の `navPoint` を持つ EPUB2 の NCX。
    ///
    /// `playOrder` は文書順と逆にしてあり、内容を持たない `navPoint` も含む。
    const NCX_NESTED: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<ncx xmlns="http://www.daisy.org/z3986/2005/ncx/" version="2005-1">
  <head><meta name="dtb:uid" content="urn:isbn:9784000000000"/></head>
  <docTitle><text>サンプル本</text></docTitle>
  <navMap>
    <navPoint id="n1" playOrder="3">
      <navLabel><text>第一章</text></navLabel>
      <content src="ch1.xhtml"/>
      <navPoint id="n1-1" playOrder="2">
        <navLabel><text>第一節</text></navLabel>
        <content src="ch2.xhtml#s1"/>
      </navPoint>
    </navPoint>
    <navPoint id="n2" playOrder="1">
      <navLabel><text>第二章</text></navLabel>
      <content src="ch2.xhtml"/>
    </navPoint>
    <navPoint id="n3" playOrder="4">
      <navLabel><text>内容の無い項目</text></navLabel>
    </navPoint>
  </navMap>
</ncx>
"#;

    /// NCX だけを情報源に持つ EPUB2 相当の OPF。
    const OPF_NCX: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="2.0" unique-identifier="pub-id">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:title>サンプル本</dc:title>
    <dc:identifier id="pub-id">urn:isbn:9784000000000</dc:identifier>
  </metadata>
  <manifest>
    <item id="ch1" href="ch1.xhtml" media-type="application/xhtml+xml"/>
    <item id="ch2" href="ch2.xhtml" media-type="application/xhtml+xml"/>
    <item id="ncx" href="toc.ncx" media-type="application/x-dtbncx+xml"/>
  </manifest>
  <spine>
    <itemref idref="ch1"/>
    <itemref idref="ch2"/>
  </spine>
</package>
"#;

    /// エントリを順に書き込んだ ZIP を組み立てる。
    fn write_zip(files: &[(&str, &[u8])]) -> Vec<u8> {
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        for (path, data) in files {
            let options =
                SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
            writer.start_file(*path, options).unwrap();
            writer.write_all(data).unwrap();
        }
        writer.finish().unwrap().into_inner()
    }

    /// `mimetype` を先頭に `Stored` で書いた EPUB を組み立てる。
    fn build_epub(files: &[(&str, &[u8])]) -> Vec<u8> {
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        writer
            .start_file(
                "mimetype",
                SimpleFileOptions::default().compression_method(CompressionMethod::Stored),
            )
            .unwrap();
        writer.write_all(b"application/epub+zip").unwrap();
        for (path, data) in files {
            let options =
                SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
            writer.start_file(*path, options).unwrap();
            writer.write_all(data).unwrap();
        }
        writer.finish().unwrap().into_inner()
    }

    /// `container.xml` を差し替えた EPUB を組み立てる。
    fn epub_with_container(container: &str) -> Vec<u8> {
        build_epub(&[
            ("META-INF/container.xml", container.as_bytes()),
            ("OEBPS/content.opf", OPF.as_bytes()),
            ("OEBPS/ch1.xhtml", CH1.as_bytes()),
            ("OEBPS/ch2.xhtml", CH2.as_bytes()),
        ])
    }

    /// OPF を差し替えた EPUB を組み立てる。
    fn epub_with_opf(opf: &str) -> Vec<u8> {
        build_epub(&[
            ("META-INF/container.xml", CONTAINER.as_bytes()),
            ("OEBPS/content.opf", opf.as_bytes()),
            ("OEBPS/ch1.xhtml", CH1.as_bytes()),
            ("OEBPS/ch2.xhtml", CH2.as_bytes()),
        ])
    }

    /// 標準の EPUB3 相当（`OEBPS/content.opf` と 2 章）。
    fn sample_epub() -> Vec<u8> {
        build_epub(&[
            ("META-INF/container.xml", CONTAINER.as_bytes()),
            ("OEBPS/content.opf", OPF.as_bytes()),
            ("OEBPS/ch1.xhtml", CH1.as_bytes()),
            ("OEBPS/ch2.xhtml", CH2.as_bytes()),
            (
                "OEBPS/nav.xhtml",
                b"<html><body><nav>nav</nav></body></html>",
            ),
        ])
    }

    /// `open` のエラーを取り出す。
    fn open_error(bytes: &[u8]) -> CoreError {
        match EpubBook::open(bytes) {
            Err(err) => err,
            Ok(_) => panic!("エラーを期待したが開けた"),
        }
    }

    /// 目次の XHTML（nav / NCX）と OPF を差し替えた EPUB を組み立てる。
    fn epub_with_toc(nav: Option<&str>, ncx: Option<&str>, opf: &str) -> Vec<u8> {
        let mut files: Vec<(&str, &[u8])> = vec![
            ("META-INF/container.xml", CONTAINER.as_bytes()),
            ("OEBPS/content.opf", opf.as_bytes()),
            ("OEBPS/ch1.xhtml", CH1.as_bytes()),
            ("OEBPS/ch2.xhtml", CH2.as_bytes()),
        ];
        if let Some(nav) = nav {
            files.push(("OEBPS/nav.xhtml", nav.as_bytes()));
        }
        if let Some(ncx) = ncx {
            files.push(("OEBPS/toc.ncx", ncx.as_bytes()));
        }
        build_epub(&files)
    }

    /// nav の XHTML を差し替えた EPUB（OPF は標準）を組み立てる。
    fn epub_with_nav(nav: &str) -> Vec<u8> {
        epub_with_toc(Some(nav), None, OPF)
    }

    /// `properties` を差し替えた OPF を返す。
    fn opf_with_nav_properties(properties: &str) -> String {
        OPF.replace(
            "properties=\"nav\"",
            &format!("properties=\"{properties}\""),
        )
    }

    /// nav と NCX の両方を manifest に持つ OPF を返す。
    fn opf_with_nav_and_ncx() -> String {
        OPF.replace(
            "<manifest>\n",
            "<manifest>\n    <item id=\"ncx\" href=\"toc.ncx\" media-type=\"application/x-dtbncx+xml\"/>\n",
        )
    }

    /// `<a>` を `count` 個持つ nav を組み立てる。
    fn nav_with_anchors(count: usize) -> String {
        let mut nav = String::from("<html><body><nav epub:type=\"toc\"><ol>");
        for _ in 0..count {
            nav.push_str("<li><a href=\"ch1.xhtml\">項目</a></li>");
        }
        nav.push_str("</ol></nav></body></html>");
        nav
    }

    /// `href` の並びから nav を組み立てる。`None` の要素は `href` 属性なし。
    fn nav_with_hrefs(hrefs: &[Option<&str>]) -> String {
        let mut nav = String::from("<html><body><nav epub:type=\"toc\"><ol>");
        for href in hrefs {
            match href {
                Some(value) => nav.push_str(&format!("<li><a href=\"{value}\">項目</a></li>")),
                None => nav.push_str("<li><a>項目</a></li>"),
            }
        }
        nav.push_str("</ol></nav></body></html>");
        nav
    }

    /// `src` の並びから `<navMap>` を組み立てた NCX を返す。
    fn ncx_with_sources(sources: &[&str]) -> String {
        let mut ncx = String::from(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <ncx xmlns=\"http://www.daisy.org/z3986/2005/ncx/\" version=\"2005-1\"><navMap>",
        );
        for (index, src) in sources.iter().enumerate() {
            ncx.push_str(&format!(
                "<navPoint id=\"n{index}\"><navLabel><text>項目{index}</text></navLabel>\
                 <content src=\"{src}\"/></navPoint>"
            ));
        }
        ncx.push_str("</navMap></ncx>");
        ncx
    }

    /// 期待する目次エントリを組み立てる（フラグメント無し）。
    fn entry(title: &str, href: &str, depth: usize) -> EpubTocEntry {
        EpubTocEntry {
            title: title.to_string(),
            href: href.to_string(),
            fragment: None,
            depth,
        }
    }

    /// フラグメント付きの期待する目次エントリを組み立てる。
    fn entry_with_fragment(title: &str, href: &str, fragment: &str, depth: usize) -> EpubTocEntry {
        EpubTocEntry {
            title: title.to_string(),
            href: href.to_string(),
            fragment: Some(fragment.to_string()),
            depth,
        }
    }

    /// `toc()` のエラーを取り出す。
    fn toc_error(bytes: &[u8]) -> CoreError {
        match EpubBook::open(bytes).and_then(|book| book.toc()) {
            Err(err) => err,
            Ok(entries) => panic!("エラーを期待したが {} 件返った", entries.len()),
        }
    }

    #[test]
    fn open_owned_matches_open() {
        let bytes = sample_epub();
        let borrowed = EpubBook::open(&bytes).unwrap();
        let owned = EpubBook::open_owned(bytes).unwrap();
        assert_eq!(owned.metadata(), borrowed.metadata());
        assert_eq!(owned.chapters(), borrowed.chapters());
        assert_eq!(
            owned.chapter_html(0).unwrap(),
            borrowed.chapter_html(0).unwrap()
        );
    }

    #[test]
    fn reads_epub3_metadata() {
        let bytes = sample_epub();
        let book = EpubBook::open(&bytes).unwrap();
        let metadata = book.metadata();

        assert_eq!(metadata.title.as_deref(), Some("サンプル本"));
        assert_eq!(
            metadata.creators,
            vec!["Alice".to_string(), "Bob".to_string()]
        );
        assert_eq!(metadata.language.as_deref(), Some("ja"));
        // `unique-identifier` と一致する `id` を優先する。
        assert_eq!(
            metadata.identifier.as_deref(),
            Some("urn:isbn:9784000000000")
        );
        assert_eq!(metadata.publisher.as_deref(), Some("Example Press"));
        assert_eq!(metadata.date.as_deref(), Some("2024-05-01"));
        assert_eq!(metadata.modified.as_deref(), Some("2024-05-02T10:00:00Z"));
        assert_eq!(metadata.version.as_deref(), Some("3.0"));
    }

    #[test]
    fn lists_spine_chapters_in_order() {
        let bytes = sample_epub();
        let book = EpubBook::open(&bytes).unwrap();
        let chapters = book.chapters();

        assert_eq!(chapters.len(), 2);

        assert_eq!(chapters[0].id, "ch1");
        assert_eq!(chapters[0].href, "OEBPS/ch1.xhtml");
        assert_eq!(chapters[0].media_type, "application/xhtml+xml");
        assert!(chapters[0].linear);

        assert_eq!(chapters[1].id, "ch2");
        assert_eq!(chapters[1].href, "OEBPS/ch2.xhtml");
        assert_eq!(chapters[1].media_type, "application/xhtml+xml");
        assert!(!chapters[1].linear);
    }

    #[test]
    fn extracts_normalized_chapter_text() {
        let bytes = sample_epub();
        let book = EpubBook::open(&bytes).unwrap();

        assert_eq!(book.chapter_text(0).unwrap(), CH1_TEXT);
    }

    #[test]
    fn skips_head_script_and_style_content() {
        let bytes = sample_epub();
        let book = EpubBook::open(&bytes).unwrap();
        let text = book.chapter_text(0).unwrap();

        assert!(!text.contains("Head Title"));
        assert!(!text.contains("script-marker"));
        assert!(!text.contains("color: red"));
    }

    #[test]
    fn returns_chapter_bytes() {
        let bytes = sample_epub();
        let book = EpubBook::open(&bytes).unwrap();

        assert_eq!(book.chapter_bytes(1).unwrap(), CH2.as_bytes().to_vec());
    }

    #[test]
    fn rejects_out_of_range_chapter() {
        let bytes = sample_epub();
        let book = EpubBook::open(&bytes).unwrap();

        assert!(matches!(
            book.chapter_bytes(2),
            Err(CoreError::DocumentNotFound(_))
        ));
        assert!(matches!(
            book.chapter_text(9),
            Err(CoreError::DocumentNotFound(_))
        ));
    }

    #[test]
    fn rejects_plain_zip() {
        let bytes = write_zip(&[("a.txt", b"plain")]);
        assert!(matches!(
            EpubBook::open(&bytes),
            Err(CoreError::UnsupportedFormat(_))
        ));
    }

    #[test]
    fn rejects_broken_container_or_package() {
        // container.xml 欠落。
        let missing_container = build_epub(&[("OEBPS/content.opf", OPF.as_bytes())]);
        // rootfile 要素なし。
        let no_rootfile =
            epub_with_container(r#"<?xml version="1.0"?><container><rootfiles/></container>"#);
        // OPF 欠落（full-path の先にエントリが無い）。
        let missing_opf = build_epub(&[("META-INF/container.xml", CONTAINER.as_bytes())]);
        // container.xml が壊れた XML（終了タグの不一致）。
        let broken_container = epub_with_container("<container><rootfiles></container>");
        // OPF が壊れた XML（終了タグの不一致）。
        let broken_opf = epub_with_opf("<package><metadata></package>");
        // OPF が途中で切れている。
        let truncated_opf = epub_with_opf("<package><metadata>");

        for bytes in [
            missing_container,
            no_rootfile,
            missing_opf,
            broken_container,
            broken_opf,
            truncated_opf,
        ] {
            assert!(matches!(open_error(&bytes), CoreError::Archive(_)));
        }
    }

    #[test]
    fn rejects_spine_idref_missing_from_manifest() {
        let bytes = epub_with_opf(&OPF.replace("idref=\"ch1\"", "idref=\"missing\""));
        assert!(matches!(open_error(&bytes), CoreError::Archive(_)));
    }

    #[test]
    fn rejects_href_escaping_root() {
        let bytes = epub_with_opf(&OPF.replace("href=\"ch1.xhtml\"", "href=\"../../evil.xhtml\""));
        assert!(matches!(open_error(&bytes), CoreError::Validation(_)));
    }

    #[test]
    fn resolves_href_relative_to_opf_directory() {
        let container = CONTAINER.replace("OEBPS/content.opf", "OEBPS/sub/content.opf");
        let opf = OPF
            .replace("href=\"ch1.xhtml\"", "href=\"../ch1.xhtml\"")
            .replace("href=\"ch2.xhtml\"", "href=\"../ch2.xhtml\"");
        let bytes = build_epub(&[
            ("META-INF/container.xml", container.as_bytes()),
            ("OEBPS/sub/content.opf", opf.as_bytes()),
            ("OEBPS/ch1.xhtml", CH1.as_bytes()),
        ]);

        let book = EpubBook::open(&bytes).unwrap();
        assert_eq!(book.chapters()[0].href, "OEBPS/ch1.xhtml");
        assert_eq!(book.chapter_text(0).unwrap(), CH1_TEXT);
    }

    #[test]
    fn strips_href_fragment() {
        let bytes = epub_with_opf(&OPF.replace("href=\"ch1.xhtml\"", "href=\"ch1.xhtml#s1\""));
        let book = EpubBook::open(&bytes).unwrap();

        assert_eq!(book.chapters()[0].href, "OEBPS/ch1.xhtml");
    }

    #[test]
    fn parses_prefixed_opf() {
        let bytes = build_epub(&[
            ("META-INF/container.xml", CONTAINER.as_bytes()),
            ("OEBPS/content.opf", OPF_PREFIXED.as_bytes()),
            ("OEBPS/ch1.xhtml", CH1.as_bytes()),
            ("OEBPS/ch2.xhtml", CH2.as_bytes()),
        ]);

        let book = EpubBook::open(&bytes).unwrap();
        assert_eq!(book.metadata().title.as_deref(), Some("サンプル本"));
        assert_eq!(
            book.metadata().creators,
            vec!["Alice".to_string(), "Bob".to_string()]
        );
        assert_eq!(book.metadata().version.as_deref(), Some("3.0"));
        assert_eq!(
            book.metadata().identifier.as_deref(),
            Some("urn:isbn:9784000000000")
        );
        assert_eq!(
            book.metadata().modified.as_deref(),
            Some("2024-05-02T10:00:00Z")
        );

        let chapters = book.chapters();
        assert_eq!(chapters.len(), 2);
        assert_eq!(chapters[0].href, "OEBPS/ch1.xhtml");
        assert!(chapters[0].linear);
        assert_eq!(chapters[1].href, "OEBPS/ch2.xhtml");
        assert!(!chapters[1].linear);
    }

    #[test]
    fn reads_percent_encoded_href() {
        let opf = OPF
            .replace("href=\"ch1.xhtml\"", "href=\"ch%201.xhtml\"")
            .replace("href=\"ch2.xhtml\"", "href=\"ch%202.xhtml\"");
        let bytes = build_epub(&[
            ("META-INF/container.xml", CONTAINER.as_bytes()),
            ("OEBPS/content.opf", opf.as_bytes()),
            ("OEBPS/ch 1.xhtml", CH1.as_bytes()),
        ]);

        let book = EpubBook::open(&bytes).unwrap();
        assert_eq!(book.chapters()[0].href, "OEBPS/ch%201.xhtml");
        assert_eq!(book.chapter_text(0).unwrap(), CH1_TEXT);
        // 復号できない名前は元のエラーのまま。
        assert!(matches!(
            book.chapter_bytes(1),
            Err(CoreError::DocumentNotFound(_))
        ));
    }

    #[test]
    fn resolves_href_for_opf_at_root() {
        let container = CONTAINER.replace("OEBPS/content.opf", "content.opf");
        let opf = OPF.replace("href=\"ch1.xhtml\"", "href=\"./text/ch1.xhtml\"");
        let bytes = build_epub(&[
            ("META-INF/container.xml", container.as_bytes()),
            ("content.opf", opf.as_bytes()),
            ("text/ch1.xhtml", CH1.as_bytes()),
        ]);

        let book = EpubBook::open(&bytes).unwrap();
        // OPF がルート直下なら、`./` を挟まずそのまま解決する。
        assert_eq!(book.chapters()[0].href, "text/ch1.xhtml");
        assert_eq!(book.chapter_text(0).unwrap(), CH1_TEXT);
    }

    #[test]
    fn breaks_lines_at_block_elements_and_br() {
        let chapter = concat!(
            "<html><body>",
            "<div>one<br/>two</div>",
            "<ul><li>three</li><li>four</li></ul>",
            "<table><tr><td>five</td><td>six</td></tr></table>",
            "</body></html>"
        );
        let bytes = build_epub(&[
            ("META-INF/container.xml", CONTAINER.as_bytes()),
            ("OEBPS/content.opf", OPF.as_bytes()),
            ("OEBPS/ch1.xhtml", chapter.as_bytes()),
        ]);

        let book = EpubBook::open(&bytes).unwrap();
        assert_eq!(
            book.chapter_text(0).unwrap(),
            "one\ntwo\nthree\nfour\nfive\nsix"
        );
    }

    #[test]
    fn classifies_chapter_decode_and_parse_errors() {
        let mut bom_chapter = Vec::from(&b"\xEF\xBB\xBF"[..]);
        bom_chapter.extend_from_slice(CH1.as_bytes());

        // BOM 付きの章は BOM を除去して読む。
        let with_bom = build_epub(&[
            ("META-INF/container.xml", CONTAINER.as_bytes()),
            ("OEBPS/content.opf", OPF.as_bytes()),
            ("OEBPS/ch1.xhtml", &bom_chapter),
        ]);
        let book = EpubBook::open(&with_bom).unwrap();
        assert_eq!(book.chapter_text(0).unwrap(), CH1_TEXT);

        // 不正な UTF-8。
        let invalid_utf8 = build_epub(&[
            ("META-INF/container.xml", CONTAINER.as_bytes()),
            ("OEBPS/content.opf", OPF.as_bytes()),
            ("OEBPS/ch1.xhtml", &[0xFF, 0xFE, 0x00]),
        ]);
        let book = EpubBook::open(&invalid_utf8).unwrap();
        assert!(matches!(book.chapter_text(0), Err(CoreError::Archive(_))));

        // XML として壊れた章。
        let broken_xml = build_epub(&[
            ("META-INF/container.xml", CONTAINER.as_bytes()),
            ("OEBPS/content.opf", OPF.as_bytes()),
            ("OEBPS/ch1.xhtml", b"<html><body><p>x</body></html>"),
        ]);
        let book = EpubBook::open(&broken_xml).unwrap();
        assert!(matches!(book.chapter_text(0), Err(CoreError::Archive(_))));
    }

    #[test]
    fn enforces_archive_limits() {
        let bytes = sample_epub();
        let limits = ArchiveLimits {
            max_entries: 2,
            ..ArchiveLimits::default()
        };

        assert!(matches!(
            EpubBook::with_limits(&bytes, limits),
            Err(CoreError::Validation(_))
        ));
        assert!(EpubBook::open(&bytes).is_ok());
    }

    #[test]
    fn epub_book_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<EpubBook>();
    }

    #[test]
    fn reads_epub3_nav_in_document_order() {
        let bytes = epub_with_nav(NAV_NESTED);
        let book = EpubBook::open(&bytes).unwrap();

        assert_eq!(
            book.toc().unwrap(),
            vec![
                entry("第一章 & はじめに", "OEBPS/ch1.xhtml", 0),
                entry_with_fragment("第二章 1 節", "OEBPS/ch2.xhtml", "s1", 1),
            ]
        );
    }

    #[test]
    fn skips_nav_items_without_anchor() {
        let bytes = epub_with_nav(NAV_SPAN_ONLY);
        let book = EpubBook::open(&bytes).unwrap();

        // `<span>` だけの `<li>` はエントリにしない。
        assert_eq!(
            book.toc().unwrap(),
            vec![entry("実項目", "OEBPS/ch1.xhtml", 0)]
        );
    }

    #[test]
    fn drops_fragment_from_nav_href() {
        let bytes = epub_with_nav(NAV_NESTED);
        let book = EpubBook::open(&bytes).unwrap();
        let toc = book.toc().unwrap();

        // `<a href="ch2.xhtml#s1">` はフラグメントを落として章と同じパスになる。
        assert_eq!(toc[1].href, "OEBPS/ch2.xhtml");
        assert_eq!(toc[1].href, book.chapters()[1].href);
        // フラグメント自体は `fragment` に保持する。
        assert_eq!(toc[1].fragment.as_deref(), Some("s1"));
        assert_eq!(toc[0].fragment, None);
    }

    #[test]
    fn keeps_nav_fragment_alongside_resolved_href() {
        let bytes = epub_with_nav(NAV_FRAGMENT_MIXED);
        let book = EpubBook::open(&bytes).unwrap();
        let toc = book.toc().unwrap();

        assert_eq!(toc.len(), 2);
        // フラグメント付きでも `href` は章と同じ値になる。
        assert_eq!(toc[0].href, "OEBPS/ch1.xhtml");
        assert_eq!(toc[0].href, book.chapters()[0].href);
        assert_eq!(toc[0].fragment.as_deref(), Some("s1"));
        assert_eq!(toc[0].depth, 0);
        // フラグメント無しは `None`。
        assert_eq!(toc[1].href, "OEBPS/ch2.xhtml");
        assert_eq!(toc[1].fragment, None);
        assert_eq!(toc[1].depth, 1);
    }

    #[test]
    fn treats_empty_nav_fragment_as_none() {
        // `#` の後ろが空なら `None`。
        let bytes = epub_with_nav(&nav_with_hrefs(&[Some("ch1.xhtml#")]));
        let book = EpubBook::open(&bytes).unwrap();

        assert_eq!(
            book.toc().unwrap(),
            vec![entry("項目", "OEBPS/ch1.xhtml", 0)]
        );
    }

    #[test]
    fn keeps_nav_fragment_verbatim() {
        let bytes = epub_with_nav(&nav_with_hrefs(&[
            Some("ch1.xhtml#%E3%81%82"),
            Some("ch2.xhtml#a#b"),
        ]));
        let book = EpubBook::open(&bytes).unwrap();
        let toc = book.toc().unwrap();

        // パーセントエンコードは復号しない。
        assert_eq!(toc[0].fragment.as_deref(), Some("%E3%81%82"));
        // 2 個目以降の `#` は区切りにしない。
        assert_eq!(toc[1].fragment.as_deref(), Some("a#b"));
    }

    #[test]
    fn decodes_nav_fragment_entities_and_trims_spaces() {
        let bytes = epub_with_nav(&nav_with_hrefs(&[
            Some("ch1.xhtml#a&amp;b"),
            Some("ch2.xhtml# s1 "),
        ]));
        let book = EpubBook::open(&bytes).unwrap();
        let toc = book.toc().unwrap();

        // 属性の復号で戻った `&` をそのまま保持する。
        assert_eq!(toc[0].fragment.as_deref(), Some("a&b"));
        // 前後の空白は落とす。
        assert_eq!(toc[1].fragment.as_deref(), Some("s1"));
    }

    #[test]
    fn skips_nav_anchor_without_path() {
        let bytes = epub_with_nav(&nav_with_hrefs(&[Some("#s1"), Some("ch1.xhtml")]));
        let book = EpubBook::open(&bytes).unwrap();

        // フラグメントだけの `<a>` は読み飛ばし、残りのエントリは残す。
        assert_eq!(
            book.toc().unwrap(),
            vec![entry("項目", "OEBPS/ch1.xhtml", 0)]
        );
    }

    #[test]
    fn skips_nav_anchor_without_href() {
        let bytes = epub_with_nav(&nav_with_hrefs(&[
            None,
            Some(""),
            Some("  "),
            Some("ch1.xhtml"),
        ]));
        let book = EpubBook::open(&bytes).unwrap();

        // `href` が無い・空・空白だけの `<a>` はエントリにしない。
        assert_eq!(
            book.toc().unwrap(),
            vec![entry("項目", "OEBPS/ch1.xhtml", 0)]
        );
    }

    #[test]
    fn keeps_depth_after_empty_path_anchor() {
        let bytes = epub_with_nav(NAV_WITH_EMPTY_PATHS);
        let book = EpubBook::open(&bytes).unwrap();

        // パス部が空の `<a>` を挟んでも深さの規則は変わらない。
        assert_eq!(
            book.toc().unwrap(),
            vec![
                entry("第一章", "OEBPS/ch1.xhtml", 1),
                entry_with_fragment("第二章", "OEBPS/ch2.xhtml", "s1", 0),
            ]
        );
    }

    #[test]
    fn prefers_nav_marked_as_toc() {
        let bytes = epub_with_nav(NAV_LANDMARKS_FIRST);
        let book = EpubBook::open(&bytes).unwrap();

        // `landmarks` の nav ではなく `toc` の nav を使う。
        assert_eq!(
            book.toc().unwrap(),
            vec![entry("目次の項目", "OEBPS/ch2.xhtml", 0)]
        );
    }

    #[test]
    fn falls_back_to_first_nav_without_type() {
        let bytes = epub_with_nav(NAV_WITHOUT_TYPE);
        let book = EpubBook::open(&bytes).unwrap();

        assert_eq!(
            book.toc().unwrap(),
            vec![entry("唯一の nav", "OEBPS/ch1.xhtml", 0)]
        );
    }

    #[test]
    fn treats_multi_token_properties_as_nav() {
        let opf = opf_with_nav_properties("nav scripted");
        let bytes = epub_with_toc(Some(NAV_NESTED), None, &opf);
        let book = EpubBook::open(&bytes).unwrap();

        assert_eq!(book.toc().unwrap().len(), 2);
    }

    #[test]
    fn reads_epub2_ncx_in_document_order() {
        let bytes = epub_with_toc(None, Some(NCX_NESTED), OPF_NCX);
        let book = EpubBook::open(&bytes).unwrap();

        // `playOrder` ではなく文書順に並べ、内容の無い `navPoint` は落とす。
        assert_eq!(
            book.toc().unwrap(),
            vec![
                entry("第一章", "OEBPS/ch1.xhtml", 0),
                // `<content src="ch2.xhtml#s1"/>` のフラグメントを保持する。
                entry_with_fragment("第一節", "OEBPS/ch2.xhtml", "s1", 1),
                entry("第二章", "OEBPS/ch2.xhtml", 0),
            ]
        );
    }

    #[test]
    fn keeps_ncx_fragment() {
        let ncx = ncx_with_sources(&["Text/ch1.xhtml#p1", "ch2.xhtml"]);
        let bytes = epub_with_toc(None, Some(&ncx), OPF_NCX);
        let book = EpubBook::open(&bytes).unwrap();
        let toc = book.toc().unwrap();

        assert_eq!(toc[0].href, "OEBPS/Text/ch1.xhtml");
        assert_eq!(toc[0].fragment.as_deref(), Some("p1"));
        assert_eq!(toc[1].href, "OEBPS/ch2.xhtml");
        assert_eq!(toc[1].fragment, None);
    }

    #[test]
    fn skips_ncx_point_without_path() {
        let ncx = ncx_with_sources(&["#p", "ch1.xhtml"]);
        let bytes = epub_with_toc(None, Some(&ncx), OPF_NCX);
        let book = EpubBook::open(&bytes).unwrap();

        // パス部が空の `navPoint` は読み飛ばし、残りのエントリは残す。
        assert_eq!(
            book.toc().unwrap(),
            vec![entry("項目1", "OEBPS/ch1.xhtml", 0)]
        );
    }

    #[test]
    fn prefers_nav_over_ncx() {
        let opf = opf_with_nav_and_ncx();
        let bytes = epub_with_toc(Some(NAV_NESTED), Some(NCX_NESTED), &opf);
        let book = EpubBook::open(&bytes).unwrap();

        assert_eq!(
            book.toc().unwrap(),
            vec![
                entry("第一章 & はじめに", "OEBPS/ch1.xhtml", 0),
                entry_with_fragment("第二章 1 節", "OEBPS/ch2.xhtml", "s1", 1),
            ]
        );
    }

    #[test]
    fn returns_empty_toc_without_source() {
        // `properties` に `nav` を持たず、NCX も無い OPF。
        let opf = opf_with_nav_properties("cover");
        let bytes = epub_with_toc(Some(NAV_NESTED), None, &opf);
        let book = EpubBook::open(&bytes).unwrap();

        assert!(book.toc().unwrap().is_empty());
    }

    #[test]
    fn returns_empty_toc_when_source_entry_is_missing() {
        // manifest には nav があるが、アーカイブに nav.xhtml が無い。
        let bytes = epub_with_toc(None, None, OPF);
        let book = EpubBook::open(&bytes).unwrap();

        assert!(book.toc().unwrap().is_empty());
    }

    #[test]
    fn rejects_broken_nav_xml() {
        let nav = "<html><body><nav epub:type=\"toc\"><ol><li><a href=\"ch1.xhtml\">x</li></ol></nav></body></html>";
        assert!(matches!(
            toc_error(&epub_with_nav(nav)),
            CoreError::Archive(_)
        ));

        // 途中で切れた nav。
        let truncated = "<html><body><nav epub:type=\"toc\"><ol><li><a href=\"ch1.xhtml\">x</a>";
        assert!(matches!(
            toc_error(&epub_with_nav(truncated)),
            CoreError::Archive(_)
        ));
    }

    #[test]
    fn rejects_broken_ncx_xml() {
        assert!(matches!(
            toc_error(&epub_with_toc(None, Some("<ncx><navMap>"), OPF_NCX)),
            CoreError::Archive(_)
        ));
    }

    #[test]
    fn rejects_nav_href_escaping_root() {
        let nav = "<html><body><nav epub:type=\"toc\"><ol><li><a href=\"../../evil.xhtml\">x</a></li></ol></nav></body></html>";
        assert!(matches!(
            toc_error(&epub_with_nav(nav)),
            CoreError::Validation(_)
        ));

        // 空の `href` はエラーにせず読み飛ばす（契約による意図した更新）。
        // 従来は不正な nav として `CoreError::Archive` にしていた。
        let empty = "<html><body><nav epub:type=\"toc\"><ol><li><a href=\"\">x</a></li></ol></nav></body></html>";
        let book = EpubBook::open(&epub_with_nav(empty)).unwrap();
        assert!(book.toc().unwrap().is_empty());
    }

    #[test]
    fn enforces_toc_entry_limit() {
        // 上限ちょうどは通る。
        let ok = epub_with_nav(&nav_with_anchors(MAX_TOC_ENTRIES));
        let book = EpubBook::open(&ok).unwrap();
        assert_eq!(book.toc().unwrap().len(), MAX_TOC_ENTRIES);

        // 上限を 1 つ超えると Validation。
        let over = epub_with_nav(&nav_with_anchors(MAX_TOC_ENTRIES + 1));
        assert!(matches!(toc_error(&over), CoreError::Validation(_)));
    }

    #[test]
    fn reads_landmarks_and_page_list_from_typed_navs() {
        let nav = r#"<html xmlns:epub="http://www.idpf.org/2007/ops"><body>
          <nav epub:type="toc"><ol><li><a href="ch1.xhtml">Contents</a></li></ol></nav>
          <nav epub:type="landmarks"><ol>
            <li><a epub:type="cover" href="ch1.xhtml#cover">Cover</a></li>
            <li><a epub:type="bodymatter" href="ch2.xhtml">Start</a>
              <ol><li><a epub:type="index" href="ch2.xhtml#idx">Index</a></li></ol>
            </li>
          </ol></nav>
          <nav epub:type="page-list"><ol>
            <li><a href="ch1.xhtml#page1">1</a></li>
            <li><a href="ch2.xhtml#page2">2</a></li>
          </ol></nav>
        </body></html>"#;
        let book = EpubBook::open(&epub_with_nav(nav)).unwrap();

        assert_eq!(
            book.landmarks().unwrap(),
            vec![
                entry_with_fragment("Cover", "OEBPS/ch1.xhtml", "cover", 0),
                entry("Start", "OEBPS/ch2.xhtml", 0),
                entry_with_fragment("Index", "OEBPS/ch2.xhtml", "idx", 1),
            ]
        );
        assert_eq!(
            book.page_list().unwrap(),
            vec![
                entry_with_fragment("1", "OEBPS/ch1.xhtml", "page1", 0),
                entry_with_fragment("2", "OEBPS/ch2.xhtml", "page2", 0),
            ]
        );
        // 通常の目次選択も従来どおり toc 型を優先する。
        assert_eq!(
            book.toc().unwrap(),
            vec![entry("Contents", "OEBPS/ch1.xhtml", 0)]
        );
    }

    #[test]
    fn returns_empty_navigation_lists_when_their_navs_are_missing() {
        let book = EpubBook::open(&epub_with_nav(NAV_NESTED)).unwrap();

        assert!(book.landmarks().unwrap().is_empty());
        assert!(book.page_list().unwrap().is_empty());
    }

    #[test]
    fn rejects_invalid_href_in_landmarks() {
        let nav = r#"<html><body>
          <nav epub:type="landmarks"><ol><li><a href="../../escape">x</a></li></ol></nav>
        </body></html>"#;
        let book = EpubBook::open(&epub_with_nav(nav)).unwrap();

        assert!(matches!(book.landmarks(), Err(CoreError::Validation(_))));
    }

    /// 章と同じディレクトリの下にあるリソースのバイト列。
    const PIC: &[u8] = b"\x89PNG\r\n\x1a\npic";

    /// 章の所属ディレクトリの隣にあるリソースのバイト列。
    const PIC_SIBLING: &[u8] = b"\x89PNG\r\n\x1a\nsibling";

    /// 1 章目のバイト列を差し替えた EPUB を組み立てる。
    fn epub_with_chapter(data: &[u8]) -> Vec<u8> {
        build_epub(&[
            ("META-INF/container.xml", CONTAINER.as_bytes()),
            ("OEBPS/content.opf", OPF.as_bytes()),
            ("OEBPS/ch1.xhtml", data),
        ])
    }

    /// `OEBPS/<name>` にリソースを持つ EPUB を組み立てる。
    fn epub_with_resource(name: &str, data: &[u8]) -> Vec<u8> {
        let path = format!("OEBPS/{name}");
        build_epub(&[
            ("META-INF/container.xml", CONTAINER.as_bytes()),
            ("OEBPS/content.opf", OPF.as_bytes()),
            ("OEBPS/ch1.xhtml", CH1.as_bytes()),
            (path.as_str(), data),
        ])
    }

    /// 章が `OEBPS/text/` にあり、`OEBPS/text/images/pic.png` と
    /// `OEBPS/images/pic.png` を持つ EPUB。
    fn epub_with_subdir_chapter() -> Vec<u8> {
        let opf = OPF.replace("href=\"ch1.xhtml\"", "href=\"text/ch1.xhtml\"");
        build_epub(&[
            ("META-INF/container.xml", CONTAINER.as_bytes()),
            ("OEBPS/content.opf", opf.as_bytes()),
            ("OEBPS/text/ch1.xhtml", CH1.as_bytes()),
            ("OEBPS/text/images/pic.png", PIC),
            ("OEBPS/images/pic.png", PIC_SIBLING),
        ])
    }

    /// 章がルート直下にある EPUB（基準ディレクトリが空の場合の検証用）。
    fn epub_with_root_chapter() -> Vec<u8> {
        let container = CONTAINER.replace("OEBPS/content.opf", "content.opf");
        build_epub(&[
            ("META-INF/container.xml", container.as_bytes()),
            ("content.opf", OPF.as_bytes()),
            ("ch1.xhtml", CH1.as_bytes()),
            ("pic.png", PIC),
        ])
    }

    /// `chapter_resource` のエラーを取り出す。
    fn resource_error(book: &EpubBook, index: usize, href: &str) -> CoreError {
        match book.chapter_resource(index, href) {
            Err(err) => err,
            Ok(data) => panic!("エラーを期待したが {} バイト返った", data.len()),
        }
    }

    #[test]
    fn returns_chapter_html_verbatim() {
        let bytes = sample_epub();
        let book = EpubBook::open(&bytes).unwrap();

        // XML 宣言・タグ・属性をそのまま返す。
        assert_eq!(book.chapter_html(0).unwrap(), CH1);
    }

    #[test]
    fn strips_single_leading_bom_from_chapter_html() {
        let mut with_bom = Vec::from(&b"\xEF\xBB\xBF"[..]);
        with_bom.extend_from_slice(CH1.as_bytes());
        let mut with_two = Vec::from(&b"\xEF\xBB\xBF\xEF\xBB\xBF"[..]);
        with_two.extend_from_slice(CH1.as_bytes());

        let book = EpubBook::open(&epub_with_chapter(&with_bom)).unwrap();
        assert_eq!(book.chapter_html(0).unwrap(), CH1);

        // BOM は 1 個だけ除き、2 個目は本文として残す。
        let book = EpubBook::open(&epub_with_chapter(&with_two)).unwrap();
        assert_eq!(book.chapter_html(0).unwrap(), format!("\u{FEFF}{CH1}"));
    }

    #[test]
    fn rejects_chapter_html_with_invalid_utf8() {
        let bytes = epub_with_chapter(&[0xFF, 0xFE, 0x00]);
        let book = EpubBook::open(&bytes).unwrap();

        assert!(matches!(book.chapter_html(0), Err(CoreError::Archive(_))));
    }

    #[test]
    fn rejects_out_of_range_chapter_html() {
        let bytes = sample_epub();
        let book = EpubBook::open(&bytes).unwrap();

        assert!(matches!(
            book.chapter_html(2),
            Err(CoreError::DocumentNotFound(_))
        ));
    }

    #[test]
    fn reads_resource_next_to_chapter() {
        let bytes = epub_with_resource("pic.png", PIC);
        let book = EpubBook::open(&bytes).unwrap();

        // `OEBPS/ch1.xhtml` から見た `pic.png` は `OEBPS/pic.png`。
        assert_eq!(book.chapter_resource(0, "pic.png").unwrap(), PIC.to_vec());
    }

    #[test]
    fn resolves_resource_in_sub_and_parent_directories() {
        let bytes = epub_with_subdir_chapter();
        let book = EpubBook::open(&bytes).unwrap();

        assert_eq!(book.chapters()[0].href, "OEBPS/text/ch1.xhtml");
        // 所属ディレクトリの下のサブディレクトリ。
        assert_eq!(
            book.chapter_resource(0, "images/pic.png").unwrap(),
            PIC.to_vec()
        );
        // 所属ディレクトリの親を経由した参照。
        assert_eq!(
            book.chapter_resource(0, "../images/pic.png").unwrap(),
            PIC_SIBLING.to_vec()
        );
    }

    #[test]
    fn drops_resource_fragment() {
        let bytes = epub_with_resource("pic.png", PIC);
        let book = EpubBook::open(&bytes).unwrap();

        assert_eq!(
            book.chapter_resource(0, "pic.png#frag").unwrap(),
            PIC.to_vec()
        );
    }

    #[test]
    fn reads_percent_encoded_resource() {
        let bytes = epub_with_resource("a b.png", PIC);
        let book = EpubBook::open(&bytes).unwrap();

        assert_eq!(book.chapter_resource(0, "a%20b.png").unwrap(), PIC.to_vec());
    }

    #[test]
    fn rejects_resource_escaping_root() {
        let bytes = sample_epub();
        let book = EpubBook::open(&bytes).unwrap();

        assert!(matches!(
            resource_error(&book, 0, "../../evil.png"),
            CoreError::Validation(_)
        ));
    }

    #[test]
    fn rejects_resource_with_scheme_or_absolute_path() {
        let bytes = sample_epub();
        let book = EpubBook::open(&bytes).unwrap();

        for href in ["https://example.com/x.png", "/x.png"] {
            assert!(matches!(
                resource_error(&book, 0, href),
                CoreError::Archive(_)
            ));
        }
    }

    #[test]
    fn rejects_resource_without_path() {
        let bytes = sample_epub();
        let book = EpubBook::open(&bytes).unwrap();

        for href in ["", "   ", "#frag"] {
            assert!(matches!(
                resource_error(&book, 0, href),
                CoreError::Archive(_)
            ));
        }
    }

    #[test]
    fn rejects_missing_resource() {
        let bytes = sample_epub();
        let book = EpubBook::open(&bytes).unwrap();

        assert!(matches!(
            resource_error(&book, 0, "missing.png"),
            CoreError::DocumentNotFound(_)
        ));
    }

    #[test]
    fn rejects_out_of_range_chapter_resource() {
        let bytes = sample_epub();
        let book = EpubBook::open(&bytes).unwrap();

        assert!(matches!(
            resource_error(&book, 2, "pic.png"),
            CoreError::DocumentNotFound(_)
        ));
    }

    #[test]
    fn resolves_resource_for_chapter_at_root() {
        let bytes = epub_with_root_chapter();
        let book = EpubBook::open(&bytes).unwrap();

        assert_eq!(book.chapters()[0].href, "ch1.xhtml");
        assert_eq!(book.chapter_resource(0, "pic.png").unwrap(), PIC.to_vec());
        // ルート直下の章からの親参照はルートを越える。
        assert!(matches!(
            resource_error(&book, 0, "../evil.png"),
            CoreError::Validation(_)
        ));
    }

    #[test]
    fn rejects_encoded_or_backslash_resource_traversal() {
        let bytes = sample_epub();
        let book = EpubBook::open(&bytes).unwrap();

        // バックスラッシュ区切りの親参照は読み出しの検証で拒否する。
        assert!(matches!(
            resource_error(&book, 0, "..\\evil.png"),
            CoreError::Validation(_)
        ));
        // `%XX` 復号後の `..` も読めない（復号側の拒否は最初のエラーで報告する）。
        assert!(matches!(
            resource_error(&book, 0, "%2e%2e%2fevil.png"),
            CoreError::DocumentNotFound(_)
        ));
    }

    /// 表紙画像のバイト列。
    const COVER: &[u8] = b"\x89PNG\r\n\x1a\ncover";

    /// 表紙を EPUB3 の `properties="cover-image"` で宣言する OPF。
    ///
    /// EPUB2 の `<meta name="cover">` も併記し、EPUB3 の宣言を優先することを
    /// 確認できるようにする。
    const OPF_COVER3: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="pub-id">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:title>サンプル本</dc:title>
    <dc:identifier id="pub-id">urn:isbn:9784000000000</dc:identifier>
    <meta name="cover" content="meta-cover"/>
  </metadata>
  <manifest>
    <item id="ch1" href="ch1.xhtml" media-type="application/xhtml+xml"/>
    <item id="ch2" href="ch2.xhtml" media-type="application/xhtml+xml"/>
    <item id="meta-cover" href="meta.png" media-type="image/png"/>
    <item id="prop-cover" href="images/cover.png" media-type="image/png" properties="cover-image"/>
  </manifest>
  <spine>
    <itemref idref="ch1"/>
    <itemref idref="ch2"/>
  </spine>
</package>
"#;

    /// 表紙を EPUB2 の `<meta name="cover">` で宣言する OPF。
    const OPF_COVER2: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="2.0" unique-identifier="pub-id">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:title>サンプル本</dc:title>
    <dc:identifier id="pub-id">urn:isbn:9784000000000</dc:identifier>
    <meta name="cover" content="cover-img"/>
  </metadata>
  <manifest>
    <item id="ch1" href="ch1.xhtml" media-type="application/xhtml+xml"/>
    <item id="ch2" href="ch2.xhtml" media-type="application/xhtml+xml"/>
    <item id="cover-img" href="cover.jpg" media-type="image/jpeg"/>
  </manifest>
  <spine>
    <itemref idref="ch1"/>
    <itemref idref="ch2"/>
  </spine>
</package>
"#;

    /// 表紙が画像ではない item を指す OPF。
    const OPF_COVER_PAGE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="2.0" unique-identifier="pub-id">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:title>サンプル本</dc:title>
    <dc:identifier id="pub-id">urn:isbn:9784000000000</dc:identifier>
    <meta name="cover" content="cover-page"/>
  </metadata>
  <manifest>
    <item id="ch1" href="ch1.xhtml" media-type="application/xhtml+xml"/>
    <item id="ch2" href="ch2.xhtml" media-type="application/xhtml+xml"/>
    <item id="cover-page" href="cover.xhtml" media-type="application/xhtml+xml"/>
  </manifest>
  <spine>
    <itemref idref="ch1"/>
    <itemref idref="ch2"/>
  </spine>
</package>
"#;

    /// 表紙の `href` がパーセントエンコードされた OPF。
    const OPF_COVER_ENCODED: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="2.0" unique-identifier="pub-id">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:title>サンプル本</dc:title>
    <dc:identifier id="pub-id">urn:isbn:9784000000000</dc:identifier>
    <meta name="cover" content="cover-img"/>
  </metadata>
  <manifest>
    <item id="ch1" href="ch1.xhtml" media-type="application/xhtml+xml"/>
    <item id="ch2" href="ch2.xhtml" media-type="application/xhtml+xml"/>
    <item id="cover-img" href="cover%20img.png" media-type="image/png"/>
  </manifest>
  <spine>
    <itemref idref="ch1"/>
    <itemref idref="ch2"/>
  </spine>
</package>
"#;

    /// OPF と追加エントリから EPUB を組み立てる（章は標準の 2 つ）。
    fn epub_with_opf_and_files(opf: &str, extra: &[(&str, &[u8])]) -> Vec<u8> {
        let mut files: Vec<(&str, &[u8])> = vec![
            ("META-INF/container.xml", CONTAINER.as_bytes()),
            ("OEBPS/content.opf", opf.as_bytes()),
            ("OEBPS/ch1.xhtml", CH1.as_bytes()),
            ("OEBPS/ch2.xhtml", CH2.as_bytes()),
        ];
        files.extend_from_slice(extra);
        build_epub(&files)
    }

    /// 非対応の圧縮方式として扱われるコード（既知の方式と重ならない値）。
    const UNSUPPORTED_COMPRESSION: u16 = 99;

    /// 指定エントリの圧縮方式だけを書き換えた ZIP を返す。
    ///
    /// 一覧の走査は通り、そのエントリの読み出しだけが失敗するアーカイブを作る
    /// ために使う。ローカルファイルヘッダーとセントラルディレクトリの両方を
    /// 書き換えないと整合しないため、両方を探して同じ値にする。
    fn patch_compression_method(zip: &[u8], name: &str, method: u16) -> Vec<u8> {
        let mut bytes = zip.to_vec();
        let name = name.as_bytes();
        for (signature, name_offset, method_offset) in
            [(b"PK\x03\x04", 30, 8), (b"PK\x01\x02", 46, 10)]
        {
            let at = find_compression_method(&bytes, signature, name_offset, method_offset, name);
            bytes[at..at + 2].copy_from_slice(&method.to_le_bytes());
        }
        bytes
    }

    /// `name` を持つヘッダーの圧縮方式の位置を返す（テスト専用）。
    ///
    /// `name_offset` は署名の先頭からファイル名まで、`method_offset` は署名の
    /// 先頭から圧縮方式までの距離。見つからない場合はテストの前提が崩れて
    /// いるため panic する。
    fn find_compression_method(
        zip: &[u8],
        signature: &[u8; 4],
        name_offset: usize,
        method_offset: usize,
        name: &[u8],
    ) -> usize {
        for start in 0..zip.len().saturating_sub(4) {
            if &zip[start..start + 4] != signature {
                continue;
            }
            let name_start = start + name_offset;
            if zip.get(name_start..name_start + name.len()) == Some(name) {
                let at = start + method_offset;
                assert!(at + 2 <= zip.len(), "ヘッダーが途中で切れています");
                return at;
            }
        }
        panic!("ZIP のヘッダーが見つかりません");
    }

    /// 1 章目を差し替えた EPUB を組み立てる（目次と `<title>` の検証用）。
    fn epub_with_chapter_and_opf(chapter: &str, opf: &str) -> Vec<u8> {
        build_epub(&[
            ("META-INF/container.xml", CONTAINER.as_bytes()),
            ("OEBPS/content.opf", opf.as_bytes()),
            ("OEBPS/ch1.xhtml", chapter.as_bytes()),
            ("OEBPS/ch2.xhtml", CH2.as_bytes()),
        ])
    }

    /// 目次の情報源を持たない OPF を返す。
    fn opf_without_toc() -> String {
        opf_with_nav_properties("cover")
    }

    #[test]
    fn reads_cover_image_from_epub3_properties() {
        let bytes = epub_with_opf_and_files(OPF_COVER3, &[("OEBPS/images/cover.png", COVER)]);
        let book = EpubBook::open(&bytes).unwrap();
        let cover = book.cover_image().unwrap().unwrap();

        // EPUB3 の `properties` を優先する（`meta` は別の item を指している）。
        assert_eq!(cover.id, "prop-cover");
        assert_eq!(cover.href, "OEBPS/images/cover.png");
        assert_eq!(cover.media_type, "image/png");
        assert_eq!(cover.bytes, COVER.to_vec());
    }

    #[test]
    fn reads_cover_image_from_epub2_meta() {
        let bytes = epub_with_opf_and_files(OPF_COVER2, &[("OEBPS/cover.jpg", COVER)]);
        let book = EpubBook::open(&bytes).unwrap();
        let cover = book.cover_image().unwrap().unwrap();

        assert_eq!(cover.id, "cover-img");
        assert_eq!(cover.href, "OEBPS/cover.jpg");
        assert_eq!(cover.media_type, "image/jpeg");
        assert_eq!(cover.bytes, COVER.to_vec());
    }

    #[test]
    fn returns_none_without_cover_declaration() {
        // `properties="nav"` を差し替えた OPF は表紙を宣言していない。
        let bytes = epub_with_toc(None, None, &opf_without_toc());
        let book = EpubBook::open(&bytes).unwrap();

        assert!(book.cover_image().unwrap().is_none());
    }

    #[test]
    fn returns_none_when_cover_id_is_unknown() {
        // 標準の OPF は `cover-image` を指すが、その id の item が無い。
        let bytes = sample_epub();
        let book = EpubBook::open(&bytes).unwrap();

        assert!(book.cover_image().unwrap().is_none());
    }

    #[test]
    fn returns_none_when_cover_entry_is_missing() {
        // 宣言はあるがアーカイブにエントリが無い場合は表紙なしとする。
        let bytes = epub_with_opf_and_files(OPF_COVER2, &[]);
        let book = EpubBook::open(&bytes).unwrap();

        assert!(book.cover_image().unwrap().is_none());
    }

    #[test]
    fn ignores_cover_item_that_is_not_an_image() {
        let bytes = epub_with_opf_and_files(
            OPF_COVER_PAGE,
            &[("OEBPS/cover.xhtml", b"<html><body>cover</body></html>")],
        );
        let book = EpubBook::open(&bytes).unwrap();

        // 表紙ページ（XHTML）を指す `id` は画像ではないため表紙にしない。
        assert!(book.cover_image().unwrap().is_none());
    }

    #[test]
    fn reads_percent_encoded_cover_href() {
        let bytes = epub_with_opf_and_files(OPF_COVER_ENCODED, &[("OEBPS/cover img.png", COVER)]);
        let book = EpubBook::open(&bytes).unwrap();
        let cover = book.cover_image().unwrap().unwrap();

        // `href` は元の綴りのまま、バイト列は復号した名前で読む。
        assert_eq!(cover.href, "OEBPS/cover%20img.png");
        assert_eq!(cover.bytes, COVER.to_vec());
    }

    #[test]
    fn reads_chapter_title_from_toc() {
        let bytes = epub_with_nav(NAV_NESTED);
        let book = EpubBook::open(&bytes).unwrap();

        // 目次の表示名を章と `href` で対応付けて返す（章の `<title>` より優先）。
        assert_eq!(
            book.chapter_title(0).unwrap().as_deref(),
            Some("第一章 & はじめに")
        );
        assert_eq!(
            book.chapter_title(1).unwrap().as_deref(),
            Some("第二章 1 節")
        );
    }

    #[test]
    fn falls_back_to_chapter_document_title() {
        // 目次の情報源を持たない EPUB では章の `<title>` を使う。
        let bytes = epub_with_toc(None, None, &opf_without_toc());
        let book = EpubBook::open(&bytes).unwrap();

        assert_eq!(
            book.chapter_title(0).unwrap().as_deref(),
            Some("Head Title")
        );
        // `<title>` を持たない章は `None`。
        assert_eq!(book.chapter_title(1).unwrap(), None);
    }

    #[test]
    fn normalizes_chapter_document_title() {
        let chapter = "<html><head><title>  第一章 &amp; 序 &#x3042;  </title></head>\
                       <body><p>x</p></body></html>";
        let bytes = epub_with_chapter_and_opf(chapter, &opf_without_toc());
        let book = EpubBook::open(&bytes).unwrap();

        // 実体参照を戻し、連続する空白をまとめて前後を落とす。
        assert_eq!(
            book.chapter_title(0).unwrap().as_deref(),
            Some("第一章 & 序 あ")
        );
    }

    #[test]
    fn treats_empty_document_title_as_none() {
        let chapter = "<html><head><title/></head><body><p>x</p></body></html>";
        let bytes = epub_with_chapter_and_opf(chapter, &opf_without_toc());
        let book = EpubBook::open(&bytes).unwrap();

        assert_eq!(book.chapter_title(0).unwrap(), None);
    }

    #[test]
    fn rejects_out_of_range_chapter_title() {
        let bytes = sample_epub();
        let book = EpubBook::open(&bytes).unwrap();

        assert!(matches!(
            book.chapter_title(9),
            Err(CoreError::DocumentNotFound(_))
        ));
    }

    #[test]
    fn rejects_chapter_title_when_document_is_broken() {
        // 目次が無く、`<title>` も持たないまま要素が閉じられていない章。
        let chapter = "<html><body><p>x</body></html>";
        let bytes = epub_with_chapter_and_opf(chapter, &opf_without_toc());
        let book = EpubBook::open(&bytes).unwrap();

        assert!(matches!(book.chapter_title(0), Err(CoreError::Archive(_))));
    }

    #[test]
    fn ignores_titles_outside_the_document_head() {
        // `<head>` に `<title>` を持たず、本文のインライン SVG にだけ持つ章。
        let chapter = "<html><head></head><body><svg><title>図</title></svg><p>x</p></body></html>";
        let bytes = epub_with_chapter_and_opf(chapter, &opf_without_toc());
        let book = EpubBook::open(&bytes).unwrap();

        assert_eq!(book.chapter_title(0).unwrap(), None);
    }

    #[test]
    fn prefers_head_title_over_inline_svg_title() {
        let chapter = "<html><head><title>章の名前</title></head>\
                       <body><svg><title>図</title></svg><p>x</p></body></html>";
        let bytes = epub_with_chapter_and_opf(chapter, &opf_without_toc());
        let book = EpubBook::open(&bytes).unwrap();

        assert_eq!(book.chapter_title(0).unwrap().as_deref(), Some("章の名前"));
    }

    #[test]
    fn reports_cover_errors_other_than_a_missing_entry() {
        // 表紙エントリの圧縮方式だけを非対応の値に書き換えた EPUB を組み立てる。
        let bytes = epub_with_opf_and_files(OPF_COVER2, &[("OEBPS/cover.jpg", COVER)]);
        let broken = patch_compression_method(&bytes, "OEBPS/cover.jpg", UNSUPPORTED_COMPRESSION);
        let book = EpubBook::open(&broken).unwrap();

        // エントリの欠落は表紙なしにするが、読み出しの失敗は報告する。
        assert!(matches!(book.cover_image(), Err(CoreError::Archive(_))));
    }

    #[test]
    fn resolves_links_to_another_chapter() {
        let book = EpubBook::open(&sample_epub()).unwrap();

        assert_eq!(
            book.resolve_chapter_link(0, "ch2.xhtml").unwrap(),
            Some(ChapterLink {
                chapter_index: 1,
                fragment: None,
            })
        );
        // 同じ章を指すリンクも解決する。
        assert_eq!(
            book.resolve_chapter_link(0, "ch1.xhtml").unwrap(),
            Some(ChapterLink {
                chapter_index: 0,
                fragment: None,
            })
        );
    }

    #[test]
    fn resolves_links_with_fragments() {
        let book = EpubBook::open(&sample_epub()).unwrap();

        assert_eq!(
            book.resolve_chapter_link(0, "ch2.xhtml#part").unwrap(),
            Some(ChapterLink {
                chapter_index: 1,
                fragment: Some(String::from("part")),
            })
        );
        // フラグメントだけのリンクは同じ章を指す。
        assert_eq!(
            book.resolve_chapter_link(0, " #part ").unwrap(),
            Some(ChapterLink {
                chapter_index: 0,
                fragment: Some(String::from("part")),
            })
        );
    }

    #[test]
    fn resolves_parent_directory_links_from_a_subdirectory_chapter() {
        let opf = OPF
            .replace("href=\"ch1.xhtml\"", "href=\"text/ch1.xhtml\"")
            .replace("href=\"ch2.xhtml\"", "href=\"text/ch2.xhtml\"");
        let bytes = build_epub(&[
            ("META-INF/container.xml", CONTAINER.as_bytes()),
            ("OEBPS/content.opf", opf.as_bytes()),
            ("OEBPS/text/ch1.xhtml", CH1.as_bytes()),
            ("OEBPS/text/ch2.xhtml", CH2.as_bytes()),
        ]);
        let book = EpubBook::open(&bytes).unwrap();
        assert_eq!(book.chapters()[1].href, "OEBPS/text/ch2.xhtml");

        assert_eq!(
            book.resolve_chapter_link(0, "../text/ch2.xhtml").unwrap(),
            Some(ChapterLink {
                chapter_index: 1,
                fragment: None,
            })
        );
    }

    #[test]
    fn resolves_percent_encoded_chapter_links() {
        // 章の実体は空白入りの名前で、リンクは `%20` で書かれている。
        let opf = OPF.replace("href=\"ch2.xhtml\"", "href=\"ch 2.xhtml\"");
        let bytes = build_epub(&[
            ("META-INF/container.xml", CONTAINER.as_bytes()),
            ("OEBPS/content.opf", opf.as_bytes()),
            ("OEBPS/ch1.xhtml", CH1.as_bytes()),
            ("OEBPS/ch 2.xhtml", CH2.as_bytes()),
        ]);
        let book = EpubBook::open(&bytes).unwrap();
        assert_eq!(book.chapters()[1].href, "OEBPS/ch 2.xhtml");

        assert_eq!(
            book.resolve_chapter_link(0, "ch%202.xhtml").unwrap(),
            Some(ChapterLink {
                chapter_index: 1,
                fragment: None,
            })
        );
    }

    #[test]
    fn rejects_links_outside_the_book() {
        let book = EpubBook::open(&sample_epub()).unwrap();

        for href in [
            "",
            "#",
            "https://example.com/ch2.xhtml",
            "/ch2.xhtml",
            "../../evil.xhtml",
            "other.xhtml",
            "ch2.xhtml?query",
        ] {
            assert_eq!(
                book.resolve_chapter_link(0, href).unwrap(),
                None,
                "unexpected link: {href:?}"
            );
        }
    }

    #[test]
    fn reports_out_of_range_index_for_chapter_links() {
        let book = EpubBook::open(&sample_epub()).unwrap();

        assert!(matches!(
            book.resolve_chapter_link(9, "ch2.xhtml"),
            Err(CoreError::DocumentNotFound(_))
        ));
    }

    #[test]
    fn malformed_chapter_xhtml_is_passed_through() {
        // 章本文は XML 検証せずそのまま返す（表示側のサニタイズに委ねる）。
        let malformed = "<p>unclosed";
        let bytes = epub_with_chapter(malformed.as_bytes());
        let book = EpubBook::open(&bytes).unwrap();

        assert_eq!(book.chapter_html(0).unwrap(), malformed);
    }

    #[test]
    fn malformed_opf_is_rejected() {
        // OPF は XML として解析し、構文エラーは fail-closed で拒否する。
        let bytes = epub_with_chapter_and_opf(CH1, "<package><metadata>");
        assert!(EpubBook::open(&bytes).is_err());
    }
}
