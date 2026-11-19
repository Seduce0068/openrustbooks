//! XML 属性・相対パスの共通 helper。

use quick_xml::XmlVersion;
use quick_xml::events::BytesStart;

use crate::{CoreError, CoreResult};

/// 生の `href` / `src` を「最初の `#` の前」と「後ろ」に分ける。
///
/// パス部は前後の空白を除いたもの、フラグメントは `#` を除いて前後の空白を
/// 除いたもの（空なら `None`）を返す。2 つ目以降の `#` は区切りにせず、
/// `%XX` はそのまま残す。属性の実体参照は復号済みの文字列を渡す。
pub(super) fn split_fragment(raw: &str) -> (&str, Option<String>) {
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
pub(super) fn resolve_href(opf_dir: &str, href: &str) -> CoreResult<String> {
    resolve_reference(opf_dir, href, "manifest の href")
}

/// 相対参照 `href` を `base_dir` 基準でアーカイブ内パスへ解決する。
///
/// 最初の `#` 以降（フラグメント）を落として前後の空白を除き、`.` と `..` を
/// 字句的に解決する。空のパス・スキーム付き URL・先頭 `/` は対象の文書が
/// 不正であるとして [`CoreError::Archive`]、ルートを越える解決は
/// [`CoreError::Validation`] を返す。`subject` はエラーメッセージの主語
/// （例: `manifest の href`）に使う。
pub(super) fn resolve_reference(base_dir: &str, href: &str, subject: &str) -> CoreResult<String> {
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
pub(super) fn has_scheme(value: &str) -> bool {
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
pub(super) fn attribute_value(element: &BytesStart<'_>, name: &str) -> CoreResult<Option<String>> {
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
pub(super) fn unescape_attribute(
    attr: &quick_xml::events::attributes::Attribute<'_>,
    name: &str,
) -> CoreResult<String> {
    attr.normalized_value(XmlVersion::Implicit1_0)
        .map(|value| value.into_owned())
        .map_err(|err| CoreError::archive(format!("属性 {name} を復号できません: {err}")))
}
