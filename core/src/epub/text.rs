//! XHTML のタイトル・本文テキスト抽出と文字列正規化。

use quick_xml::Reader;
use quick_xml::events::Event;

use crate::{CoreError, CoreResult};

use super::xmlutil::percent_decode;
use super::{BLOCK_ELEMENTS, BODY_ELEMENT, SKIPPED_ELEMENTS, TITLE_ELEMENT};

/// XHTML の `<head>` の最初の `<title>` の内容を正規化して返す。無ければ `None`。
///
/// 実体参照は定義済みの 5 つと数値参照だけを文字に戻す（本文抽出と同じ規則）。
/// 空白だけの `<title>` は情報が無いものとして `None` にする。`<body>` に入った
/// 後に現れる `<title>`（インライン SVG のアクセシブル名など）は章の表示名では
/// ないため対象にしない。要素が閉じられていない・解析できない場合は
/// [`CoreError::Archive`]。
pub(super) fn extract_title(xml: &str, label: &str) -> CoreResult<Option<String>> {
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
pub(super) fn extract_text(xml: &str, label: &str) -> CoreResult<String> {
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
pub(super) fn named_reference(name: &str) -> Option<char> {
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
pub(super) fn normalize_text(value: &str) -> String {
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

/// 章の `href` を比較するための正規化したパスを返す。
///
/// `%XX` を復号した形にする。復号に失敗した場合は [`CoreError::Validation`] を
/// 返し、未復号の文字列をパスとして使わない。
pub(super) fn normalized_chapter_path(path: &str) -> CoreResult<String> {
    percent_decode(path, "章の href")
}
