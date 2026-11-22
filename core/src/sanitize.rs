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

use ammonia::{Builder, UrlRelative};

use crate::image_dimensions::MAX_DIMENSION;

/// table 関連の数値属性（colspan/rowspan/start/value/span）の上限。
const MAX_TABLE_NUMBER: u32 = 1000;

/// サニタイズ対象の入力バイト数上限（4 MiB）。
///
/// EPUB の 1 entry 上限 64 MiB より小さく設定し、巨大な章がサニタイザで
/// 長時間消費することを防ぐ。
const MAX_SANITIZE_INPUT_BYTES: usize = 4 * 1024 * 1024;

/// サニタイズ対象のタグ深度上限。
///
/// 深い入れ子は HTML パーサのスタック消費を増やすため、軽量スキャンで
/// この深さを超えた入力は ammonia に渡さず拒否する。
const MAX_SANITIZE_TAG_DEPTH: usize = 128;

/// 上限超過時に返す固定の安全な文字列。
///
/// 入力内容を一切含めないため、EPUB の章テキストがログやエラーへ
/// 流出することがない。
const SANITIZE_LIMIT_REJECTED: &str = "";

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
///
/// 入力が [`MAX_SANITIZE_INPUT_BYTES`] または [`MAX_SANITIZE_TAG_DEPTH`] を
/// 超える場合は ammonia に渡さず、入力を含まない固定文字列を返す
/// （fail-closed）。シグネチャは変更しない。
pub fn sanitize_epub_html(html: &str) -> String {
    if exceeds_input_limits(html) {
        return SANITIZE_LIMIT_REJECTED.to_string();
    }
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
            let reject = (element == "img"
                && ((attribute == "src" && !is_local_reference(value))
                    || (matches!(attribute, "width" | "height") && !is_valid_dimension(value))))
                || (matches!(element, "td" | "th")
                    && matches!(attribute, "colspan" | "rowspan")
                    && !is_valid_table_number(value))
                || (element == "ol" && attribute == "start" && !is_valid_table_number(value))
                || (element == "li" && attribute == "value" && !is_valid_table_number(value))
                || (matches!(element, "col" | "colgroup")
                    && attribute == "span"
                    && !is_valid_table_number(value));
            if reject {
                None
            } else {
                Some(Cow::Borrowed(value))
            }
        });
    builder.clean(html).to_string()
}

/// 入力がサイズまたはタグ深度の上限を超えるか判定する。
///
/// サイズ判定を先に行うため、巨大な入力を深度走査せずに拒否できる。
fn exceeds_input_limits(html: &str) -> bool {
    html.len() > MAX_SANITIZE_INPUT_BYTES || exceeds_tag_depth(html)
}

