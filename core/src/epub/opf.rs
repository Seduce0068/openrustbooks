//! container.xml / OPF の解析。

use std::collections::HashMap;

use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

use crate::archive::ArchiveReader;
use crate::{CoreError, CoreResult};

use super::text::{named_reference, normalize_text};
use super::xmlutil::{attribute_value, resolve_href, unescape_attribute};
use super::{
    CONTAINER_PATH, COVER_META_NAME, EpubChapter, EpubMetadata, FULL_PATH_ATTRIBUTE,
    MODIFIED_PROPERTY, ROOTFILE_ELEMENT,
};

/// OPF の解析結果。
pub(super) struct ParsedOpf {
    /// 書誌メタデータ。
    pub(super) metadata: EpubMetadata,
    /// spine の順の章。
    pub(super) chapters: Vec<EpubChapter>,
    /// manifest の `item`（文書順）。
    pub(super) manifest: Vec<ManifestItem>,
    /// OPF の位置（相対 `href` の基準）。
    pub(super) base_dir: String,
    /// `<meta name="cover">` が指す表紙画像の `id`。
    pub(super) cover_id: Option<String>,
}

/// manifest の `item`。
pub(super) struct ManifestItem {
    /// manifest の `id`。
    pub(super) id: String,
    /// OPF の位置を基準に解決済みのアーカイブ内パス。
    pub(super) href: String,
    /// `media-type` の値（無い場合は空文字列）。
    pub(super) media_type: String,
    /// `properties` の値（空白区切りのトークン列。無い場合は空文字列）。
    pub(super) properties: String,
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
pub(super) fn read_text_entry(archive: &ArchiveReader, path: &str) -> CoreResult<String> {
    archive
        .read_entry_text(path)
        .map_err(|err| CoreError::archive(format!("{path} を読み出せません: {err}")))
}

/// コンテナ文書から最初の `rootfile` の `full-path` を取り出す。
pub(super) fn parse_container(xml: &str) -> CoreResult<String> {
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
pub(super) fn path_dir(path: &str) -> &str {
    match path.rfind('/') {
        Some(index) => &path[..index],
        None => "",
    }
}

/// OPF を解析してメタデータと章一覧を作る。
pub(super) fn parse_opf(xml: &str, opf_path: &str) -> CoreResult<ParsedOpf> {
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
pub(super) fn has_property(properties: &str, token: &str) -> bool {
    properties.split_whitespace().any(|value| value == token)
}
