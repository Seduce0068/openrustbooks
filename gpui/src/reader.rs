//! GPUI reader view: a minimal native window that opens an EPUB via the
//! shared core crate and displays the current chapter as plain text.

use gpui::{
    App, ClickEvent, Context, FocusHandle, Focusable, IntoElement, KeyDownEvent, ParentElement,
    Render, Role, SharedString, Styled, Window, div, prelude::*, rgb,
};

use crate::book::{
    MAX_INPUT_BYTES, OPEN_ERROR, OpenedBook, chapter_label, next_index, previous_index,
};
use std::io::Read;

/// EPUB バイト列を上限付きで読む。core は入力長の制限を呼び出し側へ委譲する
/// （core/src/archive.rs:32-34）ため、ここで事前検査と読み取り量の制限を行う。
fn read_epub_bytes(path: &str) -> Result<Vec<u8>, &'static str> {
    let file = std::fs::File::open(path).map_err(|_| OPEN_ERROR)?;
    let metadata = file.metadata().map_err(|_| OPEN_ERROR)?;
    if !metadata.is_file() || metadata.len() > MAX_INPUT_BYTES {
        return Err(OPEN_ERROR);
    }
    let mut bytes = Vec::new();
    file.take(MAX_INPUT_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| OPEN_ERROR)?;
    if bytes.len() as u64 > MAX_INPUT_BYTES {
        return Err(OPEN_ERROR);
    }
    Ok(bytes)
}
pub struct ReaderView {
    path: SharedString,
    status: SharedString,
    book: Option<OpenedBook>,
    pub(crate) focus_handle: FocusHandle,
}

impl ReaderView {
    pub fn new(path: Option<String>, cx: &mut Context<Self>) -> Self {
        match path {
            None => Self {
                path: "（ファイル未指定）".into(),
                status: format!(
                    "起動引数または {} に EPUB のパスを指定してください",
                    crate::EPUB_PATH_ENV
                )
                .into(),
                book: None,
                focus_handle: cx.focus_handle(),
            },
            Some(path) => Self::open(path, cx),
        }
    }

    fn open(path: String, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        let status = match read_epub_bytes(&path) {
            Ok(bytes) => match OpenedBook::open(bytes) {
                Ok(book) => {
                    let status = format!(
                        "{} — 章 {}",
                        book.title(),
                        chapter_label(book.index(), book.total())
                    );
                    return Self {
                        path: path.into(),
                        status: status.into(),
                        book: Some(book),
                        focus_handle,
                    };
                }
                Err(error) => error,
            },
            Err(error) => error,
        };
        Self {
            path: path.into(),
            status: status.into(),
            book: None,
            focus_handle,
        }
    }

    fn move_to(&mut self, index: usize, cx: &mut Context<Self>) {
        let Some(book) = self.book.as_mut() else {
            return;
        };
        let index = if index >= book.total() {
            book.index()
        } else {
            index
        };
        if let Err(error) = book.goto(index) {
            self.status = error.into();
        } else {
            self.status = format!(
                "{} — 章 {}",
                book.title(),
                chapter_label(book.index(), book.total())
            )
            .into();
        }
        cx.notify();
    }
}

impl Focusable for ReaderView {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ReaderView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut body = div()
            .id(SharedString::from("reader-body"))
            .flex()
            .flex_1()
            .flex_col()
            .gap_2()
            .overflow_y_scroll();

        if let Some(book) = self.book.as_ref() {
            body = body
                .child(div().text_xl().child(book.title().to_string()))
                .child(div().text_sm().child(book.authors().to_string()))
                .child(div().text_lg().child(book.chapter_title().to_string()))
                .child(div().whitespace_normal().child(book.text().to_string()));
        } else {
            body = body.child(div().child("EPUB を開けませんでした。"));
        }

        let current = self.book.as_ref().map_or(0, OpenedBook::index);
        let total = self.book.as_ref().map_or(0, OpenedBook::total);

