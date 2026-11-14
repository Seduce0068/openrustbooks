//! EPUB の XHTML を表示用にサニタイズする。
//!
//! 許可リスト方式で要素と属性を絞り、スクリプト・イベントハンドラ・
//! スタイル・外部画像を除去する。CSS は適用しない。画像の相対 URL は
//! 保持し、表示側が `chapter_resource` で解決する。
//!
//! サニタイズは表示の前提であり、EPUB の内容を書き換えない。読み取り
//! 専用の方針（ADR-0005）を維持する。

use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

use ammonia::Builder;

/// 表示を許可する要素。
///
/// `script`・`style`・`iframe`・`object`・`form` などの実行・埋め込み・
/// 入力要素は含めない。`svg` と `math` もインラインでは許可しない。
const ALLOWED_TAGS: &[&str] = &[
    "a",
    "abbr",
    "article",
    "aside",
    "b",
    "blockquote",
    "br",
    "caption",
    "cite",
    "code",
    "col",
    "colgroup",
    "dd",
    "del",
    "dfn",
    "div",
    "dl",
    "dt",
    "em",
    "figcaption",
    "figure",
    "footer",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "header",
    "hr",
    "i",
    "img",
    "ins",
    "kbd",
    "li",
    "main",
    "mark",
    "nav",
    "ol",
    "p",
    "pre",
    "q",
    "rp",
    "rt",
    "ruby",
    "s",
    "samp",
    "section",
    "small",
    "span",
    "strong",
    "sub",
    "summary",
    "sup",
    "table",
    "tbody",
    "td",
    "tfoot",
    "th",
    "thead",
    "time",
    "tr",
    "u",
    "ul",
    "var",
];

/// すべての要素で許可する属性。
const GENERIC_ATTRIBUTES: &[&str] = &["id", "class", "lang", "dir", "title", "role"];

/// 要素ごとに許可する属性。
fn tag_attributes() -> HashMap<&'static str, HashSet<&'static str>> {
    HashMap::from([
        ("a", HashSet::from(["href"])),
        ("img", HashSet::from(["src", "alt", "width", "height"])),
        (
            "td",
            HashSet::from(["colspan", "rowspan", "headers", "scope", "abbr"]),
        ),
        (
            "th",
            HashSet::from(["colspan", "rowspan", "headers", "scope", "abbr"]),
        ),
        ("ol", HashSet::from(["start", "type", "reversed"])),
        ("li", HashSet::from(["value"])),
        ("time", HashSet::from(["datetime"])),
        ("del", HashSet::from(["cite", "datetime"])),
        ("ins", HashSet::from(["cite", "datetime"])),
        ("blockquote", HashSet::from(["cite"])),
        ("q", HashSet::from(["cite"])),
        ("col", HashSet::from(["span"])),
        ("colgroup", HashSet::from(["span"])),
        ("table", HashSet::from(["summary"])),
    ])
}

/// EPUB の XHTML を表示用の HTML 断片に変換する。
///
/// 許可した要素と属性だけを残し、`script` などの実行要素、イベント
/// ハンドラ、`style` 属性、外部画像を除去する。相対 URL の画像は
/// そのまま残すため、呼び出し側が `EpubBook::chapter_resource` で
/// 解決する。`html` と `body` の外枠は落とし、本文の断片を返す。
pub fn sanitize_epub_html(html: &str) -> String {
    let mut builder = Builder::default();
    builder
        .tags(ALLOWED_TAGS.iter().copied().collect())
        .clean_content_tags(HashSet::from([
            "script", "style", "title", "template", "noscript",
        ]))
        .generic_attributes(GENERIC_ATTRIBUTES.iter().copied().collect())
        .tag_attributes(tag_attributes())
        .url_schemes(HashSet::from(["http", "https", "mailto"]))
        .link_rel(Some("noopener noreferrer"))
        .attribute_filter(|element, attribute, value| {
            if element == "img" && attribute == "src" && !is_local_reference(value) {
                None
            } else {
                Some(Cow::Borrowed(value))
            }
        });
    builder.clean(html).to_string()
}

