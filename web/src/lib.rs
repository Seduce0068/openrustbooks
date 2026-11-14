//! openrustbooks-web: Web application (Wasm) for openrustbooks.
//!
//! This crate compiles to WebAssembly and provides the browser-based
//! version of the application using Leptos.

use leptos::html::Div;
use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::hooks::use_params_map;
use leptos_router::path;
#[cfg(target_arch = "wasm32")]
use openrustbooks_core::WasmEpubBook;
use openrustbooks_core::init;
use openrustbooks_core::sanitize::sanitize_epub_html;
use wasm_bindgen::prelude::*;

// Initialize panic hook and tracing for Wasm
#[wasm_bindgen(start)]
pub fn main() {
    init();
    leptos::logging::log!("openrustbooks-web starting...");

    // Mount the Leptos app
    mount_to_body(|| view! { <App/> });
}

/// Main application component
#[component]
fn App() -> impl IntoView {
    view! {
        <Router>
            <Routes fallback=|| view! { <p class="not-found">"Page not found"</p> }>
                <Route path=path!("/") view=HomePage/>
                <Route path=path!("/editor/:id") view=EditorPage/>
                <Route path=path!("/reader/:id") view=ReaderPage/>
                <Route path=path!("/settings") view=SettingsPage/>
                <Route path=path!("/about") view=AboutPage/>
            </Routes>
        </Router>
    }
}

/// Home page - document browser
#[component]
fn HomePage() -> impl IntoView {
    view! {
        <div class="app home-page">
            <header class="app-header">
                <h1>"openrustbooks"</h1>
                <nav>
                    <a href="/settings">"Settings"</a>
                    <a href="/about">"About"</a>
                </nav>
            </header>
            <main class="app-main">
                <section class="document-browser">
                    <h2>"Documents"</h2>
                    <div class="toolbar">
                        <button class="btn btn-primary">"New File"</button>
                        <button class="btn btn-secondary">"Open Folder"</button>
                        <button class="btn btn-secondary">"Open File"</button>
                    </div>
                    <div class="file-tree">
                        <FileTree/>
                    </div>
                </section>
            </main>
        </div>
    }
}

/// Extract the `id` path parameter of the current route.
fn route_doc_id() -> Signal<String> {
    let params = use_params_map();
    Signal::derive(move || params.get().get("id").unwrap_or_default())
}

/// Editor page
#[component]
fn EditorPage() -> impl IntoView {
    let doc_id = route_doc_id();

    view! {
        <div class="app editor-page">
            <header class="app-header">
                <h1>"Editor: " {move || doc_id.get()}</h1>
                <nav>
                    <a href="/">"← Back"</a>
                </nav>
            </header>
            <main class="app-main">
                <Editor doc_id=doc_id/>
            </main>
        </div>
    }
}

/// Reader page
#[component]
fn ReaderPage() -> impl IntoView {
    let doc_id = route_doc_id();

    view! {
        <div class="app reader-page">
            <header class="app-header">
                <h1>"Reader: " {move || doc_id.get()}</h1>
                <nav>
                    <a href="/">"← Back"</a>
                </nav>
            </header>
            <main class="app-main">
                <Reader doc_id=doc_id/>
            </main>
        </div>
    }
}

/// Settings page
#[component]
fn SettingsPage() -> impl IntoView {
    view! {
        <div class="app settings-page">
            <header class="app-header">
                <h1>"Settings"</h1>
                <nav>
                    <a href="/">"← Back"</a>
                </nav>
            </header>
            <main class="app-main">
                <SettingsForm/>
            </main>
        </div>
    }
}

/// About page
#[component]
fn AboutPage() -> impl IntoView {
    view! {
        <div class="app about-page">
            <header class="app-header">
                <h1>"About"</h1>
                <nav>
                    <a href="/">"← Back"</a>
                </nav>
            </header>
            <main class="app-main">
                <div class="about-content">
                    <h2>"openrustbooks"</h2>
                    <p>"Version: " {env!("CARGO_PKG_VERSION")}</p>
                    <p>"A Rust-powered document reader and editor"</p>
                    <p>"Supports: EPUB, PDF, Markdown, TXT, ZIP"</p>
                    <hr/>
                    <p>"Built with: Tauri, Leptos, Rust"</p>
                    <p>"License: MIT"</p>
                </div>
            </main>
        </div>
    }
}