        let previous = div()
            .id(SharedString::from("previous-chapter"))
            .focusable()
            .tab_stop(true)
            .role(Role::Button)
            .aria_label(SharedString::from("前の章"))
            .px_3()
            .py_1()
            .bg(rgb(0x3a3a3a))
            .text_color(rgb(0xffffff))
            .cursor_pointer()
            .child("前の章")
            .on_click(cx.listener(move |view, _: &ClickEvent, _window, cx| {
                if let Some(book) = view.book.as_ref()
                    && book.total() > 1
                {
                    let index = previous_index(book.index());
                    view.move_to(index, cx);
                }
            }));

        let next = div()
            .id(SharedString::from("next-chapter"))
            .focusable()
            .tab_stop(true)
            .role(Role::Button)
            .aria_label(SharedString::from("次の章"))
            .px_3()
            .py_1()
            .bg(rgb(0x2e7d32))
            .text_color(rgb(0xffffff))
            .cursor_pointer()
            .child("次の章")
            .on_click(cx.listener(move |view, _: &ClickEvent, _window, cx| {
                if let Some(book) = view.book.as_ref()
                    && book.total() > 1
                {
                    let index = next_index(book.index(), book.total());
                    view.move_to(index, cx);
                }
            }));

        div()
            .id(SharedString::from("reader-root"))
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(move |view, event: &KeyDownEvent, _window, cx| {
                if event.keystroke.modifiers != gpui::Modifiers::none() {
                    return;
                }
                let Some(book) = view.book.as_ref() else {
                    return;
                };
                if book.total() <= 1 {
                    return;
                }
                match event.keystroke.key.as_str() {
                    "right" => view.move_to(next_index(book.index(), book.total()), cx),
                    "left" => view.move_to(previous_index(book.index()), cx),
                    _ => {}
                }
            }))
            .size_full()
            .flex()
            .flex_col()
            .gap_3()
            .p_4()
            .bg(rgb(0x1e1e1e))
            .text_color(rgb(0xe0e0e0))
            .child(div().text_sm().child(self.status.clone()))
            .child(div().text_xs().child(self.path.clone()))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_2()
                    .child(previous)
                    .child(format!("章 {}", chapter_label(current, total)))
                    .child(next),
            )
            .child(body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{Background, TestAppContext, rgb};
    use std::io::{Cursor, Write};
    use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

    #[gpui::test]
    fn reader_view_paints_expected_backgrounds(cx: &mut TestAppContext) {
        let (_view, cx) = cx.add_window_view(|_, cx| ReaderView::new(None, cx));
        let quads = cx.update(|window, _| window.painted_quads());

        assert!(
            quads
                .iter()
                .any(|quad| quad.background == Background::from(rgb(0x1e1e1e))),
            "ルート背景 0x1e1e1e が描画クワッドにありません"
        );
        assert!(
            quads
                .iter()
                .any(|quad| quad.background == Background::from(rgb(0x3a3a3a))),
            "前の章ボタン 0x3a3a3a が描画クワッドにありません"
        );
        assert!(
            quads
                .iter()
                .any(|quad| quad.background == Background::from(rgb(0x2e7d32))),
            "次の章ボタン 0x2e7d32 が描画クワッドにありません"
        );
    }

    #[gpui::test]
    fn reader_view_without_path_has_fixed_status(cx: &mut TestAppContext) {
        let (view, cx) = cx.add_window_view(|_, cx| ReaderView::new(None, cx));
        cx.update(|_, app| {
            let view = view.read(app);
            assert!(
                view.status.contains(crate::EPUB_PATH_ENV),
                "パス未指定時の案内に環境変数名が含まれていません: {}",
                view.status
            );
            assert!(view.book.is_none(), "パス未指定時に本が開かれています");
        });
    }

    const CONTAINER: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>
"#;

    /// spinal に 2 つの itemref を持つ最小の OPF。
    const OPF: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="pub-id">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:title>テスト本</dc:title>
    <dc:creator>Alice</dc:creator>
    <dc:language>ja</dc:language>
    <dc:identifier id="pub-id">urn:uuid:0000</dc:identifier>
  </metadata>
  <manifest>
    <item id="ch1" href="ch1.xhtml" media-type="application/xhtml+xml"/>
    <item id="ch2" href="ch2.xhtml" media-type="application/xhtml+xml"/>
  </manifest>
  <spine>
    <itemref idref="ch1"/>
    <itemref idref="ch2"/>
  </spine>
</package>
"#;

    const CH1: &str = r#"<html xmlns="http://www.w3.org/1999/xhtml"><head><title>第一章</title></head><body><p>第一章の本文</p></body></html>"#;
    const CH2: &str = r#"<html xmlns="http://www.w3.org/1999/xhtml"><head><title>第二章</title></head><body><p>第二章の本文</p></body></html>"#;

    /// mimetype を Stored で先頭に書き、他を Deflated で書いた 2 章 EPUB。
    fn two_chapter_epub() -> Vec<u8> {
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        writer
            .start_file(
                "mimetype",
                SimpleFileOptions::default().compression_method(CompressionMethod::Stored),
            )
            .unwrap();
        writer.write_all(b"application/epub+zip").unwrap();
        for (path, data) in [
            ("META-INF/container.xml", CONTAINER.as_bytes()),
            ("OEBPS/content.opf", OPF.as_bytes()),
            ("OEBPS/ch1.xhtml", CH1.as_bytes()),
            ("OEBPS/ch2.xhtml", CH2.as_bytes()),
        ] {
            writer
                .start_file(
                    path,
                    SimpleFileOptions::default().compression_method(CompressionMethod::Deflated),
                )
                .unwrap();
            writer.write_all(data).unwrap();
        }
        writer.finish().unwrap().into_inner()
    }

    impl ReaderView {
        /// バイト列から直接ビューを作るテストヘルパ（一時ファイルを使わない）。
        fn from_bytes(cx: &mut Context<Self>, bytes: Vec<u8>) -> Self {
            let book = OpenedBook::open(bytes).expect("テスト用 EPUB が開けませんでした");
            let status = format!(
                "{} — 章 {}",
                book.title(),
                chapter_label(book.index(), book.total())
            );
            Self {
                path: "test.epub".into(),
                status: status.into(),
                book: Some(book),
                focus_handle: cx.focus_handle(),
            }
        }
    }

    #[gpui::test]
    fn arrow_keys_move_chapter(cx: &mut TestAppContext) {
        let bytes = two_chapter_epub();
        let (view, cx) = cx.add_window_view(|_, cx| ReaderView::from_bytes(cx, bytes));
        let focus_handle = cx.update(|_, app| view.read(app).focus_handle.clone());
        cx.update(|window, app| window.focus(&focus_handle, app));

        let current_index = |cx: &mut gpui::VisualTestContext| {
            cx.update(|_, app| {
                view.read(app)
                    .book
                    .as_ref()
                    .expect("テスト用の本が開かれていません")
                    .index()
            })
        };

        assert_eq!(current_index(cx), 0);
        cx.simulate_keystrokes("right");
        assert_eq!(current_index(cx), 1);
        cx.simulate_keystrokes("right");
        assert_eq!(current_index(cx), 1);
        cx.simulate_keystrokes("left");
        assert_eq!(current_index(cx), 0);
    }

    #[test]
    fn oversized_epub_is_rejected_before_read() {
        let path = std::env::temp_dir().join(format!(
            "openrustbooks-gpui-oversized-{}.epub",
            std::process::id()
        ));
        let file = std::fs::File::create(&path).expect("一時ファイルを作成できませんでした");
        file.set_len(MAX_INPUT_BYTES + 1)
            .expect("スパースファイルのサイズを設定できませんでした");
        drop(file);

        let result = read_epub_bytes(path.to_str().expect("パスを変換できませんでした"));
        let _ = std::fs::remove_file(&path);
        assert_eq!(
            result.err(),
            Some(OPEN_ERROR),
            "上限超過が事前拒否されませんでした"
        );
    }
}