/// タグ深度が [`MAX_SANITIZE_TAG_DEPTH`] を超えるかを軽量スキャンで判定する。
///
/// 開始タグはタグ名をスタックへ積み、終了タグはスタック先頭と名前が一致した
/// 場合だけ pop する。そのため `<div></x>` のような未対応の終了タグで深度が
/// 減ることはない。`br` などの void 要素と `/>` の自己閉じタグは積まない。
///
/// 開始タグ名は ASCII 英字で始まる必要があり、`1 < 2` のようなテキストは
/// タグ開始として数えない。タグ終端の `>` は引用符の内外を区別して探し、
/// 属性値内の `<` や `>` はタグ開始／終端として扱わない。
/// コメント・DOCTYPE・処理命令は終端まで読み飛ばす。
fn exceeds_tag_depth(html: &str) -> bool {
    let bytes = html.as_bytes();
    let mut stack: Vec<&[u8]> = Vec::with_capacity(MAX_SANITIZE_TAG_DEPTH + 1);
    let mut i = 0usize;

    while i < bytes.len() {
        let Some(offset) = bytes[i..].iter().position(|&byte| byte == b'<') else {
            break;
        };
        i += offset;
        let Some(&next) = bytes.get(i + 1) else {
            break;
        };

        if next == b'/' {
            let name_start = skip_ascii_whitespace(bytes, i + 2);
            let name = read_tag_name(bytes, name_start);
            let Some(end) = (if name.is_empty() {
                None
            } else {
                find_tag_end(bytes, name_start + name.len())
            }) else {
                // 不正または終端の無い終了タグはテキスト扱いにして続ける。
                i += 1;
                continue;
            };
            if stack
                .last()
                .is_some_and(|open| open.eq_ignore_ascii_case(name))
            {
                stack.pop();
            }
            i = end + 1;
            continue;
        }

        if next == b'!' || next == b'?' {
            let terminator: &[u8] = if next == b'!'
                && bytes.get(i + 2) == Some(&b'-')
                && bytes.get(i + 3) == Some(&b'-')
            {
                b"-->"
            } else if next == b'?' {
                b"?>"
            } else {
                b">"
            };
            let Some(after) = advance_past(bytes, i + 2, terminator) else {
                break;
            };
            i = after;
            continue;
        }

        // `<` の直後が英字でない場合はテキスト（例: `1 < 2`）として扱う。
        if !next.is_ascii_alphabetic() {
            i += 1;
            continue;
        }

        let name = read_tag_name(bytes, i + 1);
        let name_end = i + 1 + name.len();
        let Some(end) = find_tag_end(bytes, name_end) else {
            // 終端が無い開始タグはテキスト扱いにして走査を続ける。
            i += 1;
            continue;
        };

        // '/>' のみを自己閉じとみなす。引用なし属性の末尾 '/' (href=x/) は
        // 属性値の一部なので開始タグとして数える。
        let mut last_is_self_close_slash = false;
        let mut saw_significant = false;
        let mut j = name_end;
        let mut quote = None::<u8>;
        while j < end {
            let byte = bytes[j];
            if let Some(active) = quote {
                if byte == active {
                    quote = None;
                }
            } else {
                match byte {
                    b'"' | b'\'' => quote = Some(byte),
                    b'/' => {
                        saw_significant = true;
                        last_is_self_close_slash =
                            j > i + 1 && (bytes[j - 1].is_ascii_whitespace() || j - 1 < name_end);
                    }
                    _ if byte.is_ascii_whitespace() => {}
                    _ => {
                        saw_significant = true;
                        last_is_self_close_slash = false;
                    }
                }
            }
            j += 1;
        }
        let self_closing = saw_significant && last_is_self_close_slash;

        if !self_closing && !is_void_tag(name) {
            stack.push(name);
            if stack.len() > MAX_SANITIZE_TAG_DEPTH {
                return true;
            }
        }
        i = end + 1;
    }
    false
}

/// ASCII 空白を飛ばす。
fn skip_ascii_whitespace(bytes: &[u8], mut i: usize) -> usize {
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    i
}

/// タグ名を読み取る。ASCII 英字で始まり、英数字・'-'・':' が続く。
fn read_tag_name(bytes: &[u8], start: usize) -> &[u8] {
    let mut end = start;
    while end < bytes.len()
        && (bytes[end].is_ascii_alphanumeric() || matches!(bytes[end], b'-' | b':'))
    {
        end += 1;
    }
    &bytes[start..end]
}

/// タグ終端の '>' を引用符の内外を区別して探す。
fn find_tag_end(bytes: &[u8], mut i: usize) -> Option<usize> {
    let mut quote = None::<u8>;
    while i < bytes.len() {
        let byte = bytes[i];
        if let Some(active) = quote {
            if byte == active {
                quote = None;
            }
        } else {
            match byte {
                b'"' | b'\'' => quote = Some(byte),
                b'>' => return Some(i),
                _ => {}
            }
        }
        i += 1;
    }
    None
}

/// `i` 以降で最初に `needle` が現れる位置の直後を返す。見つからなければ None。
fn advance_past(bytes: &[u8], i: usize, needle: &[u8]) -> Option<usize> {
    bytes[i..]
        .windows(needle.len())
        .position(|window| window == needle)
        .map(|offset| i + offset + needle.len())
}

/// HTML の void 要素か判定する（開始タグだけでは深度を増やさない）。
///
/// HTML のタグ名は大文字小文字を区別しないため、比較は ASCII 非依存で行う。
fn is_void_tag(name: &[u8]) -> bool {
    const VOID_TAGS: &[&[u8]] = &[
        b"area", b"base", b"br", b"col", b"embed", b"hr", b"img", b"input", b"link", b"meta",
        b"param", b"source", b"track", b"wbr",
    ];
    VOID_TAGS.iter().any(|void| void.eq_ignore_ascii_case(name))
}