/// 画像として表示してよい参照か判定する。
///
/// 相対パスだけを許可する。スキーム付き URL とプロトコル相対 URL
/// (`//host/...`) は外部参照として拒否する。判定の前に
/// [`normalize_reference`] でブラウザの URL 正規化に合わせる。
fn is_local_reference(value: &str) -> bool {
    let value = normalize_reference(value);
    !value.is_empty()
        && !value.starts_with('/')
        && !value.chars().any(char::is_control)
        && !has_url_scheme(&value)
}

/// 参照をブラウザが URL を解釈する前の正規化に合わせる。
///
/// 前後の C0 制御文字と空白を除き、残るタブ・LF・CR を除き、
/// バックスラッシュを `/` に読み替える。特殊スキームの URL では
/// バックスラッシュが `/` として扱われるため、`\\host` のような
/// プロトコル相対 URL の抜け道を塞ぐ。
fn normalize_reference(value: &str) -> Cow<'_, str> {
    let trimmed = value.trim_matches(|character: char| character <= ' ');
    if !trimmed
        .chars()
        .any(|character| matches!(character, '\t' | '\n' | '\r' | '\\'))
    {
        return Cow::Borrowed(trimmed);
    }
    let mut normalized = String::with_capacity(trimmed.len());
    for character in trimmed.chars() {
        match character {
            '\t' | '\n' | '\r' => {}
            '\\' => normalized.push('/'),
            other => normalized.push(other),
        }
    }
    Cow::Owned(normalized)
}

