//! Markdown テキストを表示用の安全な HTML 断片へ変換する。
//!
//! レンダリングには `comrak`（`default-features = false`）を使う。既定の
//! CLI / シンタックスハイライト機能は一切有効化せず、GFM のうち必要最小限の
//! `strikethrough` / `table` / `autolink` だけをオンにする。生 HTML は
//! comrak の既定設定（`render.r#unsafe = false`）で出力されないため、
//! Markdown 入力中の `<script>` 等はここで既に除去される。さらに生成物を
//! 必ず [`sanitize_epub_html`] に通すことで、許可リスト方式の二段構えで
//! XSS を防ぐ（リンクの `rel="noopener noreferrer"` もここで付与される）。

use crate::sanitize::sanitize_epub_html;

/// Markdown をサニタイズ済みの HTML 断片へ変換する純粋関数。
///
/// # 安全性
///
/// - 生 HTML は comrak の既定で無効（`<!-- raw HTML omitted -->` に置換）。
/// - 生成 HTML は [`sanitize_epub_html`] で許可リスト方式に絞り込み、
///   `script` / イベントハンドラ / `javascript:` スキーム等を除去する。
/// - リンクには `rel="noopener noreferrer"` が付与される。
pub fn render_markdown(md: &str) -> String {
    let mut options = comrak::Options::default();
    options.extension.strikethrough = true;
    options.extension.table = true;
    options.extension.autolink = true;

    let html = comrak::markdown_to_html(md, &options);
    sanitize_epub_html(&html)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn renders_heading() {
        let html = render_markdown("# Heading\n\nSome text.");
        assert_eq!(html, "<h1>Heading</h1>\n<p>Some text.</p>\n");
    }

    #[test]
    fn adds_noopener_noreferrer_to_links() {
        let html = render_markdown("[example](https://example.com)");
        assert!(
            html.contains("href=\"https://example.com\""),
            "link href missing: {html}"
        );
        assert!(
            html.contains("rel=\"noopener noreferrer\""),
            "link rel missing: {html}"
        );
    }

    #[test]
    fn removes_script_from_markdown_input() {
        let html = render_markdown("# Safe\n\n<script>alert('xss')</script>");
        assert!(html.contains("<h1>Safe</h1>"), "heading missing: {html}");
        assert!(!html.contains("<script"), "script tag survived: {html}");
        assert!(!html.contains("alert("), "script body survived: {html}");
    }

    #[test]
    fn removes_javascript_scheme_links() {
        let html = render_markdown("[click](javascript:alert(1))");
        assert!(
            !html.contains("javascript:"),
            "unsafe scheme survived: {html}"
        );
    }

    #[test]
    fn renders_gfm_table() {
        let html = render_markdown("| a | b |\n|---|---|\n| c | d |");
        assert!(html.contains("<table>"), "table missing: {html}");
        assert!(html.contains("<th>a</th>"), "table header missing: {html}");
        assert!(html.contains("<td>c</td>"), "table cell missing: {html}");
    }

    #[test]
    fn renders_strikethrough_and_autolink() {
        let html = render_markdown("~~gone~~ www.github.com");
        assert!(
            html.contains("<del>gone</del>"),
            "strikethrough missing: {html}"
        );
        assert!(
            html.contains("href=\"http://www.github.com\""),
            "autolink missing: {html}"
        );
        assert!(
            html.contains("rel=\"noopener noreferrer\""),
            "autolink rel missing: {html}"
        );
    }
}
