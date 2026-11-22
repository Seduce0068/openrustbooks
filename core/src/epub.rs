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

use self::navigation::{collect_nav_entries, parse_nav, parse_ncx, select_nav_type};
use self::opf::{
    ManifestItem, has_property, parse_container, parse_opf, path_dir, read_text_entry,
};
use self::text::{extract_text, extract_title, normalized_chapter_path};
// web 側の同等 helper と cross-check できるよう、規則の权威だけを公開する。
pub use self::xmlutil::{has_scheme, percent_decode};
use self::xmlutil::{resolve_reference, split_fragment};

use crate::archive::{ArchiveLimits, ArchiveReader};
use crate::{CoreError, CoreResult};

mod navigation;
mod opf;
mod text;
mod xmlutil;

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
/// OPF / NCX / 目次の構造要素数の上限。
///
/// 目次・manifest・spine・メタデータ・navPoint で 1 つの値を共有し、
/// 64 MiB の文書から数百万要素を確保できないようにする。
const MAX_STRUCTURE_ENTRIES: usize = 10_000;

/// 構造要素の追加前に上限を確認する。
///
/// `subject` は要素名だけを含み、`id` や `href` などの値は渡さない。
/// 超過時は `subject` と上限値だけを含む [`CoreError::Validation`] を返す
/// （ADR-0212 の redaction 契約）。
pub(super) fn check_structure_limit(count: usize, subject: &str) -> CoreResult<()> {
    if count >= MAX_STRUCTURE_ENTRIES {
        return Err(CoreError::validation(format!(
            "{subject}が上限を超えています: {MAX_STRUCTURE_ENTRIES}"
        )));
    }
    Ok(())
}

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
    /// `href` の最初の `#` の後ろ（`#` を除き、前後の空白を除いたもの）。
    /// `%XX` は [`split_fragment`] で復号済み。`#` が無い・空なら `None`。
    /// 不正な `%XX` は [`CoreError::Validation`] として拒否される。
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
            // `%XX` の復号そのものが失敗した場合は Validation を伝播する。
            Err(first) => self.read_decoded_entry(&href)?.ok_or(first),
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
            Err(first) => self.read_decoded_entry(&path)?.ok_or(first),
        }
    }

    /// 章 `index` の本文にある相対リンク `href` を、同じ本の章へ解決する。
    ///
    /// `#fragment` だけのリンクは同じ章を返す。スキーム付き URL・先頭 `/`・
    /// 空のリンク・spine に無いパス・ルートを越える参照は `None` を返し、
    /// 外部や本の外へは移動しない（fail-closed）。範囲外の `index` は
    /// [`CoreError::DocumentNotFound`]。`%XX` は復号して比較するが、リテラル
    /// `%` を含む実文件名は生のまま読める。`fragment` は [`split_fragment`] で
    /// 復号されて返るため、未復号の文字列を位置指定子として使わない。不正な
    /// `%XX` は [`CoreError::Validation`] を返す。
    pub fn resolve_chapter_link(
        &self,
        index: usize,
        href: &str,
    ) -> CoreResult<Option<ChapterLink>> {
        let chapter_href = self.chapter(index)?.href.clone();
        let (path, fragment) = split_fragment(href)?;
        if path.is_empty() {
            return Ok(fragment.map(|fragment| ChapterLink {
                chapter_index: index,
                fragment: Some(fragment),
            }));
        }
        if path.starts_with('/') || has_scheme(path) {
            return Ok(None);
        }
        // 解決失敗のうち空・絶対・スキーム・ルート越えは既存どおり None に落とす。
        // リテラル `%` を含む実文件名を生名で読めるよう、ここでは復号検証しない。
        let Ok(target) = resolve_reference(path_dir(&chapter_href), path, "章リンクの href")
        else {
            return Ok(None);
        };
        let Some(chapter_index) = self.chapter_index_for_path(&target)? else {
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
    /// 復号に失敗した場合は [`CoreError::Validation`] を返す。
    fn chapter_index_for_path(&self, target: &str) -> CoreResult<Option<usize>> {
        // リテラル `%` を含む実文件名は、復号せず生名で先に照合する。
        if let Some(index) = self
            .chapters
            .iter()
            .position(|chapter| chapter.href == target)
        {
            return Ok(Some(index));
        }
        let target = normalized_chapter_path(target)?;
        for (index, chapter) in self.chapters.iter().enumerate() {
            if normalized_chapter_path(&chapter.href)? == target {
                return Ok(Some(index));
            }
        }
        Ok(None)
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
            // 復号そのものの失敗は Validation として報告する。
            Err(CoreError::DocumentNotFound(_)) => match self.read_decoded_entry(&item.href)? {
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
    ///
    /// 復号そのものが失敗した場合は [`CoreError::Validation`] を返す。復号後の
    /// パスが存在しない等の読み出し失敗は、呼び出し側が最初のエラーを報告できる
    /// ように `None` を返す（既存の `%XX` フォールバックの契約を維持する）。
    fn read_decoded_entry(&self, href: &str) -> CoreResult<Option<Vec<u8>>> {
        let decoded = percent_decode(href, "アーカイブのパス")?;
        if decoded == href {
            return Ok(None);
        }
        Ok(self.archive.read_entry(&decoded).ok())
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

/// 表紙に使える画像の media-type かどうか。
///
/// `image/` 以外と明示されている item は表紙にしない（表紙ページの XHTML を
/// 指す `id` を返さないため）。未宣言（空文字列）は判定材料が無いため許容する。
fn is_image_media_type(media_type: &str) -> bool {
    let value = media_type.trim();
    value.is_empty() || value.to_ascii_lowercase().starts_with("image/")
}

#[cfg(test)]
#[rustfmt::skip]
#[path = "epub/tests.rs"]
mod tests;