/// 値がスキーム付き URL か判定する。
fn has_url_scheme(value: &str) -> bool {
    let Some((scheme, _)) = value.split_once(':') else {
        return false;
    };
    let mut chars = scheme.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    first.is_ascii_alphabetic()
        && chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_scripts_and_event_handlers() {
        let sanitized =
            sanitize_epub_html(r#"<p onclick="alert(1)">safe</p><script>alert(1)</script>"#);
        assert_eq!(sanitized, "<p>safe</p>");
    }

    #[test]
    fn removes_embedded_and_form_elements() {
        let sanitized = sanitize_epub_html(
            r#"<iframe src="x"></iframe><object data="x"></object><form><input></form><p>ok</p>"#,
        );
        assert_eq!(sanitized, "<p>ok</p>");
    }

    #[test]
    fn keeps_safe_structure_and_ids() {
        let sanitized = sanitize_epub_html(
            r#"<section id="ch1"><h1 class="title">Title</h1><p><strong>bold</strong></p></section>"#,
        );
        assert!(sanitized.contains(r#"<section id="ch1">"#));
        assert!(sanitized.contains(r#"<h1 class="title">Title</h1>"#));
        assert!(sanitized.contains("<strong>bold</strong>"));
    }

    #[test]
    fn removes_style_attributes_and_tags() {
        let sanitized =
            sanitize_epub_html(r#"<style>p{color:red}</style><p style="color:red">x</p>"#);
        assert_eq!(sanitized, "<p>x</p>");
    }

    #[test]
    fn keeps_relative_image_sources() {
        let sanitized = sanitize_epub_html(r#"<img src="images/pic.png" alt="pic">"#);
        assert!(sanitized.contains(r#"src="images/pic.png""#));
        assert!(sanitized.contains(r#"alt="pic""#));
    }

    #[test]
    fn removes_external_image_sources() {
        for src in [
            "https://example.com/x.png",
            "http://example.com/x.png",
            "//example.com/x.png",
            "data:image/png;base64,AAAA",
        ] {
            let sanitized = sanitize_epub_html(&format!(r#"<img src="{src}" alt="x">"#));
            assert!(!sanitized.contains("src="), "kept external src: {src}");
        }
    }

    #[test]
    fn removes_javascript_links_and_adds_rel() {
        let sanitized = sanitize_epub_html(
            r#"<a href="javascript:alert(1)">bad</a><a href="https://example.com">good</a>"#,
        );
        assert!(!sanitized.contains("javascript:"));
        assert!(sanitized.contains(r#"href="https://example.com""#));
        assert!(sanitized.contains(r#"rel="noopener noreferrer""#));
    }

    #[test]
    fn drops_html_and_head_wrappers() {
        let sanitized = sanitize_epub_html(
            r#"<html><head><title>Book</title></head><body><p>body</p></body></html>"#,
        );
        assert_eq!(sanitized, "<p>body</p>");
    }

    #[test]
    fn keeps_table_structure_and_span_attributes() {
        let sanitized = sanitize_epub_html(
            r#"<table><tr><th colspan="2">h</th></tr><tr><td>v</td></tr></table>"#,
        );
        assert!(sanitized.contains("<table>"));
        assert!(sanitized.contains(r#"<th colspan="2">h</th>"#));
        assert!(sanitized.contains("<td>v</td>"));
    }

    #[test]
    fn rejects_empty_and_root_relative_image_sources() {
        let sanitized = sanitize_epub_html(r#"<img src="/x.png" alt="x">"#);
        assert!(!sanitized.contains("src="));
        let sanitized = sanitize_epub_html(r#"<img src="" alt="x">"#);
        assert!(!sanitized.contains("src="));
    }

    #[test]
    fn rejects_backslash_protocol_relative_image_sources() {
        for src in [
            r"\\evil.com/x.png",
            r"\/evil.com/x.png",
            r"/\evil.com/x.png",
            r"\evil.com/x.png",
        ] {
            let sanitized = sanitize_epub_html(&format!(r#"<img src="{src}" alt="x">"#));
            assert!(!sanitized.contains("src="), "kept external src: {src}");
        }
    }

    #[test]
    fn rejects_entity_encoded_backslash_image_sources() {
        let sanitized = sanitize_epub_html(r#"<img src="&#92;&#92;evil.com/x.png" alt="x">"#);
        assert!(!sanitized.contains("src="));
    }

    #[test]
    fn rejects_schemes_with_embedded_whitespace() {
        for src in [
            "java\tscript:alert(1)",
            "java\nscript:alert(1)",
            "java\rscript:alert(1)",
        ] {
            let sanitized = sanitize_epub_html(&format!(r#"<img src="{src}" alt="x">"#));
            assert!(!sanitized.contains("src="), "kept scheme: {src:?}");
        }
        let sanitized = sanitize_epub_html(r#"<img src="java&#9;script:alert(1)" alt="x">"#);
        assert!(!sanitized.contains("src="));
    }

    #[test]
    fn rejects_protocol_relative_hidden_by_control_characters() {
        for src in ["\u{1}\u{1}//evil.com/x.png", "\u{1}\\/evil.com/x.png"] {
            let sanitized = sanitize_epub_html(&format!(r#"<img src="{src}" alt="x">"#));
            assert!(!sanitized.contains("src="), "kept external src: {src:?}");
        }
    }

    #[test]
    fn rejects_non_http_schemes_in_image_sources() {
        for src in [
            "DATA:image/png;base64,AAAA",
            "blob:https://example.com/id",
            "vbscript:msgbox(1)",
            "file:///etc/passwd",
        ] {
            let sanitized = sanitize_epub_html(&format!(r#"<img src="{src}" alt="x">"#));
            assert!(!sanitized.contains("src="), "kept scheme: {src}");
        }
    }

    #[test]
    fn removes_srcset_and_keeps_relative_source() {
        let sanitized =
            sanitize_epub_html(r#"<img src="x.png" srcset="https://evil.com/x 1x" alt="x">"#);
        assert!(!sanitized.contains("srcset"));
        assert!(sanitized.contains(r#"src="x.png""#));
    }

    #[test]
    fn removes_scriptable_containers() {
        for html in [
            r#"<svg><script>alert(1)</script></svg>"#,
            r#"<template><img src="x" onerror="alert(1)"></template>"#,
            r#"<noscript><img src="x" onerror="alert(1)"></noscript>"#,
            r#"<iframe srcdoc="<script>alert(1)</script>"></iframe>"#,
        ] {
            let sanitized = sanitize_epub_html(html);
            assert!(
                !sanitized.contains("script"),
                "kept script: {html} -> {sanitized}"
            );
            assert!(
                !sanitized.contains("onerror"),
                "kept handler: {html} -> {sanitized}"
            );
        }
    }

    #[test]
    fn keeps_relative_source_with_backslash_separator() {
        let sanitized = sanitize_epub_html(r#"<img src="images\pic.png" alt="x">"#);
        assert!(sanitized.contains(r#"src="images\pic.png""#));
    }
}
