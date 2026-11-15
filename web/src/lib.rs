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

/// 画像の読み込み待ちの上限（ミリ秒）。
#[cfg(target_arch = "wasm32")]
const IMAGE_SETTLE_MS: f64 = 2000.0;

/// 本文の画像が DOM に現れ、すべて読み込みを終えたか判定する。
///
/// `expected` は設定した HTML に含まれる `img` の数。本文の更新は非同期に
/// DOM へ反映されるため、数が揃うまでは未確定として扱う。`src` が無い画像
/// （解決できず除去したもの）は待たない。
#[cfg(target_arch = "wasm32")]
fn images_settled(expected: usize) -> bool {
    let Some(window) = web_sys::window() else {
        return true;
    };
    let Some(document) = window.document() else {
        return true;
    };
    let Ok(images) = document.query_selector_all(".reader-html img") else {
        return expected == 0;
    };
    let mut with_source = 0_usize;
    for index in 0..images.length() {
        let Some(node) = images.get(index) else {
            continue;
        };
        let Ok(image) = node.dyn_into::<web_sys::HtmlImageElement>() else {
            continue;
        };
        if image.get_attribute("src").is_none() {
            continue;
        }
        with_source += 1;
        if !image.complete() {
            return false;
        }
    }
    with_source >= expected
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
    expected_images: usize,
    position_key: SharedPositionKey,
) {
    let deadline = now_millis() + IMAGE_SETTLE_MS;
    apply_position_frame(
        chapter_index,
        fragment,
        scroll_top,
        expected_images,
        position_key,
        0,
        deadline,
    );
}

