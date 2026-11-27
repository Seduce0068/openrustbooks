//! openrustbooks GPUI viewer (prototype).
//!
//! Kept outside the root Cargo workspace so the existing Tauri/Android/PWA
//! gates and MSRV 1.95 stay untouched. See
//! docs/decisions/0354-gpui-viewer-prototype.md.

mod book;
mod reader;

use gpui::{App, Bounds, WindowBounds, WindowOptions, prelude::*, px, size};
use gpui_platform::application;

use reader::ReaderView;

/// Environment fallback for the EPUB path (same name as the CLI argument).
pub const EPUB_PATH_ENV: &str = "OPENRUSTBOOKS_EPUB";

fn main() {
    let path = std::env::args()
        .nth(1)
        .or_else(|| std::env::var(EPUB_PATH_ENV).ok());

    application().run(move |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(1000.), px(760.)), cx);
        let window = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar: Some(gpui::TitlebarOptions {
                        title: Some("openrustbooks — GPUI".into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                |_window, cx| cx.new(|cx| ReaderView::new(path.clone(), cx)),
            )
            .expect("open the GPUI reader window");
        window
            .update(cx, |view, window, cx| window.focus(&view.focus_handle, cx))
            .expect("focus the GPUI reader view");
        cx.activate(true);
    });
}
