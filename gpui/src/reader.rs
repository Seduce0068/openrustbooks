//! GPUI reader view: a minimal native window that opens an EPUB via the
//! shared core crate and displays the current chapter as plain text.

use gpui::{
    ClickEvent, Context, IntoElement, ParentElement, Render, Role, SharedString, Styled, Window,
    div, prelude::*, rgb,
};

use crate::book::{OPEN_ERROR, OpenedBook, chapter_label, next_index, previous_index};

pub struct ReaderView {
    path: SharedString,
    status: SharedString,
    book: Option<OpenedBook>,
}

impl ReaderView {
    pub fn new(path: Option<String>) -> Self {
        match path {
            None => Self {
                path: "（ファイル未指定）".into(),
                status: format!(
                    "起動引数または {} に EPUB のパスを指定してください",
                    crate::EPUB_PATH_ENV
                )
                .into(),
                book: None,
            },
            Some(path) => Self::open(path),
        }
    }

    fn open(path: String) -> Self {
        let status = match std::fs::read(&path) {
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
                    };
                }
                Err(error) => error,
            },
            Err(_) => OPEN_ERROR,
        };
        Self {
            path: path.into(),
            status: status.into(),
            book: None,
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

        let can_navigate = self.book.as_ref().is_some_and(|book| book.total() > 1);
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
                if can_navigate {
                    view.move_to(previous_index(current), cx);
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
                if can_navigate {
                    view.move_to(next_index(current, total), cx);
                }
            }));

        div()
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

    #[gpui::test]
    fn reader_view_paints_expected_backgrounds(cx: &mut TestAppContext) {
        let (_view, cx) = cx.add_window_view(|_, _| ReaderView::new(None));
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
        let (view, cx) = cx.add_window_view(|_, _| ReaderView::new(None));
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
}