/// File tree component
#[component]
fn FileTree() -> impl IntoView {
    view! {
        <ul class="file-tree">
            <li class="folder">
                <span class="folder-name">"📁 Documents"</span>
                <ul>
                    <li class="file">"📄 README.md"</li>
                    <li class="file">"📄 notes.txt"</li>
                    <li class="folder">
                        <span class="folder-name">"📁 Projects"</span>
                        <ul>
                            <li class="file">"📄 project1.md"</li>
                        </ul>
                    </li>
                </ul>
            </li>
        </ul>
    }
}

/// Editor component with CodeMirror
#[component]
fn Editor(doc_id: Signal<String>) -> impl IntoView {
    let (content, set_content) = signal(String::new());
    let (saving, set_saving) = signal(false);
    let editor_ref = NodeRef::<Div>::new();

    // Initialize CodeMirror once the editor pane is mounted.
    // CodeMirror 6 integration via wasm-bindgen will hook in here.
    Effect::new(move |_| {
        if editor_ref.get().is_some() {
            tracing::debug!("editor pane mounted for {}", doc_id.get());
        }
    });

    let save = move |_| {
        set_saving.set(true);
        // Save logic
        set_saving.set(false);
    };

    view! {
        <div class="editor-container">
            <div class="editor-toolbar">
                <button class="btn" on:click=save disabled=move || saving.get()>
                    {move || if saving.get() { "Saving..." } else { "Save" }}
                </button>
                <button class="btn">"AI Generate"</button>
                <button class="btn">"AI Refine"</button>
                <button class="btn">"Lint"</button>
                <select class="format-select">
                    <option value="markdown">"Markdown"</option>
                    <option value="txt">"Plain Text"</option>
                </select>
            </div>
            <div class="editor-pane" id="editor" node_ref=editor_ref></div>
            <div class="preview-pane">
                <Preview content=content set_content=set_content/>
            </div>
        </div>
    }
}

/// 章内のフラグメントへスクロールする。
#[cfg(target_arch = "wasm32")]
fn scroll_to_fragment(fragment: &str) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(document) = window.document() else {
        return;
    };
    let Some(element) = document.get_element_by_id(fragment) else {
        return;
    };
    element.scroll_into_view();
}

/// 本文のスクロール位置を先頭に戻す。
#[cfg(target_arch = "wasm32")]
fn scroll_content_to_top() {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(document) = window.document() else {
        return;
    };
    let Ok(Some(element)) = document.query_selector(".reader-content") else {
        return;
    };
    element.set_scroll_top(0);
}

/// 本文の現在のスクロール位置を返す。
#[cfg(target_arch = "wasm32")]
fn content_scroll_top() -> i32 {
    let Some(window) = web_sys::window() else {
        return 0;
    };
    let Some(document) = window.document() else {
        return 0;
    };
    let Ok(Some(element)) = document.query_selector(".reader-content") else {
        return 0;
    };
    element.scroll_top()
}

/// 本文のスクロール位置を設定する。
#[cfg(target_arch = "wasm32")]
fn set_content_scroll_top(offset: i32) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(document) = window.document() else {
        return;
    };
    let Ok(Some(element)) = document.query_selector(".reader-content") else {
        return;
    };
    element.set_scroll_top(offset);
}

/// 次の描画フレームで位置を適用し、実際の位置を保存する。
///
/// 本文の更新は非同期に DOM へ反映されるため、更新直後に
/// `getElementById` やスクロール位置の設定を行うと対象が古いことがある。
/// スクロール位置はレイアウトが終わるまで反映されないことがあるため、
/// 数フレームまで再試行する。
#[cfg(target_arch = "wasm32")]
fn apply_position_after_render(
    chapter_index: usize,
    fragment: Option<String>,
    scroll_top: Option<i32>,
    position_key: SharedPositionKey,
) {
    apply_position_frame(chapter_index, fragment, scroll_top, position_key, 0);
}

