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
/// リモート資源を宣言する `properties` のトークン（EPUB3）。
const REMOTE_RESOURCES_PROPERTY: &str = "remote-resources";
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
        Self::open_owned_with_limits(bytes, ArchiveLimits::default())
    }

    /// 所有権を持つバイト列とリミットを指定して EPUB を開く。
    ///
    /// `mimetype` を持たないアーカイブは [`CoreError::UnsupportedFormat`]、
    /// コンテナ文書や OPF の読み出し・解析の失敗は [`CoreError::Archive`]、
    /// パス逸脱やリミット超過は [`CoreError::Validation`] を返す。
    pub fn open_owned_with_limits(bytes: Vec<u8>, limits: ArchiveLimits) -> CoreResult<Self> {
        Self::from_archive(ArchiveReader::from_owned(bytes, limits)?)
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

    /// manifest のいずれかの item が `remote-resources` を宣言しているか。
    pub fn has_remote_resources(&self) -> bool {
        self.manifest
            .iter()
            .any(|item| has_property(&item.properties, REMOTE_RESOURCES_PROPERTY))
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
        return Err(CoreError::archive("manifest の item に id がありません"));
    };
    let Some(href) = attribute_value(element, "href")? else {
        return Err(CoreError::archive("manifest の item に href がありません"));
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
                    "text" if in_label && text.is_none() => {
                        text = Some(String::new());
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
#[rustfmt::skip]
#[path = "epub/tests.rs"]
mod tests;
