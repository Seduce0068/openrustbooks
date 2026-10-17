//! EPUB の本文読み出し。
//!
//! [`ArchiveReader`] の上に、`META-INF/container.xml` → OPF（メタデータ・
//! manifest・spine）→ 章本文のテキスト抽出までを載せる。ファイル I/O に
//! 依存せず、ネイティブと `wasm32-unknown-unknown` の両方でビルドできる。
//!
//! `docs/requirements.md` の REQ-002（電子書籍の閲覧）のうち、本文と章の
//! 読み出しだけを提供する。目次（nav / NCX）の解析、章タイトルの取得、
//! 画像の扱い、`Document` 型との統合は範囲外。読み取り専用であり、EPUB の
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
        let archive = ArchiveReader::with_limits(bytes, limits)?;
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
        let href = self.chapter(index)?.href.clone();
        let data = self.chapter_bytes(index)?;
        let decoded = String::from_utf8(data).map_err(|err| {
            CoreError::archive(format!("章を UTF-8 として復号できません: {href} ({err})"))
        })?;
        let body = match decoded.strip_prefix('\u{FEFF}') {
            Some(stripped) => stripped.to_string(),
            None => decoded,
        };
        extract_text(&body, &href)
    }

    /// 範囲を検証して章を取り出す。
    fn chapter(&self, index: usize) -> CoreResult<&EpubChapter> {
        self.chapters
            .get(index)
            .ok_or_else(|| CoreError::DocumentNotFound(format!("章の index が範囲外です: {index}")))
    }

    /// `href` の `%XX` を復号した名前で読めたらその内容を返す。
    fn read_decoded_entry(&self, href: &str) -> Option<Vec<u8>> {
        let decoded = percent_decode(href)?;
        if decoded == href {
            return None;
        }
        self.archive.read_entry(&decoded).ok()
    }
}

/// OPF の解析結果。
struct ParsedOpf {
    /// 書誌メタデータ。
    metadata: EpubMetadata,
    /// spine の順の章。
    chapters: Vec<EpubChapter>,
}

/// manifest の `item`。
struct ManifestItem {
    /// manifest の `id`。
    id: String,
    /// OPF の位置を基準に解決済みのアーカイブ内パス。
    href: String,
    /// `media-type` の値（無い場合は空文字列）。
    media_type: String,
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

/// OPF のパスから、相対 `href` の基準となるディレクトリを返す。
fn opf_dir(opf_path: &str) -> &str {
    match opf_path.rfind('/') {
        Some(index) => &opf_path[..index],
        None => "",
    }
}

/// `href` を OPF のディレクトリ基準でアーカイブ内パスへ解決する。
///
/// フラグメント (`#...`) を落とし、`.` と `..` を字句的に解決する。スキーム付き
/// URL と先頭 `/` は不正な OPF として [`CoreError::Archive`]、ルートを越える
/// 解決は [`CoreError::Validation`] を返す。
fn resolve_href(opf_dir: &str, href: &str) -> CoreResult<String> {
    let target = href.split('#').next().unwrap_or_default().trim();
    if target.is_empty() {
        return Err(CoreError::archive("manifest の href が空です"));
    }
    if target.starts_with('/') {
        return Err(CoreError::archive(format!(
            "manifest の href が絶対パスです: {href:?}"
        )));
    }
    if has_scheme(target) {
        return Err(CoreError::archive(format!(
            "manifest の href がスキーム付き URL です: {href:?}"
        )));
    }

    let mut components: Vec<&str> = Vec::new();
    for component in opf_dir.split('/').filter(|part| !part.is_empty()) {
        components.push(component);
    }
    for component in target.split('/') {
        match component {
            "" | "." => {}
            ".." => {
                if components.pop().is_none() {
                    return Err(CoreError::validation(format!(
                        "manifest の href がルートを越えて解決されます: {href:?}"
                    )));
                }
            }
            other => components.push(other),
        }
    }

    if components.is_empty() {
        return Err(CoreError::archive(format!(
            "manifest の href が空のパスに解決されます: {href:?}"
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
    let base_dir = opf_dir(opf_path);

    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);

    let mut version: Option<String> = None;
    let mut unique_identifier: Option<String> = None;
    let mut captures: Vec<Capture> = Vec::new();
    let mut collected: Vec<CollectedField> = Vec::new();
    let mut manifest: Vec<ManifestItem> = Vec::new();
    let mut manifest_index: HashMap<String, usize> = HashMap::new();
    let mut spine: Vec<SpineRef> = Vec::new();
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

    Ok(ParsedOpf { metadata, chapters })
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
    let resolved = resolve_href(base_dir, &href)?;

    index.entry(id.clone()).or_insert(manifest.len());
    manifest.push(ManifestItem {
        id,
        href: resolved,
        media_type,
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
}