#[cfg(target_arch = "wasm32")]
fn apply_position_frame(
    chapter_index: usize,
    fragment: Option<String>,
    scroll_top: Option<i32>,
    position_key: SharedPositionKey,
    attempt: u32,
) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let callback = Closure::once_into_js(move || {
        match fragment.as_deref() {
            Some(fragment) => scroll_to_fragment(fragment),
            None => match scroll_top {
                Some(offset) if offset > 0 => set_content_scroll_top(offset),
                _ => scroll_content_to_top(),
            },
        }
        let actual = content_scroll_top();
        let wanted = scroll_top.unwrap_or(0);
        if fragment.is_none() && wanted > 0 && actual < wanted && attempt < 3 {
            apply_position_frame(
                chapter_index,
                fragment,
                scroll_top,
                position_key,
                attempt + 1,
            );
            return;
        }
        if let Some(key) = position_key.borrow().as_ref() {
            save_position(
                key,
                &SavedPosition {
                    chapter_index,
                    fragment,
                    scroll_top: actual,
                },
            );
        }
    });
    let _ = window.request_animation_frame(callback.unchecked_ref());
}

/// 画像の拡張子から MIME タイプを推定する。
#[cfg(target_arch = "wasm32")]
fn mime_for_path(path: &str) -> &'static str {
    let path = path.split(['#', '?']).next().unwrap_or(path);
    let extension = path
        .rsplit_once('.')
        .map(|(_, extension)| extension)
        .unwrap_or("");
    match extension.to_ascii_lowercase().as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "bmp" => "image/bmp",
        "avif" => "image/avif",
        _ => "application/octet-stream",
    }
}

/// バイト列から Blob URL を作る。呼び出し側が寿命を管理する。
#[cfg(target_arch = "wasm32")]
fn object_url_for(bytes: &[u8], href: &str) -> Result<String, JsValue> {
    let parts = js_sys::Array::new();
    parts.push(&js_sys::Uint8Array::from(bytes));
    let options = web_sys::BlobPropertyBag::new();
    options.set_type(mime_for_path(href));
    let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(parts.as_ref(), &options)?;
    web_sys::Url::create_object_url_with_blob(&blob)
}

/// サニタイズ済み HTML の画像を Blob URL に置き換える。
///
/// 解決できない画像は `src` を削除し、alt テキストだけを残す。
#[cfg(target_arch = "wasm32")]
fn resolve_images(
    book: &WasmEpubBook,
    index: usize,
    html: &str,
) -> Result<(String, Vec<String>), JsValue> {
    let parser = web_sys::DomParser::new()?;
    let document = parser.parse_from_string(html, web_sys::SupportedType::TextHtml)?;
    let Some(body) = document.body() else {
        return Ok((html.to_string(), Vec::new()));
    };
    let images = body.query_selector_all("img[src]")?;
    let mut urls = Vec::new();
    for position in 0..images.length() {
        let Some(node) = images.get(position) else {
            continue;
        };
        let Ok(image) = node.dyn_into::<web_sys::HtmlImageElement>() else {
            continue;
        };
        let Some(src) = web_sys::Element::get_attribute(&image, "src") else {
            continue;
        };
        match book.chapter_resource(index, src.clone()) {
            Ok(bytes) => match object_url_for(&bytes, &src) {
                Ok(url) => {
                    image.set_attribute("src", &url)?;
                    urls.push(url);
                }
                Err(_) => {
                    image.remove_attribute("src").ok();
                }
            },
            Err(_) => {
                image.remove_attribute("src").ok();
            }
        }
    }
    Ok((body.inner_html(), urls))
}

/// 読書位置を保存する localStorage のキーの接頭辞。
#[cfg(target_arch = "wasm32")]
const POSITION_KEY_PREFIX: &str = "openrustbooks:position:";

/// 読書位置のキーを複数のコールバックで共有するための型。
#[cfg(target_arch = "wasm32")]
type SharedPositionKey = std::rc::Rc<std::cell::RefCell<Option<String>>>;

/// localStorage に保存する読書位置。
#[cfg(target_arch = "wasm32")]
#[derive(serde::Deserialize, serde::Serialize, Clone)]
struct SavedPosition {
    #[serde(rename = "chapterIndex")]
    chapter_index: usize,
    fragment: Option<String>,
    /// 章内のスクロール位置（px）。旧形式の保存値には無いので 0 とする。
    #[serde(rename = "scrollTop", default)]
    scroll_top: i32,
}

/// EPUB ファイルから読書位置のキーを作る。
#[cfg(target_arch = "wasm32")]
fn position_key(file: &web_sys::File) -> String {
    format!(
        "{POSITION_KEY_PREFIX}{}:{}:{}",
        file.name(),
        file.size(),
        file.last_modified()
    )
}