#[cfg(target_arch = "wasm32")]
fn apply_position_frame(
    chapter_index: usize,
    fragment: Option<String>,
    scroll_top: Option<i32>,
    expected_images: usize,
    position_key: SharedPositionKey,
    attempt: u32,
    deadline: f64,
) {
    let Some(window) = web_sys::window() else {
        return;
    };
    // 画像の読み込みで本文の高さが変わると、同じ scrollTop が別の内容を指す。
    // 画像が DOM に現れて確定するまで（上限付きで）位置の適用を待つ。
    if !images_settled(expected_images) && now_millis() < deadline {
        let callback = Closure::once_into_js(move || {
            apply_position_frame(
                chapter_index,
                fragment,
                scroll_top,
                expected_images,
                position_key,
                attempt,
                deadline,
            );
        });
        let _ = window.request_animation_frame(callback.unchecked_ref());
        return;
    }
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
                expected_images,
                position_key,
                attempt + 1,
                deadline,
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
                    saved_at: 0.0,
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

/// 保持する読書位置の最大件数。
#[cfg(target_arch = "wasm32")]
const POSITION_LIMIT: usize = 40;

/// 読書位置の寿命（日）。
#[cfg(target_arch = "wasm32")]
const POSITION_MAX_AGE_DAYS: f64 = 90.0;

/// 容量超過時に残す件数。
#[cfg(target_arch = "wasm32")]
const POSITION_MIN_KEEP: usize = 10;

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
    /// 保存した時刻（Unix ミリ秒）。旧形式の保存値には無いので 0 とする。
    #[serde(rename = "savedAt", default)]
    saved_at: f64,
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

/// 現在時刻（Unix ミリ秒）を返す。
#[cfg(target_arch = "wasm32")]
fn now_millis() -> f64 {
    js_sys::Date::now()
}

/// `openrustbooks:position:` のキーと保存時刻（Unix ミリ秒）を列挙する。
#[cfg(target_arch = "wasm32")]
fn list_positions(storage: &web_sys::Storage) -> Vec<(String, f64)> {
    let Ok(length) = storage.length() else {
        return Vec::new();
    };
    let mut entries = Vec::new();
    for index in 0..length {
        let Ok(Some(key)) = storage.key(index) else {
            continue;
        };
        if !key.starts_with(POSITION_KEY_PREFIX) {
            continue;
        }
        let saved_at = storage
            .get_item(&key)
            .ok()
            .flatten()
            .and_then(|raw| serde_json::from_str::<SavedPosition>(&raw).ok())
            .map(|position| position.saved_at)
            .unwrap_or(0.0);
        entries.push((key, saved_at));
    }
    entries
}

/// 保存時刻の新しい順に並べ替える（`savedAt` が 0 の旧形式は最も古い扱い）。
#[cfg(target_arch = "wasm32")]
fn sort_newest_first(entries: &mut [(String, f64)]) {
    entries.sort_by(|left, right| {
        let left_time = if left.1 > 0.0 {
            left.1
        } else {
            f64::NEG_INFINITY
        };
        let right_time = if right.1 > 0.0 {
            right.1
        } else {
            f64::NEG_INFINITY
        };
        right_time
            .partial_cmp(&left_time)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}

/// 読書位置を掃除する。
///
/// 寿命（90 日）を過ぎたエントリを削除し、件数が上限（40）を超えていれば
/// 古い順に削除する。`keep` に渡したキー（現在開いている本）は必ず残す。
/// 読書位置以外のキーには触らない。
#[cfg(target_arch = "wasm32")]
fn prune_positions(storage: &web_sys::Storage, keep: &str) {
    let cutoff = now_millis() - POSITION_MAX_AGE_DAYS * 24.0 * 60.0 * 60.0 * 1000.0;
    for (key, saved_at) in list_positions(storage) {
        if key == keep {
            continue;
        }
        // `savedAt` が 0 の旧形式は寿命判定の対象にしない（件数上限で扱う）。
        if saved_at > 0.0 && saved_at < cutoff {
            let _ = storage.remove_item(&key);
        }
    }

    let remaining = list_positions(storage);
    if remaining.len() <= POSITION_LIMIT {
        return;
    }
    let has_current = remaining.iter().any(|(key, _)| key == keep);
    let budget = if has_current {
        POSITION_LIMIT.saturating_sub(1)
    } else {
        POSITION_LIMIT
    };
    let mut others: Vec<(String, f64)> = remaining
        .into_iter()
        .filter(|(key, _)| key != keep)
        .collect();
    sort_newest_first(&mut others);
    for (key, _) in others.into_iter().skip(budget) {
        let _ = storage.remove_item(&key);
    }
}

/// 読書位置を保存する。localStorage が使えない場合は何もしない。
///
/// 保存のたびに古いエントリを掃除する。容量超過で失敗した場合は件数を
/// 減らして再試行し、それでも失敗したら諦めて読書を妨げない。
#[cfg(target_arch = "wasm32")]
fn save_position(key: &str, position: &SavedPosition) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Ok(Some(storage)) = window.local_storage() else {
        return;
    };
    let position = SavedPosition {
        saved_at: now_millis(),
        ..position.clone()
    };
    let Ok(raw) = serde_json::to_string(&position) else {
        return;
    };
    if storage.set_item(key, &raw).is_ok() {
        prune_positions(&storage, key);
        return;
    }

    // 容量超過。古い順に減らして再試行する。
    for keep in [POSITION_MIN_KEEP, 1] {
        let mut entries: Vec<(String, f64)> = list_positions(&storage)
            .into_iter()
            .filter(|(entry_key, _)| entry_key != key)
            .collect();
        sort_newest_first(&mut entries);
        for (old_key, _) in entries.into_iter().skip(keep) {
            let _ = storage.remove_item(&old_key);
        }
        if storage.set_item(key, &raw).is_ok() {
            return;
        }
    }
}

/// 表示設定の localStorage キー。
#[cfg(target_arch = "wasm32")]
const SETTINGS_KEY: &str = "openrustbooks:settings";

/// 文字サイズの段階（rem）。
#[cfg(target_arch = "wasm32")]
const FONT_SCALES: [f64; 5] = [1.0, 1.125, 1.25, 1.5, 1.75];

/// 行間の段階。
#[cfg(target_arch = "wasm32")]
const LINE_HEIGHTS: [f64; 3] = [1.5, 1.8, 2.1];

/// 既定の文字サイズの段階。
#[cfg(target_arch = "wasm32")]
const DEFAULT_FONT_SCALE: usize = 1;

/// 既定の行間の段階。
#[cfg(target_arch = "wasm32")]
const DEFAULT_LINE_HEIGHT: usize = 1;

/// リーダーの表示設定。
#[cfg(target_arch = "wasm32")]
#[derive(serde::Deserialize, serde::Serialize, Clone, Copy, PartialEq)]
struct ReaderSettings {
    #[serde(rename = "fontScale", default = "default_font_scale")]
    font_scale: usize,
    #[serde(rename = "lineHeight", default = "default_line_height")]
    line_height: usize,
    #[serde(default)]
    dark: bool,
}

#[cfg(target_arch = "wasm32")]
fn default_font_scale() -> usize {
    DEFAULT_FONT_SCALE
}

#[cfg(target_arch = "wasm32")]
fn default_line_height() -> usize {
    DEFAULT_LINE_HEIGHT
}

#[cfg(target_arch = "wasm32")]
impl Default for ReaderSettings {
    fn default() -> Self {
        Self {
            font_scale: DEFAULT_FONT_SCALE,
            line_height: DEFAULT_LINE_HEIGHT,
            dark: false,
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl ReaderSettings {
    /// 範囲外の値を既定へ戻す。
    fn normalized(self) -> Self {
        Self {
            font_scale: if self.font_scale < FONT_SCALES.len() {
                self.font_scale
            } else {
                DEFAULT_FONT_SCALE
            },
            line_height: if self.line_height < LINE_HEIGHTS.len() {
                self.line_height
            } else {
                DEFAULT_LINE_HEIGHT
            },
            dark: self.dark,
        }
    }
}

/// 表示設定を読み込む。localStorage が使えない・壊れている場合は既定。
#[cfg(target_arch = "wasm32")]
fn load_settings() -> ReaderSettings {
    let Some(window) = web_sys::window() else {
        return ReaderSettings::default();
    };
    let Ok(Some(storage)) = window.local_storage() else {
        return ReaderSettings::default();
    };
    let Ok(Some(raw)) = storage.get_item(SETTINGS_KEY) else {
        return ReaderSettings::default();
    };
    serde_json::from_str::<ReaderSettings>(&raw)
        .map(ReaderSettings::normalized)
        .unwrap_or_default()
}

/// 表示設定を保存する。localStorage が使えない場合は何もしない。
#[cfg(target_arch = "wasm32")]
fn save_settings(settings: &ReaderSettings) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Ok(Some(storage)) = window.local_storage() else {
        return;
    };
    if let Ok(raw) = serde_json::to_string(settings) {
        let _ = storage.set_item(SETTINGS_KEY, &raw);
    }
}

/// `<html>` に dark クラスを付け外しする。
#[cfg(target_arch = "wasm32")]
fn apply_dark_class(dark: bool) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(document) = window.document() else {
        return;
    };
    let Some(element) = document.document_element() else {
        return;
    };
    if dark {
        let _ = element.class_list().add_1("dark");
    } else {
        let _ = element.class_list().remove_1("dark");
    }
}

/// 保存した読書位置をすべて削除し、削除した件数を返す。
#[cfg(target_arch = "wasm32")]
fn clear_reading_positions() -> usize {
    let Some(window) = web_sys::window() else {
        return 0;
    };
    let Ok(Some(storage)) = window.local_storage() else {
        return 0;
    };
    let Ok(length) = storage.length() else {
        return 0;
    };
    let mut keys = Vec::new();
    for index in 0..length {
        if let Ok(Some(key)) = storage.key(index)
            && key.starts_with(POSITION_KEY_PREFIX)
        {
            keys.push(key);
        }
    }
    let mut removed = 0;
    for key in keys {
        if storage.remove_item(&key).is_ok() {
            removed += 1;
        }
    }
    removed
}

/// 読書位置を JSON としてダウンロードする。
#[cfg(target_arch = "wasm32")]
fn export_positions() -> usize {
    let Some(window) = web_sys::window() else {
        return 0;
    };
    let Ok(Some(storage)) = window.local_storage() else {
        return 0;
    };
    let Ok(length) = storage.length() else {
        return 0;
    };
    let mut positions = serde_json::Map::new();
    for index in 0..length {
        let Ok(Some(key)) = storage.key(index) else {
            continue;
        };
        if !key.starts_with(POSITION_KEY_PREFIX) {
            continue;
        }
        if let Ok(Some(value)) = storage.get_item(&key) {
            positions.insert(key, serde_json::Value::String(value));
        }
    }
    let count = positions.len();

    // 表示設定も含める。
    let settings = storage
        .get_item(SETTINGS_KEY)
        .ok()
        .flatten()
        .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok());

    let mut root = serde_json::Map::new();
    root.insert(
        "positions".to_string(),
        serde_json::Value::Object(positions),
    );
    root.insert(
        "settings".to_string(),
        settings.unwrap_or(serde_json::Value::Null),
    );

    let Ok(raw) = serde_json::to_string_pretty(&root) else {
        return 0;
    };
    // Blob を作ってダウンロードリンクをクリックする。
    let parts = js_sys::Array::new();
    parts.push(&JsValue::from_str(&raw));
    let options = web_sys::BlobPropertyBag::new();
    options.set_type("application/json");
    let Ok(blob) = web_sys::Blob::new_with_str_sequence_and_options(parts.as_ref(), &options)
    else {
        return 0;
    };
    let Ok(url) = web_sys::Url::create_object_url_with_blob(&blob) else {
        return 0;
    };
    let Some(document) = window.document() else {
        let _ = web_sys::Url::revoke_object_url(&url);
        return 0;
    };
    let Ok(anchor) = document.create_element("a") else {
        let _ = web_sys::Url::revoke_object_url(&url);
        return 0;
    };
    let _ = anchor.set_attribute("href", &url);
    let _ = anchor.set_attribute("download", "openrustbooks-positions.json");
    if let Some(element) = anchor.dyn_ref::<web_sys::HtmlElement>() {
        element.click();
    }
    // 少し待ってから URL を解放する。
    let url_for_cleanup = url.clone();
    let callback = Closure::once_into_js(move || {
        let _ = web_sys::Url::revoke_object_url(&url_for_cleanup);
    });
    let _ = window
        .set_timeout_with_callback_and_timeout_and_arguments_0(callback.unchecked_ref(), 1000);
    count
}

/// JSON 文字列を検証して取り込む。取り込んだ件数を返す。不正なら 0。
#[cfg(target_arch = "wasm32")]
fn import_positions(raw: &str) -> Result<usize, String> {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(raw) else {
        return Err("JSON の解析に失敗しました".to_string());
    };
    let serde_json::Value::Object(map) = value else {
        return Err("JSON はオブジェクトである必要があります".to_string());
    };
    let Some(window) = web_sys::window() else {
        return Err("window が取得できません".to_string());
    };
    let Ok(Some(storage)) = window.local_storage() else {
        return Err("localStorage が使えません".to_string());
    };
    // 新形式: { positions: {...}, settings: {...} }
    // 旧形式: { position_key: value, ... }
    let (positions, settings) = if map.contains_key("positions") || map.contains_key("settings") {
        let positions = match map.get("positions") {
            Some(serde_json::Value::Object(map)) => map.clone(),
            _ => serde_json::Map::new(),
        };
        let settings = map.get("settings").cloned();
        (positions, settings)
    } else {
        (map.clone(), None)
    };

    let mut imported = 0_usize;
    for (key, value) in &positions {
        if !key.starts_with(POSITION_KEY_PREFIX) {
            continue;
        }
        // 値は文字列でもオブジェクトでも受け付ける。
        let text = match value {
            serde_json::Value::String(text) => text.clone(),
            serde_json::Value::Object(_) => match serde_json::to_string(value) {
                Ok(text) => text,
                Err(_) => continue,
            },
            _ => continue,
        };
        // 値が SavedPosition として読めるか確認する。
        if serde_json::from_str::<SavedPosition>(&text).is_err() {
            continue;
        }
        if storage.set_item(&key, &text).is_ok() {
            imported += 1;
        }
    }

    // 表示設定の取り込み。
    let settings_obj = settings.clone();
    if let Some(serde_json::Value::Object(settings)) = settings_obj {
        let merged = ReaderSettings {
            font_scale: settings
                .get("fontScale")
                .and_then(|v| v.as_u64())
                .map(|v| v as usize)
                .unwrap_or_else(|| load_settings().font_scale),
            line_height: settings
                .get("lineHeight")
                .and_then(|v| v.as_u64())
                .map(|v| v as usize)
                .unwrap_or_else(|| load_settings().line_height),
            dark: settings
                .get("dark")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
        }
        .normalized();
        save_settings(&merged);
    }

    if imported == 0 && positions.is_empty() && settings.is_none() {
        return Err("取り込めるデータがありませんでした".to_string());
    }
    Ok(imported)
}

/// 履歴を保存する localStorage のキーの接頭辞。
#[cfg(target_arch = "wasm32")]
const HISTORY_KEY_PREFIX: &str = "openrustbooks:history:";

/// 履歴の 1 エントリを JSON に変換するための構造体。
#[cfg(target_arch = "wasm32")]
#[derive(serde::Deserialize, serde::Serialize, Clone)]
struct SavedHistoryEntry {
    #[serde(rename = "chapterIndex")]
    chapter_index: usize,
    fragment: Option<String>,
    #[serde(rename = "scrollTop", default)]
    scroll_top: i32,
}

/// 履歴全体を JSON に変換するための構造体。
#[cfg(target_arch = "wasm32")]
#[derive(serde::Deserialize, serde::Serialize, Clone)]
struct SavedHistory {
    entries: Vec<SavedHistoryEntry>,
    index: usize,
}

/// 履歴のキーを作る。
#[cfg(target_arch = "wasm32")]
fn history_key(file: &web_sys::File) -> String {
    format!(
        "{HISTORY_KEY_PREFIX}{}:{}:{}",
        file.name(),
        file.size(),
        file.last_modified()
    )
}

/// 保存された履歴を読む。localStorage が使えない・壊れている場合は `None`。
#[cfg(target_arch = "wasm32")]
fn load_history(key: &str, chapter_count: usize) -> Option<SavedHistory> {
    let storage = web_sys::window()?.local_storage().ok()??;
    let raw = storage.get_item(key).ok()??;
    let saved: SavedHistory = serde_json::from_str(&raw).ok()?;
    if saved.index >= saved.entries.len() {
        return None;
    }
    if saved
        .entries
        .iter()
        .any(|entry| entry.chapter_index >= chapter_count)
    {
        return None;
    }
    Some(saved)
}

/// 履歴を保存する。localStorage が使えない場合は何もしない。
#[cfg(target_arch = "wasm32")]
fn save_history(key: &str, history: &[HistoryEntry], index: usize) {
    if history.len() <= 1 {
        return;
    }
    let Some(window) = web_sys::window() else {
        return;
    };
    let Ok(Some(storage)) = window.local_storage() else {
        return;
    };
    let saved = SavedHistory {
        entries: history
            .iter()
            .map(|entry| SavedHistoryEntry {
                chapter_index: entry.chapter_index,
                fragment: entry.fragment.clone(),
                scroll_top: entry.scroll_top,
            })
            .collect(),
        index,
    };
    if let Ok(raw) = serde_json::to_string(&saved) {
        let _ = storage.set_item(key, &raw);
    }
}

/// 移動履歴の 1 エントリ。
#[cfg(target_arch = "wasm32")]
#[derive(Clone)]
struct HistoryEntry {
    chapter_index: usize,
    fragment: Option<String>,
    scroll_top: i32,
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
    /// 移動履歴の上限。
    const HISTORY_LIMIT: usize = 50;

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
    let history_key_state = Rc::new(RefCell::new(None::<String>));
    let touch_start = Rc::new(Cell::new(None::<(f64, f64)>));
    // 現在の章で適用中のフラグメント。スクロール保存で消さないために保持する。
    let current_fragment = Rc::new(RefCell::new(None::<String>));
    let history = Rc::new(RefCell::new(Vec::<HistoryEntry>::new()));
    let history_index = Rc::new(Cell::new(0_usize));
    let (history_state, set_history_state) = signal((0_usize, 0_usize));
    let can_back = Memo::new(move |_| history_state.get().1 > 0);
    let can_forward = Memo::new(move |_| {
        let (len, index) = history_state.get();
        index + 1 < len
    });
    let (chapters, set_chapters) = signal(Vec::<(usize, String, Option<String>)>::new());
    let (current, set_current) = signal(0_usize);
    let (content, set_content) = signal(String::new());
    let (status, set_status) = signal(String::from("Choose an EPUB file to begin."));
    let (cover_url, set_cover_url) = signal(None::<String>);
    let initial_settings = load_settings();
    let (font_scale, set_font_scale) = signal(initial_settings.font_scale);
    let (line_height, set_line_height) = signal(initial_settings.line_height);
    let (dark, set_dark) = signal(initial_settings.dark);
    // ダーク表示をアプリ全体（<html>）へ反映する。
    Effect::new(move || {
        apply_dark_class(dark.get());
    });

    let render_chapter: Rc<dyn Fn(usize, Option<String>, Option<i32>)> = {
        let book = Rc::clone(&book);
        let object_urls = Rc::clone(&object_urls);
        let position_key = Rc::clone(&position_key_state);
        let current_fragment = Rc::clone(&current_fragment);
        Rc::new(move |index, fragment, scroll_top| {
            *current_fragment.borrow_mut() = fragment.clone();
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
                    // `set_content` は html を move するため、先に画像数を数える。
                    let expected_images = html.matches("<img").count();
                    set_current.set(index);
                    set_content.set(html);
                    set_status.set(String::new());
                    apply_position_after_render(
                        index,
                        fragment,
                        scroll_top,
                        expected_images,
                        Rc::clone(&position_key),
                    );
                }
                Err(error) => set_status.set(error),
            }
        })
    };

    // 履歴の件数と現在位置を UI 用の signal に反映する。
    let sync_history = {
        let history = Rc::clone(&history);
        let history_index = Rc::clone(&history_index);
        Rc::new(move || {
            set_history_state.set((history.borrow().len(), history_index.get()));
        })
    };

    // 章を移動し、履歴に積む。
    let navigate_to: Rc<dyn Fn(usize, Option<String>)> = {
        let history = Rc::clone(&history);
        let history_index = Rc::clone(&history_index);
        let sync_history = Rc::clone(&sync_history);
        let render_chapter = Rc::clone(&render_chapter);
        let history_key_state = Rc::clone(&history_key_state);
        Rc::new(move |index, fragment| {
            let current_index = history_index.get();
            {
                let mut entries = history.borrow_mut();
                // 移動前の位置を現在エントリへ記録する。
                if let Some(entry) = entries.get_mut(current_index)
                    && entry.chapter_index == current.get()
                {
                    entry.fragment = None;
                    entry.scroll_top = content_scroll_top();
                }
                // 戻った後に新しい移動をしたら、進む方向は捨てる。
                entries.truncate(current_index + 1);
                entries.push(HistoryEntry {
                    chapter_index: index,
                    fragment: fragment.clone(),
                    scroll_top: 0,
                });
                if entries.len() > HISTORY_LIMIT {
                    let overflow = entries.len() - HISTORY_LIMIT;
                    entries.drain(0..overflow);
                }
                history_index.set(entries.len() - 1);
            }
            sync_history();
            if let Some(key) = history_key_state.borrow().as_ref() {
                save_history(key, &history.borrow(), history_index.get());
            }
            render_chapter(index, fragment, None);
        })
    };

    let back_action: Rc<dyn Fn()> = {
        let history = Rc::clone(&history);
        let history_index = Rc::clone(&history_index);
        let sync_history = Rc::clone(&sync_history);
        let render_chapter = Rc::clone(&render_chapter);
        let history_key_state = Rc::clone(&history_key_state);
        Rc::new(move || {
            let index = history_index.get();
            if index == 0 {
                return;
            }
            // 移動前の位置を現在エントリへ記録する。
            if let Some(entry) = history.borrow_mut().get_mut(index)
                && entry.chapter_index == current.get()
            {
                entry.fragment = None;
                entry.scroll_top = content_scroll_top();
            }
            let new_index = index - 1;
            let entry = history.borrow().get(new_index).cloned();
            let Some(entry) = entry else {
                return;
            };
            history_index.set(new_index);
            sync_history();
            if let Some(key) = history_key_state.borrow().as_ref() {
                save_history(key, &history.borrow(), history_index.get());
            }
            render_chapter(entry.chapter_index, entry.fragment, Some(entry.scroll_top));
        })
    };

    let forward_action: Rc<dyn Fn()> = {
        let history = Rc::clone(&history);
        let history_index = Rc::clone(&history_index);
        let sync_history = Rc::clone(&sync_history);
        let render_chapter = Rc::clone(&render_chapter);
        let history_key_state = Rc::clone(&history_key_state);
        Rc::new(move || {
            let index = history_index.get();
            let len = history.borrow().len();
            if index + 1 >= len {
                return;
            }
            if let Some(entry) = history.borrow_mut().get_mut(index)
                && entry.chapter_index == current.get()
            {
                entry.fragment = None;
                entry.scroll_top = content_scroll_top();
            }
            let new_index = index + 1;
            let entry = history.borrow().get(new_index).cloned();
            let Some(entry) = entry else {
                return;
            };
            history_index.set(new_index);
            sync_history();
            if let Some(key) = history_key_state.borrow().as_ref() {
                save_history(key, &history.borrow(), history_index.get());
            }
            render_chapter(entry.chapter_index, entry.fragment, Some(entry.scroll_top));
        })
    };
    let on_history_back = {
        let back_action = Rc::clone(&back_action);
        move |_| back_action()
    };
    let on_history_forward = {
        let forward_action = Rc::clone(&forward_action);
        move |_| forward_action()
    };

    let on_file_change = {
        let book = Rc::clone(&book);
        let object_urls = Rc::clone(&object_urls);
        let cover_url_state = Rc::clone(&cover_url_state);
        let position_key_state = Rc::clone(&position_key_state);
        let history_key_state = Rc::clone(&history_key_state);
        let render_chapter = Rc::clone(&render_chapter);
        let history = Rc::clone(&history);
        let history_index = Rc::clone(&history_index);
        let sync_history = Rc::clone(&sync_history);
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
            // 別の本を開いたら履歴をリセットする。
            *history.borrow_mut() = Vec::new();
            history_index.set(0);
            sync_history();
            *history_key_state.borrow_mut() = Some(history_key(&file));
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
            let history = Rc::clone(&history);
            let history_index = Rc::clone(&history_index);
            let sync_history = Rc::clone(&sync_history);
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

                let restored_history = load_history(&history_key(&file), count)
                    .map(|saved| {
                        (
                            saved
                                .entries
                                .into_iter()
                                .map(|entry| HistoryEntry {
                                    chapter_index: entry.chapter_index,
                                    fragment: entry.fragment,
                                    scroll_top: entry.scroll_top,
                                })
                                .collect::<Vec<_>>(),
                            saved.index,
                        )
                    })
                    .filter(|(entries, index)| *index < entries.len() && entries.len() > 1);
                let (history_entries, history_idx) = restored_history.unwrap_or_else(|| {
                    (
                        vec![HistoryEntry {
                            chapter_index: start_index,
                            fragment: start_fragment.clone(),
                            scroll_top: start_scroll,
                        }],
                        0,
                    )
                });

                *history.borrow_mut() = history_entries;
                history_index.set(history_idx);
                sync_history();

                *book.borrow_mut() = Some(opened);
                set_chapters.set(options);
                render_chapter(start_index, start_fragment, Some(start_scroll));
            });
        }
    };

    let on_select = {
        let navigate_to = Rc::clone(&navigate_to);
        move |event: Event| {
            if let Some(select) = event
                .target()
                .and_then(|target| target.dyn_into::<web_sys::HtmlSelectElement>().ok())
                && let Ok(index) = select.value().parse::<usize>()
            {
                let fragment = chapters
                    .get()
                    .iter()
                    .find(|(chapter, _, _)| *chapter == index)
                    .and_then(|(_, _, fragment)| fragment.clone());
                navigate_to(index, fragment);
            }
        }
    };
    // 前後の章へ移動する。ボタンとキーボードの両方から使う。
    let prev_action: Rc<dyn Fn()> = {
        let navigate_to = Rc::clone(&navigate_to);
        Rc::new(move || {
            let index = current.get();
            if index > 0 {
                let fragment = chapters
                    .get()
                    .iter()
                    .find(|(chapter, _, _)| *chapter == index - 1)
                    .and_then(|(_, _, fragment)| fragment.clone());
                navigate_to(index - 1, fragment);
            }
        })
    };
    let next_action: Rc<dyn Fn()> = {
        let navigate_to = Rc::clone(&navigate_to);
        Rc::new(move || {
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
                navigate_to(index + 1, fragment);
            }
        })
    };
    let on_previous = {
        let prev_action = Rc::clone(&prev_action);
        move |_| prev_action()
    };
    let on_next = {
        let next_action = Rc::clone(&next_action);
        move |_| next_action()
    };
    let touch_start = Rc::clone(&touch_start);
    let on_touch_start = {
        let touch_start = Rc::clone(&touch_start);
        move |event: web_sys::TouchEvent| {
            if event.touches().length() != 1 {
                touch_start.set(None);
                return;
            }
            if let Some(touch) = event.touches().get(0) {
                touch_start.set(Some((touch.client_x() as f64, touch.client_y() as f64)));
            }
        }
    };
    let on_touch_end = {
        let touch_start = Rc::clone(&touch_start);
        let prev_action = Rc::clone(&prev_action);
        let next_action = Rc::clone(&next_action);
        move |event: web_sys::TouchEvent| {
            let Some((start_x, start_y)) = touch_start.get() else {
                return;
            };
            touch_start.set(None);
            if event.changed_touches().length() == 0 {
                return;
            }
            let Some(touch) = event.changed_touches().get(0) else {
                return;
            };
            let dx = touch.client_x() as f64 - start_x;
            let dy = touch.client_y() as f64 - start_y;
            // 縦移動が大きい場合はスクロールとみなして無視する。
            if dy.abs() >= 30.0 {
                return;
            }
            if dx.abs() < 50.0 {
                return;
            }
            if dx < 0.0 {
                next_action();
            } else {
                prev_action();
            }
        }
    };

    let on_content_click = {
        let book = Rc::clone(&book);
        let navigate_to = Rc::clone(&navigate_to);
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
                            saved_at: 0.0,
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
                            saved_at: 0.0,
                        },
                    );
                }
            } else {
                navigate_to(link.chapter_index, link.fragment);
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
            let current_fragment = Rc::clone(&current_fragment);
            let chapter_index = current.get();
            let callback = Closure::once_into_js(move || {
                timer.set(None);
                // 章が切り替わった後の遅延保存は書き込まない。
                if current.get() != chapter_index {
                    return;
                }
                let scroll_top = content_scroll_top();
                // 適用中のフラグメントを消さずに位置だけ更新する。
                let fragment = current_fragment.borrow().clone();
                if let Some(key) = position_key.borrow().as_ref() {
                    save_position(
                        key,
                        &SavedPosition {
                            chapter_index,
                            fragment,
                            scroll_top,
                            saved_at: 0.0,
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

    let persist_settings = Rc::new(move || {
        save_settings(&ReaderSettings {
            font_scale: font_scale.get(),
            line_height: line_height.get(),
            dark: dark.get(),
        });
    });
    let font_smaller_action: Rc<dyn Fn()> = {
        let persist = Rc::clone(&persist_settings);
        Rc::new(move || {
            set_font_scale.set(font_scale.get().saturating_sub(1));
            persist();
        })
    };
    let font_larger_action: Rc<dyn Fn()> = {
        let persist = Rc::clone(&persist_settings);
        Rc::new(move || {
            set_font_scale.set((font_scale.get() + 1).min(FONT_SCALES.len() - 1));
            persist();
        })
    };
    let dark_action: Rc<dyn Fn()> = {
        let persist = Rc::clone(&persist_settings);
        Rc::new(move || {
            set_dark.set(!dark.get());
            persist();
        })
    };
    let on_font_smaller = {
        let action = Rc::clone(&font_smaller_action);
        move |_| action()
    };
    let on_font_larger = {
        let action = Rc::clone(&font_larger_action);
        move |_| action()
    };
    let on_line_spacing = {
        let persist = Rc::clone(&persist_settings);
        move |_| {
            set_line_height.set((line_height.get() + 1) % LINE_HEIGHTS.len());
            persist();
        }
    };
    let on_toggle_dark = {
        let action = Rc::clone(&dark_action);
        move |_| action()
    };
    let on_clear_data = move |_| {
        let removed = clear_reading_positions();
        set_status.set(format!("Removed {removed} saved reading positions."));
    };
    let on_export = move |_: web_sys::MouseEvent| {
        let exported = export_positions();
        if exported == 0 {
            set_status.set("No saved reading positions to export.".to_string());
        } else {
            set_status.set(format!("Exported {exported} reading positions."));
        }
    };
    let on_import_click = move |_: web_sys::MouseEvent| {
        if let Some(window) = web_sys::window()
            && let Some(document) = window.document()
            && let Ok(Some(element)) = document.query_selector("#reader-import-file")
            && let Ok(input) = element.dyn_into::<web_sys::HtmlInputElement>()
        {
            input.click();
        }
    };
    let on_import_change = move |event: web_sys::Event| {
        let Some(input) = event
            .target()
            .and_then(|target| target.dyn_into::<web_sys::HtmlInputElement>().ok())
        else {
            return;
        };
        let Some(file) = input.files().and_then(|files| files.item(0)) else {
            return;
        };
        let file_clone = file.clone();
        spawn_local(async move {
            let Ok(buffer) = JsFuture::from(file_clone.text()).await else {
                set_status.set("Failed to read the import file.".to_string());
                return;
            };
            let Some(raw) = buffer.as_string() else {
                set_status.set("Failed to read the import file.".to_string());
                return;
            };
            match import_positions(&raw) {
                Ok(count) => set_status.set(format!("Imported {count} reading positions.")),
                Err(error) => set_status.set(error),
            }
        });
    };
    // view マクロ内の比較で `>` を使わないよう、条件を外で計算する。
    let font_larger_disabled = Memo::new(move |_| font_scale.get() + 1 >= FONT_SCALES.len());

    // キーボード操作。フォーカスが入力欄にあるときは無視する。
    let key_handler = {
        let prev_action = Rc::clone(&prev_action);
        let next_action = Rc::clone(&next_action);
        let back_action = Rc::clone(&back_action);
        let forward_action = Rc::clone(&forward_action);
        let font_smaller_action = Rc::clone(&font_smaller_action);
        let font_larger_action = Rc::clone(&font_larger_action);
        let dark_action = Rc::clone(&dark_action);
        move |event: web_sys::KeyboardEvent| {
            // Ctrl / Meta 併用はブラウザのショートカットを奪わない。
            if event.ctrl_key() || event.meta_key() {
                return;
            }
            // Alt は ArrowLeft / ArrowRight の章の戻る・進むにだけ使う。
            if event.alt_key() && event.key() != "ArrowLeft" && event.key() != "ArrowRight" {
                return;
            }
            // フォーカスが入力欄にあるときはキーを奪わない。
            if let Some(active) = web_sys::window()
                .and_then(|w| w.document())
                .and_then(|d| d.active_element())
            {
                let tag = active.tag_name().to_ascii_lowercase();
                if tag == "input" || tag == "select" || tag == "textarea" {
                    return;
                }
            }
            let key = event.key();
            let handled = match key.as_str() {
                "]" => {
                    next_action();
                    true
                }
                "[" => {
                    prev_action();
                    true
                }
                "+" | "=" => {
                    font_larger_action();
                    true
                }
                "-" | "_" => {
                    font_smaller_action();
                    true
                }
                "d" => {
                    dark_action();
                    true
                }
                "ArrowLeft" if event.alt_key() => {
                    back_action();
                    true
                }
                "ArrowRight" if event.alt_key() => {
                    forward_action();
                    true
                }
                _ => false,
            };
            if handled {
                event.prevent_default();
            }
        }
    };
    let key_listener = window_event_listener(leptos::ev::keydown, key_handler);
    on_cleanup(move || {
        key_listener.remove();
    });

    let current_title = Memo::new(move |_| {
        chapters
            .get()
            .iter()
            .find(|(chapter, _, _)| *chapter == current.get())
            .map(|(_, title, _)| title.clone())
            .unwrap_or_default()
    });

    view! {
        <div
            class="reader-container"
            class:reader-dark=move || dark.get()
            class:reader-font-0=move || font_scale.get() == 0
            class:reader-font-1=move || font_scale.get() == 1
            class:reader-font-2=move || font_scale.get() == 2
            class:reader-font-3=move || font_scale.get() == 3
            class:reader-font-4=move || font_scale.get() == 4
            class:reader-line-0=move || line_height.get() == 0
            class:reader-line-1=move || line_height.get() == 1
            class:reader-line-2=move || line_height.get() == 2
            data-doc-id=move || doc_id.get()
        >
            <div class="reader-toolbar">
                <label class="btn btn-primary" for="epub-file">"Open EPUB"</label>
                <input id="epub-file" class="reader-file-input" type="file" accept=".epub,application/epub+zip" on:change=on_file_change/>
                <button id="reader-font-smaller" class="btn" title="Smaller text" aria-label="Smaller text" aria-keyshortcuts="-" on:click=on_font_smaller disabled=move || font_scale.get() == 0>"A-"</button>
                <button id="reader-font-larger" class="btn" title="Larger text" aria-label="Larger text" aria-keyshortcuts="+" on:click=on_font_larger disabled=move || font_larger_disabled.get()>"A+"</button>
                <button id="reader-line-spacing" class="btn" title="Line spacing" aria-label="Line spacing" on:click=on_line_spacing>"Line spacing"</button>
                <button id="reader-theme" class="btn" title="Toggle dark mode" aria-label="Toggle dark mode" aria-keyshortcuts="d" on:click=on_toggle_dark>{move || if dark.get() { "Light" } else { "Dark" }}</button>
                <button id="reader-clear-data" class="btn" title="Clear saved reading positions" aria-label="Clear saved reading positions" on:click=on_clear_data>"Clear data"</button>
                <button id="reader-export" class="btn" title="Export saved reading positions" aria-label="Export saved reading positions" on:click=on_export>"Export"</button>
                <button id="reader-import" class="btn" title="Import saved reading positions" aria-label="Import saved reading positions" on:click=on_import_click>"Import"</button>
                <input id="reader-import-file" class="reader-file-input" type="file" accept=".json,application/json" on:change=on_import_change/>
                <button id="reader-back" class="btn" aria-label="Go back" aria-keyshortcuts="Alt+ArrowLeft" on:click=on_history_back disabled=move || !can_back.get()>"Back"</button>
                <button id="reader-forward" class="btn" aria-label="Go forward" aria-keyshortcuts="Alt+ArrowRight" on:click=on_history_forward disabled=move || !can_forward.get()>"Forward"</button>
                <button class="btn" aria-label="Previous chapter" aria-keyshortcuts="[" on:click=on_previous disabled=move || current.get() == 0>"◀"</button>
                <select class="reader-chapter-select" aria-label="Chapter" on:change=on_select prop:disabled=move || chapters.get().is_empty()>
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
                <button class="btn" aria-label="Next chapter" aria-keyshortcuts="]" on:click=on_next disabled=move || {
                    let index = current.get();
                    !chapters.get().iter().any(|(chapter, _, _)| *chapter > index)
                }>
                    "▶"
                </button>
                <span class="reader-chapter-title">{move || current_title.get()}</span>
            </div>
            <p class="reader-status" role="status" aria-live="polite">{move || status.get()}</p>
            <div class="reader-body">
                <Show when=move || cover_url.get().is_some()>
                    <aside class="reader-sidebar">
                        <img class="reader-cover" src=move || cover_url.get().unwrap_or_default() alt="Cover"/>
                    </aside>
                </Show>
                <div class="reader-content" role="region" aria-label="Chapter content" tabindex="0" on:click=on_content_click on:scroll=on_scroll on:touchstart=on_touch_start on:touchend=on_touch_end>
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