/// 画像の width/height 属性が安全な整数か判定する。
///
/// ASCII 数字 1〜5 桁だけを許し、`1..=MAX_DIMENSION` に収める。
/// 単位・負数・全角数字・上限超えは fail-closed で拒否する。
fn is_valid_dimension(value: &str) -> bool {
    if value.is_empty() || value.len() > 5 || !value.bytes().all(|b| b.is_ascii_digit()) {
        return false;
    }
    value
        .parse::<u32>()
        .is_ok_and(|dimension| (1..=MAX_DIMENSION).contains(&dimension))
}

/// table 関連の数値属性が安全な整数か判定する。
///
/// ASCII 数字 1〜4 桁だけを許し、`1..=MAX_TABLE_NUMBER` に収める。
/// 単位・負数・全角数字・上限超えは fail-closed で拒否する。
/// `len()` による桁数チェックで超長い入力は `parse` 前に弾く。
fn is_valid_table_number(value: &str) -> bool {
    if value.is_empty() || value.len() > 4 || !value.bytes().all(|b| b.is_ascii_digit()) {
        return false;
    }
    value
        .parse::<u32>()
        .is_ok_and(|number| (1..=MAX_TABLE_NUMBER).contains(&number))
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

// --- SVG 専用サニタイザ ---------------------------------------------------

/// SVG の既定名前空間。
const SVG_NAMESPACE: &str = "http://www.w3.org/2000/svg";

/// 表示を許可する SVG 要素。
///
/// html5ever が SVG のタグ名を camelCase に調整するため、
/// `clipPath` / `linearGradient` / `radialGradient` は camelCase で登録する。
/// `script` / `foreignObject` / SMIL などは [`SVG_CLEAN_CONTENT_TAGS`] に置き、
/// ここには入れない（ammonia が重複を assert するため）。
const SVG_ALLOWED_TAGS: &[&str] = &[
    "svg",
    "g",
    "rect",
    "circle",
    "ellipse",
    "line",
    "polygon",
    "polyline",
    "path",
    "text",
    "tspan",
    "defs",
    "use",
    "image",
    "symbol",
    "clipPath",
    "mask",
    "pattern",
    "linearGradient",
    "radialGradient",
    "stop",
    "marker",
    "desc",
    "view",
    "switch",
];

/// 内容ごと削除する要素。
///
/// 実行要素・埋め込み要素・SMIL に加え、`title` / `a` も SVG 画像には不要な
/// ため落とす。html5ever の調整に合わせ camelCase で登録する。
const SVG_CLEAN_CONTENT_TAGS: &[&str] = &[
    "script",
    "style",
    "foreignObject",
    "set",
    "animate",
    "animateColor",
    "animateMotion",
    "animateTransform",
    "discard",
    "mpath",
    "title",
    "template",
    "noscript",
    "iframe",
    "object",
    "embed",
    "a",
];

/// すべての SVG 要素で許可する属性。CSS 適用につながる `style` は含めない。
const SVG_GENERIC_ATTRIBUTES: &[&str] = &["id"];

/// 図形・テキストで許可する描画属性。
const SVG_SHAPE_ATTRIBUTES: &[&str] = &[
    "cx",
    "cy",
    "d",
    "dx",
    "dy",
    "fill",
    "fill-opacity",
    "fill-rule",
    "font-family",
    "font-size",
    "height",
    "opacity",
    "points",
    "preserveAspectRatio",
    "r",
    "rx",
    "ry",
    "stroke",
    "stroke-dasharray",
    "stroke-dashoffset",
    "stroke-linecap",
    "stroke-linejoin",
    "stroke-opacity",
    "stroke-width",
    "text-anchor",
    "transform",
    "width",
    "x",
    "x1",
    "x2",
    "y",
    "y1",
    "y2",
];

/// SVG 要素ごとに許可する属性。
///
/// html5ever は `viewBox` / `preserveAspectRatio` / `gradientUnits` などを
/// camelCase に調整し、`xmlns:xlink` はローカル名 `xlink` として保持する。
fn svg_tag_attributes() -> HashMap<&'static str, HashSet<&'static str>> {
    let shape: HashSet<&'static str> = SVG_SHAPE_ATTRIBUTES.iter().copied().collect();
    let mut linked = shape.clone();
    linked.extend(["href", "preserveAspectRatio"]);
    HashMap::from([
        (
            "svg",
            HashSet::from([
                "xmlns",
                "xlink",
                "viewBox",
                "width",
                "height",
                "preserveAspectRatio",
            ]),
        ),
        ("image", linked.clone()),
        ("use", HashSet::from(["href", "x", "y", "width", "height"])),
        ("rect", shape.clone()),
        ("circle", shape.clone()),
        ("ellipse", shape.clone()),
        ("line", shape.clone()),
        ("polygon", shape.clone()),
        ("polyline", shape.clone()),
        ("path", shape.clone()),
        ("text", linked.clone()),
        ("tspan", linked),
        (
            "symbol",
            HashSet::from([
                "viewBox",
                "preserveAspectRatio",
                "x",
                "y",
                "width",
                "height",
            ]),
        ),
        ("clipPath", HashSet::from(["clipPathUnits"])),
        (
            "mask",
            HashSet::from(["maskUnits", "maskContentUnits", "x", "y", "width", "height"]),
        ),
        (
            "pattern",
            HashSet::from([
                "patternUnits",
                "patternContentUnits",
                "patternTransform",
                "viewBox",
                "preserveAspectRatio",
                "x",
                "y",
                "width",
                "height",
                "href",
            ]),
        ),
        (
            "linearGradient",
            HashSet::from([
                "gradientUnits",
                "gradientTransform",
                "x1",
                "y1",
                "x2",
                "y2",
                "href",
            ]),
        ),
        (
            "radialGradient",
            HashSet::from([
                "gradientUnits",
                "gradientTransform",
                "cx",
                "cy",
                "r",
                "fx",
                "fy",
                "href",
            ]),
        ),
        (
            "stop",
            HashSet::from(["offset", "stop-color", "stop-opacity"]),
        ),
        (
            "marker",
            HashSet::from([
                "markerWidth",
                "markerHeight",
                "markerUnits",
                "refX",
                "refY",
                "orient",
                "viewBox",
                "preserveAspectRatio",
            ]),
        ),
    ])
}

/// SVG の XML 文字列を表示用にサニタイズする（fail-closed）。
///
/// XML 処理命令を入力側で除いたうえで ammonia の許可リストを適用し、
/// 実行・埋め込み・アニメーション要素、イベントハンドラ、外部スキームの
/// 参照を落とす。相対参照（例: `images/pic.png`, `#gradient`）は残し、
/// 表示側が解決する。XHTML 用の [`sanitize_epub_html`] とは独立した契約。
///
/// 空入力・XML 処理命令の閉じ欠落・SVG ルート要素が無い場合・出力が空の
/// 場合は `None` を返す。
pub fn sanitize_svg_xml(svg: &str) -> Option<String> {
    if svg.trim().is_empty() {
        return None;
    }
    let stripped = strip_xml_processing_instructions(svg)?;
    if stripped.trim().is_empty() {
        return None;
    }

    let mut builder = Builder::default();
    builder
        .tags(SVG_ALLOWED_TAGS.iter().copied().collect())
        .clean_content_tags(SVG_CLEAN_CONTENT_TAGS.iter().copied().collect())
        .generic_attributes(SVG_GENERIC_ATTRIBUTES.iter().copied().collect())
        .tag_attributes(svg_tag_attributes())
        // 絶対スキームはすべて拒否する。相対 URL は PassThrough で残すが、
        // `//host/path` も相対として扱われるため attribute_filter で弾く。
        .url_schemes(HashSet::new())
        .url_relative(UrlRelative::PassThrough)
        .strip_comments(true)
        .link_rel(None)
        // 入力の xmlns 値に関わらず正しい既定名前空間を付与する。
        .set_tag_attribute_value("svg", "xmlns", SVG_NAMESPACE)
        .attribute_filter(|element, attribute, value| {
            let reject = (element == "svg" && attribute == "xmlns" && value != SVG_NAMESPACE)
                || (matches!(attribute, "href" | "src") && !is_local_reference(value));
            if reject {
                None
            } else {
                Some(Cow::Borrowed(value))
            }
        });

    // PI 除去で残った前後の空白テキストノードは取り除き、SVG ルートだけを返す。
    let cleaned = builder.clean(&stripped).to_string();
    let cleaned = cleaned.trim().to_string();
    if cleaned.is_empty() || !cleaned.starts_with("<svg") || !cleaned.ends_with("</svg>") {
        return None;
    }
    Some(cleaned)
}

/// XML 処理命令（`<?...?>`）を入力から除去する。
///
/// ammonia は DOM 上の ProcessingInstruction を常に削除するが、HTML パーサの
/// 文脈では `<?xml-stylesheet?>` がコメントとして扱われるため、動作をパーサ
/// 実装に依存させず入力側で明示的に落とす。閉じが無い場合は fail-closed。
fn strip_xml_processing_instructions(svg: &str) -> Option<String> {
    if !svg.contains("<?") {
        return Some(svg.to_string());
    }
    let mut output = String::with_capacity(svg.len());
    let mut rest = svg;
    while let Some(start) = rest.find("<?") {
        output.push_str(&rest[..start]);
        let after_open = &rest[start + 2..];
        let end = after_open.find("?>")?;
        rest = &after_open[end + 2..];
    }
    output.push_str(rest);
    Some(output)
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
    fn keeps_valid_table_numbers() {
        let sanitized = sanitize_epub_html(
            r#"<table><colgroup span="2"><col span="3"></colgroup><tr><td colspan="3" rowspan="2">cell</td><th colspan="4" rowspan="1">head</th></tr></table><ol start="5"><li value="2">item</li></ol>"#,
        );
        assert!(sanitized.contains(r#"colspan="3""#), "{sanitized}");
        assert!(sanitized.contains(r#"colspan="4""#), "{sanitized}");
        assert!(sanitized.contains(r#"rowspan="2""#), "{sanitized}");
        assert!(sanitized.contains(r#"rowspan="1""#), "{sanitized}");
        assert!(sanitized.contains(r#"start="5""#), "{sanitized}");
        assert!(sanitized.contains(r#"value="2""#), "{sanitized}");
        assert!(sanitized.contains(r#"span="2""#), "{sanitized}");
        assert!(sanitized.contains(r#"span="3""#), "{sanitized}");
        assert!(sanitized.contains(">cell<"), "{sanitized}");
        assert!(sanitized.contains(">head<"), "{sanitized}");
        assert!(sanitized.contains(">item<"), "{sanitized}");
    }

    #[test]
    fn removes_invalid_table_numbers() {
        let huge = "9".repeat(10_000);
        for value in ["0", "-1", "1001", "１２３", "3px", huge.as_str()] {
            let cases = [
                (
                    format!(r#"<table><tr><td colspan="{value}">cell</td></tr></table>"#),
                    "colspan",
                ),
                (
                    format!(r#"<table><tr><td rowspan="{value}">cell</td></tr></table>"#),
                    "rowspan",
                ),
                (
                    format!(r#"<table><tr><th colspan="{value}">cell</th></tr></table>"#),
                    "colspan",
                ),
                (
                    format!(r#"<table><tr><th rowspan="{value}">cell</th></tr></table>"#),
                    "rowspan",
                ),
                (
                    format!(r#"<ol start="{value}"><li>item</li></ol>"#),
                    "start",
                ),
                (
                    format!(r#"<ol><li value="{value}">item</li></ol>"#),
                    "value",
                ),
                (
                    format!(
                        r#"<table><colgroup span="{value}"></colgroup><tr><td>cell</td></tr></table>"#
                    ),
                    "span",
                ),
                (
                    format!(
                        r#"<table><colgroup><col span="{value}"></colgroup><tr><td>cell</td></tr></table>"#
                    ),
                    "span",
                ),
            ];
            for (html, attribute) in cases {
                let sanitized = sanitize_epub_html(&html);
                assert!(
                    !sanitized.contains(&format!("{attribute}=")),
                    "kept {attribute}={value}: {sanitized}"
                );
                assert!(
                    sanitized.contains("cell") || sanitized.contains("item"),
                    "lost body text: {value} -> {sanitized}"
                );
            }
        }
    }

    #[test]
    fn keeps_non_numeric_table_attributes() {
        let sanitized = sanitize_epub_html(
            r#"<table summary="s"><tr><th scope="col" headers="h" abbr="a">head</th><td scope="row">cell</td></tr></table><ol type="a" reversed start="3"><li value="4">item</li></ol>"#,
        );
        assert!(sanitized.contains(r#"summary="s""#), "{sanitized}");
        assert!(sanitized.contains(r#"scope="col""#), "{sanitized}");
        assert!(sanitized.contains(r#"headers="h""#), "{sanitized}");
        assert!(sanitized.contains(r#"abbr="a""#), "{sanitized}");
        assert!(sanitized.contains(r#"type="a""#), "{sanitized}");
        assert!(sanitized.contains("reversed"), "{sanitized}");
        assert!(sanitized.contains(r#"start="3""#), "{sanitized}");
        assert!(sanitized.contains(r#"value="4""#), "{sanitized}");
    }

    #[test]
    fn table_number_boundaries() {
        assert!(is_valid_table_number("1"));
        assert!(is_valid_table_number("1000"));
        assert!(is_valid_table_number("0003"));
        assert!(!is_valid_table_number("0"));
        assert!(!is_valid_table_number("1001"));
        assert!(!is_valid_table_number("00001"));
        assert!(!is_valid_table_number(""));
        assert!(!is_valid_table_number("3px"));
        assert!(!is_valid_table_number("１２"));
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

    #[test]
    fn rejects_deeply_nested_content_without_leaking_text() {
        // 上限を超える入れ子は ammonia に渡さず固定文字列を返す。
        let depth = MAX_SANITIZE_TAG_DEPTH + 1;
        let mut html = String::new();
        for _ in 0..depth {
            html.push_str("<div>");
        }
        html.push_str(r#"<p onclick="alert(1)">deep</p>"#);
        for _ in 0..depth {
            html.push_str("</div>");
        }
        let sanitized = sanitize_epub_html(&html);
        assert_eq!(sanitized, SANITIZE_LIMIT_REJECTED);
        assert!(!sanitized.contains("deep"));
        assert!(!sanitized.contains("onclick"));
    }

    #[test]
    fn rejects_10k_nesting_depth() {
        // 10000 階層は従来約 5 秒かかっていたが、軽量スキャンで拒否する。
        // 時間依存の assert は不安定なため使わず、結果だけを検証する。
        let depth = 10_000;
        let mut html = String::new();
        for _ in 0..depth {
            html.push_str("<div>");
        }
        html.push_str("<p>very deep</p>");
        for _ in 0..depth {
            html.push_str("</div>");
        }
        let sanitized = sanitize_epub_html(&html);
        assert_eq!(sanitized, SANITIZE_LIMIT_REJECTED);
    }

    #[test]
    fn keeps_nesting_at_limit() {
        // 上限ちょうどの入れ子は従来どおりサニタイズされる。
        // 内側の <p> も 1 階層として数えるため div は上限 - 1 にする。
        let mut html = String::new();
        for _ in 0..MAX_SANITIZE_TAG_DEPTH - 1 {
            html.push_str("<div>");
        }
        html.push_str(r#"<p onclick="alert(1)">edge</p>"#);
        for _ in 0..MAX_SANITIZE_TAG_DEPTH - 1 {
            html.push_str("</div>");
        }
        let sanitized = sanitize_epub_html(&html);
        assert!(sanitized.contains("edge"), "{sanitized}");
        assert!(!sanitized.contains("onclick"), "{sanitized}");
    }

    #[test]
    fn rejects_oversized_input() {
        // 4 MiB を超える入力は走査せず固定文字列を返す。
        let html = format!("<p>{}</p>", "x".repeat(MAX_SANITIZE_INPUT_BYTES));
        assert!(html.len() > MAX_SANITIZE_INPUT_BYTES);
        assert!(exceeds_input_limits(&html));
        assert_eq!(sanitize_epub_html(&html), SANITIZE_LIMIT_REJECTED);
    }

    #[test]
    fn sanitizes_normal_chapter_under_limits() {
        // 通常の章（見出し・段落・画像・改行）は従来どおりサニタイズされる。
        // void 要素が深度として積み上がらないことも確認する。
        let mut body = String::from(r#"<h1 id="title">第1章</h1>"#);
        for index in 0..64 {
            body.push_str(&format!(
                r#"<p class="para"><strong>段落{index}</strong>本文<img src="images/p{index}.png" alt="図"><br></p>"#
            ));
        }
        let sanitized = sanitize_epub_html(&body);
        assert!(
            sanitized.contains(r#"<h1 id="title">第1章</h1>"#),
            "{sanitized}"
        );
        assert!(sanitized.contains(r#"<p class="para"><strong>段落0</strong>"#));
        assert!(sanitized.contains(r#"src="images/p0.png""#));
        assert!(sanitized.contains("<br"));
        assert!(!sanitized.contains("<script"));
    }

    #[test]
    fn tag_depth_scan_ignores_void_self_closing_and_comments() {
        // void 要素・自己閉じタグ・コメントは深度を増やさない。
        let mut html = String::from("<!-- <div> --><?xml version='1.0'?>");
        for _ in 0..MAX_SANITIZE_TAG_DEPTH {
            html.push_str(r#"<br><img src="x.png"><hr/>"#);
        }
        html.push_str("<p>ok</p>");
        assert!(!exceeds_tag_depth(&html));

        let mut nested = String::new();
        for _ in 0..=MAX_SANITIZE_TAG_DEPTH {
            nested.push_str("<section>");
        }
        assert!(exceeds_tag_depth(&nested));
    }

    #[test]
    fn uppercase_void_tags_do_not_inflate_depth() {
        // HTML のタグ名は大文字小文字を区別しない。<BR>/<IMG>/<Hr/> も
        // void 要素として扱い、深度に積まない。
        let mut html = String::new();
        for _ in 0..MAX_SANITIZE_TAG_DEPTH * 2 {
            html.push_str(r#"<BR><br><IMG SRC="x.png"><Hr/>"#);
        }
        html.push_str("<p>ok</p>");
        assert!(!exceeds_tag_depth(&html));
        let sanitized = sanitize_epub_html(&html);
        assert!(sanitized.contains("ok"), "{sanitized}");
        assert!(!sanitized.contains("<BR"), "{sanitized}");
    }

    #[test]
    fn text_angle_brackets_and_quoted_attributes_do_not_inflate_depth() {
        // 本文の `1 < 2` と属性値内の `<` `>` `"` `'` はタグ開始と数えない。
        let mut body = String::new();
        for index in 0..256 {
            body.push_str(&format!(
                r#"<p title="a > b < c ' d">式: 1 < 2 かつ 3 > 2 (段落{index})</p>"#
            ));
        }
        assert!(!exceeds_tag_depth(&body));
        let sanitized = sanitize_epub_html(&body);
        assert!(sanitized.contains("段落0"), "{sanitized}");
        assert!(sanitized.contains("&lt;"), "{sanitized}");
    }

    #[test]
    fn unmatched_end_tags_do_not_reduce_depth() {
        // `<div></x>` は HTML パーサで div が閉じられないため、
        // 繰り返すと深い入れ子として拒否されなければならない。
        let mut html = String::new();
        for _ in 0..=MAX_SANITIZE_TAG_DEPTH {
            html.push_str("<div></x>");
        }
        assert!(exceeds_tag_depth(&html));
        assert_eq!(sanitize_epub_html(&html), SANITIZE_LIMIT_REJECTED);
    }

    #[test]
    fn keeps_numeric_image_dimensions() {
        let sanitized = sanitize_epub_html(r#"<img src="x.png" width="100" height="200" alt="x">"#);
        assert!(sanitized.contains(r#"width="100""#), "{sanitized}");
        assert!(sanitized.contains(r#"height="200""#), "{sanitized}");
    }

    #[test]
    fn removes_invalid_image_dimensions() {
        for value in ["100px", "-1", "0", "16385", "１２３"] {
            let sanitized = sanitize_epub_html(&format!(
                r#"<img src="x.png" width="{value}" height="{value}" alt="x">"#
            ));
            assert!(!sanitized.contains("width="), "kept width: {value}");
            assert!(!sanitized.contains("height="), "kept height: {value}");
        }
    }

    #[test]
    fn keeps_max_and_leading_zero_dimensions() {
        let sanitized = sanitize_epub_html(r#"<img src="x.png" width="16384" height="007">"#);
        assert!(sanitized.contains(r#"width="16384""#), "{sanitized}");
        assert!(sanitized.contains(r#"height="007""#), "{sanitized}");
    }

    #[test]
    fn external_src_still_removed_with_dimensions() {
        let sanitized = sanitize_epub_html(
            r#"<img src="https://evil.com/x.png" width="100" height="50" alt="x">"#,
        );
        assert!(!sanitized.contains("src="), "{sanitized}");
        assert!(sanitized.contains(r#"width="100""#), "{sanitized}");
        assert!(sanitized.contains(r#"height="50""#), "{sanitized}");
    }

    /// E2E の EVIL_SVG と同じ悪意要素を含む入力。
    const EVIL_SVG: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<?xml-stylesheet href="https://attacker.example/style.css" type="text/css"?>
<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink"
     width="20" height="20" onload="window.__pwned = 1">
<script>window.__pwned = 1;</script>
<rect width="20" height="20" fill="red"/>
<foreignObject><div onclick="window.__pwned = 1">x</div></foreignObject>
<image xlink:href="java&#10;script:window.__pwned = 1" width="1" height="1"/>
<set attributeName="onload" to="window.__pwned = 1" begin="0s"/>
<animate attributeName="href" values="javascript:window.__pwned = 1" begin="0s"/>
<use xlink:href="https://attacker.example/track.svg#x" width="1" height="1"/>
<image href="data:image/png;base64,AAAA" width="1" height="1"/>
<image href="images/pic.png" width="1" height="1"/>
</svg>
"#;

    #[test]
    fn svg_sanitizer_removes_evil_content_and_keeps_relative_images() {
        let clean = sanitize_svg_xml(EVIL_SVG).expect("EVIL_SVG should be sanitized");
        let lowered = clean.to_ascii_lowercase();
        assert!(clean.starts_with("<svg"), "{clean}");
        assert!(clean.trim_end().ends_with("</svg>"), "{clean}");
        assert!(
            clean.contains(r#"xmlns="http://www.w3.org/2000/svg""#),
            "{clean}"
        );
        assert!(clean.contains(r#"width="20""#), "{clean}");
        assert!(clean.contains(r#"height="20""#), "{clean}");
        assert!(clean.contains("<rect"), "{clean}");
        assert!(!lowered.contains("<script"), "{clean}");
        assert!(!lowered.contains("onload="), "{clean}");
        assert!(!lowered.contains("onclick="), "{clean}");
        assert!(!lowered.contains("foreignobject"), "{clean}");
        assert!(!lowered.contains("<set"), "{clean}");
        assert!(!lowered.contains("<animate"), "{clean}");
        assert!(!lowered.contains("javascript:"), "{clean}");
        assert!(!lowered.replace('\n', "").contains("script:"), "{clean}");
        assert!(!lowered.contains("attacker.example"), "{clean}");
        assert!(!lowered.contains("data:image"), "{clean}");
        assert!(!lowered.contains("xml-stylesheet"), "{clean}");
        assert!(!lowered.contains("__pwned"), "{clean}");
        assert!(clean.contains("images/pic.png"), "{clean}");
    }

    #[test]
    fn svg_sanitizer_rejects_empty_and_non_svg_input() {
        assert_eq!(sanitize_svg_xml(""), None);
        assert_eq!(sanitize_svg_xml("  \n\t"), None);
        assert_eq!(sanitize_svg_xml("<p>not svg</p>"), None);
        assert_eq!(sanitize_svg_xml("<?xml version='1.0'?>"), None);
        assert_eq!(sanitize_svg_xml("<?xml version='1.0'<svg/>"), None);
    }

    #[test]
    fn svg_sanitizer_rejects_external_and_root_relative_references() {
        for href in [
            "//evil.example/x.svg",
            "/images/pic.png",
            r"\\evil.example\x.svg",
            "https://example.com/x.svg",
            "data:image/svg+xml;base64,AAAA",
            "blob:https://example.com/id",
            "",
        ] {
            let svg = format!(
                r#"<svg xmlns="http://www.w3.org/2000/svg"><image href="{href}" width="1" height="1"/></svg>"#
            );
            let clean = sanitize_svg_xml(&svg).expect("svg should be sanitized");
            assert!(!clean.contains("href="), "kept href {href:?}: {clean}");
        }
    }

    #[test]
    fn svg_sanitizer_keeps_fragment_references_and_camel_case_attributes() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"><defs><linearGradient id="g"><stop offset="0" stop-color="red"/></linearGradient></defs><rect width="10" height="10" fill="url(#g)" preserveAspectRatio="xMidYMid meet"/></svg>"#;
        let clean = sanitize_svg_xml(svg).expect("svg should be sanitized");
        assert!(clean.contains(r#"viewBox="0 0 10 10""#), "{clean}");
        assert!(clean.contains("linearGradient"), "{clean}");
        assert!(clean.contains(r#"fill="url(#g)""#), "{clean}");
        assert!(clean.contains("preserveAspectRatio"), "{clean}");
    }
}