/// 保存された読書位置を読む。localStorage が使えない場合は `None`。
#[cfg(target_arch = "wasm32")]
fn load_position(key: &str) -> Option<SavedPosition> {
    let storage = web_sys::window()?.local_storage().ok()??;
    let raw = storage.get_item(key).ok()??;
    serde_json::from_str(&raw).ok()
}

/// 読書位置を保存する。localStorage が使えない場合は何もしない。
#[cfg(target_arch = "wasm32")]
fn save_position(key: &str, position: &SavedPosition) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Ok(Some(storage)) = window.local_storage() else {
        return;
    };
    if let Ok(raw) = serde_json::to_string(position) {
        let _ = storage.set_item(key, &raw);
    }
}

/// Reader component
#[cfg(target_arch = "wasm32")]
#[component]
fn Reader(doc_id: Signal<String>) -> impl IntoView {
    use js_sys::Uint8Array;
    use serde::Deserialize;
    use std::{
        cell::{Cell, RefCell},
        collections::HashMap,
        rc::Rc,
    };
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::{JsFuture, spawn_local};
    use web_sys::{Event, HtmlInputElement};

    /// 読み込む EPUB のサイズ上限（core の `ArchiveLimits::max_total_size` と同じ）。
    const MAX_EPUB_FILE_MIB: u64 = 256;
    const MAX_EPUB_FILE_BYTES: f64 = MAX_EPUB_FILE_MIB as f64 * 1024.0 * 1024.0;

    #[derive(Deserialize)]
    struct TocEntry {
        title: String,
        href: String,
        fragment: Option<String>,
        depth: usize,
    }

    #[derive(Deserialize)]
    struct ChapterLink {
        #[serde(rename = "chapterIndex")]
        chapter_index: usize,
        fragment: Option<String>,
    }

    #[allow(dead_code)]
    #[derive(Deserialize)]
    struct CoverImage {
        id: String,
        href: String,
        media_type: String,
        bytes: Vec<u8>,
    }

    let book = Rc::new(RefCell::new(None::<WasmEpubBook>));
    let object_urls = Rc::new(RefCell::new(Vec::<String>::new()));
    let cover_url_state = Rc::new(RefCell::new(None::<String>));
    let position_key_state = Rc::new(RefCell::new(None::<String>));
    let (chapters, set_chapters) = signal(Vec::<(usize, String, Option<String>)>::new());
    let (current, set_current) = signal(0_usize);
    let (content, set_content) = signal(String::new());
    let (status, set_status) = signal(String::from("Choose an EPUB file to begin."));
    let (cover_url, set_cover_url) = signal(None::<String>);

    let render_chapter: Rc<dyn Fn(usize, Option<String>, Option<i32>)> = {
        let book = Rc::clone(&book);
        let object_urls = Rc::clone(&object_urls);
        let position_key = Rc::clone(&position_key_state);
        Rc::new(move |index, fragment, scroll_top| {
            let result = book
                .borrow()
                .as_ref()
                .ok_or_else(|| String::from("No EPUB is open."))
                .and_then(|book| {
                    let html = book
                        .chapter_html(index)
                        .map_err(|error| format!("{error:?}"))?;
                    let safe = sanitize_epub_html(&html);
                    resolve_images(book, index, &safe).map_err(|error| format!("{error:?}"))
                });
            match result {
                Ok((html, urls)) => {
                    for url in object_urls.borrow_mut().drain(..) {
                        let _ = web_sys::Url::revoke_object_url(&url);
                    }
                    *object_urls.borrow_mut() = urls;
                    set_current.set(index);
                    set_content.set(html);
                    set_status.set(String::new());
                    apply_position_after_render(
                        index,
                        fragment,
                        scroll_top,
                        Rc::clone(&position_key),
                    );
                }
                Err(error) => set_status.set(error),
            }
        })
    };

    let on_file_change = {
        let book = Rc::clone(&book);
        let object_urls = Rc::clone(&object_urls);
        let cover_url_state = Rc::clone(&cover_url_state);
        let position_key_state = Rc::clone(&position_key_state);
        let render_chapter = Rc::clone(&render_chapter);
        move |event: Event| {
            let Some(input) = event
                .target()
                .and_then(|target| target.dyn_into::<HtmlInputElement>().ok())
            else {
                set_status.set("Unable to read the file input.".to_string());
                return;
            };
            let Some(file) = input.files().and_then(|files| files.item(0)) else {
                return;
            };
            if file.size() > MAX_EPUB_FILE_BYTES {
                set_status.set(format!(
                    "This EPUB is too large (limit: {MAX_EPUB_FILE_MIB} MiB)."
                ));
                return;
            }
            let key = position_key(&file);
            *position_key_state.borrow_mut() = Some(key.clone());
            set_status.set(format!("Opening {}…", file.name()));
            set_content.set(String::new());
            set_chapters.set(Vec::new());
            *book.borrow_mut() = None;
            for url in object_urls.borrow_mut().drain(..) {
                let _ = web_sys::Url::revoke_object_url(&url);
            }
            if let Some(url) = cover_url_state.borrow_mut().take() {
                let _ = web_sys::Url::revoke_object_url(&url);
            }
            set_cover_url.set(None);

            let book = Rc::clone(&book);
            let cover_url_state = Rc::clone(&cover_url_state);
            let render_chapter = Rc::clone(&render_chapter);
            spawn_local(async move {
                let buffer = match JsFuture::from(file.array_buffer()).await {
                    Ok(buffer) => buffer,
                    Err(error) => {
                        set_status.set(format!("File read failed: {error:?}"));
                        return;
                    }
                };
                let bytes = Uint8Array::new(&buffer).to_vec();
                let opened = match WasmEpubBook::new(bytes) {
                    Ok(opened) => opened,
                    Err(error) => {
                        set_status.set(format!("EPUB open failed: {error:?}"));
                        return;
                    }
                };

                let count = opened.chapter_count();
                if count == 0 {
                    set_status.set("This EPUB has no readable chapters.".to_string());
                    return;
                }
                let mut spine = Vec::with_capacity(count);
                for index in 0..count {
                    match opened.chapter_href(index) {
                        Ok(href) => spine.push((index, href)),
                        Err(error) => {
                            set_status.set(format!("Chapter list failed: {error:?}"));
                            return;
                        }
                    }
                }

                let toc = opened
                    .toc()
                    .ok()
                    .and_then(|value| serde_wasm_bindgen::from_value::<Vec<TocEntry>>(value).ok())
                    .unwrap_or_default();
                let mut toc_entries = HashMap::new();
                for entry in toc {
                    toc_entries.entry(entry.href).or_insert((
                        entry.title,
                        entry.fragment,
                        entry.depth,
                    ));
                }
                let options: Vec<(usize, String, Option<String>)> = spine
                    .into_iter()
                    .map(|(index, href)| {
                        let (title, fragment, depth) =
                            toc_entries.remove(&href).unwrap_or((href, None, 0));
                        let indent = "　".repeat(depth.min(8));
                        (index, format!("{indent}{title}"), fragment)
                    })
                    .collect();

                let cover = opened
                    .cover_image()
                    .ok()
                    .and_then(|value| {
                        serde_wasm_bindgen::from_value::<Option<CoverImage>>(value).ok()
                    })
                    .flatten()
                    .and_then(|cover| object_url_for(&cover.bytes, &cover.href).ok());
                if let Some(url) = cover {
                    *cover_url_state.borrow_mut() = Some(url.clone());
                    set_cover_url.set(Some(url));
                }

                let saved = load_position(&key).filter(|position| position.chapter_index < count);
                let (start_index, start_fragment, start_scroll) = saved
                    .map(|position| {
                        (
                            position.chapter_index,
                            position.fragment,
                            position.scroll_top,
                        )
                    })
                    .unwrap_or((0, None, 0));

                *book.borrow_mut() = Some(opened);
                set_chapters.set(options);
                render_chapter(start_index, start_fragment, Some(start_scroll));
            });
        }
    };

    let on_select = {
        let render_chapter = Rc::clone(&render_chapter);
        move |event: Event| {
            if let Some(input) = event
                .target()
                .and_then(|target| target.dyn_into::<HtmlInputElement>().ok())
                && let Ok(index) = input.value().parse::<usize>()
            {
                let fragment = chapters
                    .get()
                    .iter()
                    .find(|(chapter, _, _)| *chapter == index)
                    .and_then(|(_, _, fragment)| fragment.clone());
                render_chapter(index, fragment, None);
            }
        }
    };
    let on_previous = {
        let render_chapter = Rc::clone(&render_chapter);
        move |_| {
            let index = current.get();
            if index > 0 {
                let fragment = chapters
                    .get()
                    .iter()
                    .find(|(chapter, _, _)| *chapter == index - 1)
                    .and_then(|(_, _, fragment)| fragment.clone());
                render_chapter(index - 1, fragment, None);
            }
        }
    };
    let on_next = {
        let render_chapter = Rc::clone(&render_chapter);
        move |_| {
            let index = current.get();
            if chapters
                .get()
                .iter()
                .any(|(chapter, _, _)| *chapter > index)
            {
                let fragment = chapters
                    .get()
                    .iter()
                    .find(|(chapter, _, _)| *chapter == index + 1)
                    .and_then(|(_, _, fragment)| fragment.clone());
                render_chapter(index + 1, fragment, None);
            }
        }
    };
    let on_content_click = {
        let book = Rc::clone(&book);
        let render_chapter = Rc::clone(&render_chapter);
        let position_key = Rc::clone(&position_key_state);
        move |event: web_sys::MouseEvent| {
            let Some(target) = event
                .target()
                .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
            else {
                return;
            };
            let Ok(Some(anchor)) = target.closest("a") else {
                return;
            };
            let Some(href) = anchor.get_attribute("href") else {
                return;
            };
            event.prevent_default();
            if let Some(fragment) = href.strip_prefix('#') {
                scroll_to_fragment(fragment);
                if let Some(key) = position_key.borrow().as_ref() {
                    save_position(
                        key,
                        &SavedPosition {
                            chapter_index: current.get(),
                            fragment: Some(fragment.to_string()),
                            scroll_top: content_scroll_top(),
                        },
                    );
                }
                return;
            }
            if href.starts_with("http://")
                || href.starts_with("https://")
                || href.starts_with("mailto:")
            {
                if let Some(window) = web_sys::window() {
                    let _ = window.open_with_url_and_target_and_features(
                        &href,
                        "_blank",
                        "noopener,noreferrer",
                    );
                }
                return;
            }
            let resolved = {
                let book = book.borrow();
                let Some(book) = book.as_ref() else {
                    return;
                };
                book.resolve_chapter_link(current.get(), href)
            };
            let link = match resolved {
                Ok(value) if !value.is_null() => {
                    serde_wasm_bindgen::from_value::<ChapterLink>(value).ok()
                }
                Ok(_) => None,
                Err(error) => {
                    set_status.set(format!("Link resolution failed: {error:?}"));
                    return;
                }
            };
            let Some(link) = link else {
                set_status.set("This link cannot be opened.".to_string());
                return;
            };
            if link.chapter_index == current.get() {
                let saved_fragment = link.fragment.clone();
                match link.fragment {
                    Some(fragment) => scroll_to_fragment(&fragment),
                    None => scroll_content_to_top(),
                }
                if let Some(key) = position_key.borrow().as_ref() {
                    save_position(
                        key,
                        &SavedPosition {
                            chapter_index: link.chapter_index,
                            fragment: saved_fragment,
                            scroll_top: content_scroll_top(),
                        },
                    );
                }
            } else {
                render_chapter(link.chapter_index, link.fragment, None);
            }
        }
    };
    // スクロール位置は 200ms のデバウンスで保存する。
    let scroll_save_timer = Rc::new(Cell::new(None::<i32>));
    let on_scroll = {
        let position_key = Rc::clone(&position_key_state);
        let scroll_save_timer = Rc::clone(&scroll_save_timer);
        move |_event: web_sys::Event| {
            let Some(window) = web_sys::window() else {
                return;
            };
            if let Some(handle) = scroll_save_timer.take() {
                window.clear_timeout_with_handle(handle);
            }
            let position_key = Rc::clone(&position_key);
            let timer = Rc::clone(&scroll_save_timer);
            let chapter_index = current.get();
            let callback = Closure::once_into_js(move || {
                timer.set(None);
                // 章が切り替わった後の遅延保存は書き込まない。
                if current.get() != chapter_index {
                    return;
                }
                let scroll_top = content_scroll_top();
                if let Some(key) = position_key.borrow().as_ref() {
                    save_position(
                        key,
                        &SavedPosition {
                            chapter_index,
                            fragment: None,
                            scroll_top,
                        },
                    );
                }
            });
            if let Ok(handle) = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                callback.unchecked_ref(),
                200,
            ) {
                scroll_save_timer.set(Some(handle));
            }
        }
    };

    let current_title = Memo::new(move |_| {
        chapters
            .get()
            .iter()
            .find(|(chapter, _, _)| *chapter == current.get())
            .map(|(_, title, _)| title.clone())
            .unwrap_or_default()
    });

    view! {
        <div class="reader-container" data-doc-id=move || doc_id.get()>
            <div class="reader-toolbar">
                <label class="btn btn-primary" for="epub-file">"Open EPUB"</label>
                <input id="epub-file" class="reader-file-input" type="file" accept=".epub,application/epub+zip" on:change=on_file_change/>
                <button class="btn" on:click=on_previous disabled=move || current.get() == 0>"◀"</button>
                <select class="reader-chapter-select" on:change=on_select prop:disabled=move || chapters.get().is_empty()>
                    <For
                        each=move || chapters.get()
                        key=|(index, _, _)| *index
                        children=move |(index, title, _)| view! {
                            <option value=index.to_string() prop:selected=move || current.get() == index>
                                {title}
                            </option>
                        }
                    />
                </select>
                <button class="btn" on:click=on_next disabled=move || {
                    let index = current.get();
                    !chapters.get().iter().any(|(chapter, _, _)| *chapter > index)
                }>
                    "▶"
                </button>
                <span class="reader-chapter-title">{move || current_title.get()}</span>
            </div>
            <p class="reader-status" role="status">{move || status.get()}</p>
            <div class="reader-body">
                <Show when=move || cover_url.get().is_some()>
                    <aside class="reader-sidebar">
                        <img class="reader-cover" src=move || cover_url.get().unwrap_or_default() alt="Cover"/>
                    </aside>
                </Show>
                <div class="reader-content" on:click=on_content_click on:scroll=on_scroll>
                    <div class="document-content reader-html" inner_html=move || content.get()></div>
                </div>
            </div>
        </div>
    }
}

