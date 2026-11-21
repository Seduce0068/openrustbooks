//! XML 属性・相対パスの共通 helper。

use quick_xml::XmlVersion;
use quick_xml::events::BytesStart;

use crate::{CoreError, CoreResult};

/// 生の `href` / `src` を「最初の `#` の前」と「後ろ」に分ける。
///
/// パス部は前後の空白を除いたもの、フラグメントは `#` を除いて前後の空白を
/// 除いたもの（空なら `None`）を返す。2 つ目以降の `#` は区切りにしない。
/// フラグメントの `%XX` はここで一括して復号し、復号に失敗した場合は
/// [`CoreError::Validation`] を返す（nav / NCX / 章リンクで規則を揃える）。
/// 属性の実体参照は復号済みの文字列を渡す。
pub(super) fn split_fragment(raw: &str) -> CoreResult<(&str, Option<String>)> {
    let Some((path, fragment)) = raw.split_once('#') else {
        return Ok((raw.trim(), None));
    };
    let fragment = fragment.trim();
    let fragment = if fragment.is_empty() {
        None
    } else {
        Some(percent_decode(fragment, "フラグメント")?)
    };
    Ok((path.trim(), fragment))
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

    // パス部の `%XX` はここでは復号しない。リテラル `%` を含む実文件名を
    // 生のまま読めるようにするためで、復号フォールバックの失敗は呼び出し側の
    // `percent_decode` が Validation として報告する。

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
pub fn has_scheme(value: &str) -> bool {
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

/// `%XX`（16 進 2 桁）を UTF-8 バイト列として復号する。
///
/// `%` を含まない場合は入力をそのまま返す。`%` の後に 16 進 2 桁が無い場合、
/// 復号結果が不正な UTF-8 になる場合は [`CoreError::Validation`] を返す
/// （黙って未復号の文字列をパスや位置指定子として使わない）。
///
/// `subject` はエラーメッセージの主語（例: `章リンクの fragment`）に使う。
pub fn percent_decode(value: &str, subject: &str) -> CoreResult<String> {
    if !value.contains('%') {
        return Ok(value.to_string());
    }

    let bytes = value.as_bytes();
    let mut decoded: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut position = 0;
    while position < bytes.len() {
        if bytes[position] == b'%' {
            if position + 2 >= bytes.len() {
                return Err(CoreError::validation(format!(
                    "{subject} の %XX が途中で終わっています"
                )));
            }
            let (Some(high), Some(low)) = (
                hex_digit(bytes[position + 1]),
                hex_digit(bytes[position + 2]),
            ) else {
                return Err(CoreError::validation(format!(
                    "{subject} の %XX が不正です"
                )));
            };
            decoded.push(high * 16 + low);
            position += 3;
            continue;
        }
        decoded.push(bytes[position]);
        position += 1;
    }

    String::from_utf8(decoded)
        .map_err(|_| CoreError::validation(format!("{subject} の %XX が UTF-8 として不正です")))
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
