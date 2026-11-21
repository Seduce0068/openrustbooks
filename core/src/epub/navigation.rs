//! EPUB3 nav / EPUB2 NCX の目次解析。

use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

use crate::{CoreError, CoreResult};

use super::text::{named_reference, normalize_text};
use super::xmlutil::{attribute_value, resolve_href, split_fragment, unescape_attribute};
use super::{EpubTocEntry, MAX_TOC_ENTRIES, NAV_ELEMENT, TOC_TYPE};

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
pub(super) fn parse_nav(xml: &str, base_dir: &str, label: &str) -> CoreResult<Vec<EpubTocEntry>> {
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
pub(super) fn select_nav_type(
    xml: &str,
    label: &str,
    navigation_type: &str,
) -> CoreResult<Option<usize>> {
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
pub(super) fn collect_nav_entries(
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
                            let (path, fragment) = split_fragment(&raw)?;
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
pub(super) fn parse_ncx(xml: &str, base_dir: &str, label: &str) -> CoreResult<Vec<EpubTocEntry>> {
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
        let (path, fragment) = split_fragment(&raw)?;
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