/// ネイティブの型検査用 Reader。実際のファイル読込は Wasm ターゲット専用。
#[cfg(not(target_arch = "wasm32"))]
#[component]
fn Reader(doc_id: Signal<String>) -> impl IntoView {
    view! {
        <div class="reader-container" data-doc-id=move || doc_id.get()>
            <p>"EPUB reading is available in the browser build."</p>
        </div>
    }
}

/// Preview component for Markdown
#[component]
fn Preview(content: ReadSignal<String>, set_content: WriteSignal<String>) -> impl IntoView {
    let _ = set_content;
    view! {
        <div class="preview" inner_html=move || sanitize_epub_html(&content.get())/>
    }
}

/// Settings form component
#[component]
fn SettingsForm() -> impl IntoView {
    view! {
        <form class="settings-form">
            <section>
                <h3>"AI Settings"</h3>
                <label>
                    <input type="checkbox"/> "Enable AI Features"
                </label>
                <label>
                    "Provider: "
                    <select>
                        <option value="openai">"OpenAI"</option>
                        <option value="ollama">"Ollama (Local)"</option>
                    </select>
                </label>
                <label>
                    "Model: "
                    <input type="text" placeholder="gpt-4o-mini"/>
                </label>
                <label>
                    "API Key: "
                    <input type="password" placeholder="sk-..."/>
                </label>
            </section>
            <section>
                <h3>"Editor Settings"</h3>
                <label>
                    "Font Size: "
                    <input type="number" value="14" min="10" max="24"/>
                </label>
                <label>
                    <input type="checkbox"/> "Line Numbers"
                </label>
                <label>
                    <input type="checkbox"/> "Word Wrap"
                </label>
                <label>
                    <input type="checkbox"/> "Vim Mode"
                </label>
            </section>
            <section>
                <h3>"Lint Settings"</h3>
                <label>
                    <input type="checkbox"/> "Enable Linting"
                </label>
                <label>
                    <input type="checkbox"/> "Auto-fix on Save"
                </label>
            </section>
            <button type="submit" class="btn btn-primary">"Save Settings"</button>
        </form>
    }
}
