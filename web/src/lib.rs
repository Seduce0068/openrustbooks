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
    // scrollIntoView / offsetTop はレイアウト未確定のときに 0 を返すことが
    // あるため、getBoundingClientRect() で実座標を求めて scrollTop を設定する。
    if let Ok(Some(content)) = document.query_selector(".reader-content") {
        let content_rect = content.get_bounding_client_rect();
        let target_rect = element.get_bounding_client_rect();
        let current = content.scroll_top();
        let target = current + (target_rect.top() - content_rect.top()) as i32;
        content.set_scroll_top(target);
        return;
    }
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
/// 位置の適用に必要な状態。再試行の間で共有するため `Clone` を持つ。
#[cfg(target_arch = "wasm32")]
#[derive(Clone)]
struct PositionApplyCtx {
    chapter_index: usize,
    fragment: Option<String>,
    scroll_top: Option<i32>,
    expected_images: usize,
    position_key: SharedPositionKey,
    /// 位置の適用中フラグ。適用中のスクロール保存を抑制する。
    applying: std::rc::Rc<std::cell::Cell<bool>>,
    /// 位置の適用を完了した時刻。適用直後のスクロール保存を抑制する。
    last_apply_time: std::rc::Rc<std::cell::Cell<f64>>,
    /// 章描画の世代。古い適用コールバックが新しい章を壊さないようにする。
    generation: u32,
    /// 現在の世代を保持する共有カウンタ。
    current_generation: std::rc::Rc<std::cell::Cell<u32>>,
    /// 状態メッセージの更新用（巨大画像の警告に使う）。
    set_status: std::rc::Rc<dyn Fn(String)>,
    /// 巨大画像を縮小するか（既定オフ）。
    downscale_images: bool,
    /// 生成した Blob URL の管理リスト。
    object_urls: std::rc::Rc<std::cell::RefCell<Vec<String>>>,
    /// 縮小 JPEG の品質（パーセント）。
    jpeg_quality: usize,
}

/// エクスポートのダウンロード名をローカル時刻のタイムスタンプ付きで作る。
#[cfg(target_arch = "wasm32")]
fn export_filename() -> String {
    let date = js_sys::Date::new_0();
    let pad = |value: u32| -> String { format!("{value:02}") };
    // getTimezoneOffset は「UTC - 現地」の分（JST なら -540）なので符号を反転する。
    let offset_minutes = -(date.get_timezone_offset() as i32);
    let sign = if offset_minutes < 0 { '-' } else { '+' };
    let absolute = offset_minutes.unsigned_abs();
    format!(
        "openrustbooks-{:04}{}{}-{}{}{}{}{:02}{:02}.json",
        date.get_full_year(),
        pad(date.get_month() + 1),
        pad(date.get_date()),
        pad(date.get_hours()),
        pad(date.get_minutes()),
        pad(date.get_seconds()),
        sign,
        absolute / 60,
        absolute % 60,
    )
}

/// Blob URL から元画像の MIME を取得する（失敗時は None）。
#[cfg(target_arch = "wasm32")]
async fn fetch_blob_type(url: &str) -> Option<String> {
    let window = web_sys::window()?;
    let response = wasm_bindgen_futures::JsFuture::from(window.fetch_with_str(url))
        .await
        .ok()?;
    let response: web_sys::Response = response.dyn_into().ok()?;
    let blob = wasm_bindgen_futures::JsFuture::from(response.blob().ok()?)
        .await
        .ok()?;
    let blob: web_sys::Blob = blob.dyn_into().ok()?;
    Some(blob.type_())
}

/// 巨大画像を Canvas で縮小し、Blob URL を差し替える。
///
/// CSS の `max-width: 100%` で表示幅が決まる画像（自然幅が本文領域より大きい
/// もの）だけを対象にする。狭い画像を縮小すると表示サイズが変わり、
/// スクロール位置補正が壊れるため除外する。
#[cfg(target_arch = "wasm32")]
async fn downscale_large_images(
    object_urls: std::rc::Rc<std::cell::RefCell<Vec<String>>>,
    jpeg_quality: usize,
) {
    const DOWNSCALE_THRESHOLD: u32 = 4000;
    const MIN_MAX_DIMENSION: u32 = 2000;
    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(document) = window.document() else {
        return;
    };
    let Ok(images) = document.query_selector_all(".reader-html img") else {
        return;
    };
    let Ok(Some(content)) = document.query_selector(".reader-content") else {
        return;
    };
    let content_width = content.client_width().max(0) as u32;
    let max_dimension = MIN_MAX_DIMENSION.max(content_width);

    for index in 0..images.length() {
        let Some(node) = images.get(index) else {
            continue;
        };
        let Ok(image) = node.dyn_into::<web_sys::HtmlImageElement>() else {
            continue;
        };
        let width = image.natural_width();
        let height = image.natural_height();
        if width <= DOWNSCALE_THRESHOLD && height <= DOWNSCALE_THRESHOLD {
            continue;
        }
        // 表示幅が自然幅に依存する画像は縮小しない（レイアウト変化を避ける）。
        if width <= content_width || width == 0 || height == 0 {
            continue;
        }
        let scale = max_dimension as f64 / width.max(height) as f64;
        let target_width = (width as f64 * scale).round().max(1.0) as u32;
        let target_height = (height as f64 * scale).round().max(1.0) as u32;
        let Ok(canvas) = document.create_element("canvas") else {
            continue;
        };
        let Ok(canvas) = canvas.dyn_into::<web_sys::HtmlCanvasElement>() else {
            continue;
        };
        canvas.set_width(target_width);
        canvas.set_height(target_height);
        let Ok(Some(context)) = canvas.get_context("2d") else {
            continue;
        };
        let Ok(context) = context.dyn_into::<web_sys::CanvasRenderingContext2d>() else {
            continue;
        };
        if context
            .draw_image_with_html_image_element_and_dw_and_dh(
                &image,
                0.0,
                0.0,
                target_width as f64,
                target_height as f64,
            )
            .is_err()
        {
            continue;
        }
        let old_url = image.get_attribute("src").unwrap_or_default();
        // 元画像の MIME を取得し、JPEG / PNG / WebP なら同じ形式で再エンコードする。
        let original_mime = if old_url.is_empty() {
            None
        } else {
            fetch_blob_type(&old_url).await
        };
        let mime = original_mime
            .filter(|value| matches!(value.as_str(), "image/jpeg" | "image/png" | "image/webp"));
        let urls = std::rc::Rc::clone(&object_urls);
        let target = image.clone();
        let callback = Closure::once_into_js(move |blob: web_sys::Blob| {
            if let Ok(url) = web_sys::Url::create_object_url_with_blob(&blob) {
                target.set_attribute("src", &url).ok();
                urls.borrow_mut().push(url);
                if !old_url.is_empty() {
                    let _ = web_sys::Url::revoke_object_url(&old_url);
                }
            }
        });
        let callback: js_sys::Function = callback.unchecked_into();
        if let Some(mime) = mime {
            if mime == "image/jpeg" {
                // JPEG は品質を明示してブラウザ依存の既定値を避ける。
                let options = js_sys::Object::new();
                if js_sys::Reflect::set(
                    &options,
                    &JsValue::from_str("quality"),
                    &JsValue::from_f64(jpeg_quality as f64 / 100.0),
                )
                .is_ok()
                {
                    let _ = canvas.to_blob_with_type_and_encoder_options(
                        &callback,
                        &mime,
                        &JsValue::from(options),
                    );
                }
            } else {
                let _ = canvas.to_blob_with_type(&callback, &mime);
            }
        } else {
            let _ = canvas.to_blob(&callback);
        }
    }
}

/// 本文の巨大画像を検出する（閾値より大きい場合のみ警告）。
#[cfg(target_arch = "wasm32")]
fn find_large_image() -> Option<String> {
    const LARGE_IMAGE_THRESHOLD: u32 = 4000;
    let document = web_sys::window()?.document()?;
    let images = document.query_selector_all(".reader-html img").ok()?;
    for index in 0..images.length() {
        let Some(node) = images.get(index) else {
            continue;
        };
        let Ok(image) = node.dyn_into::<web_sys::HtmlImageElement>() else {
            continue;
        };
        let width = image.natural_width();
        let height = image.natural_height();
        if width > LARGE_IMAGE_THRESHOLD || height > LARGE_IMAGE_THRESHOLD {
            return Some(format!(
                "Large image detected ({width}x{height}). Consider a smaller source."
            ));
        }
    }
    None
}

#[cfg(target_arch = "wasm32")]
fn apply_position_after_render(ctx: PositionApplyCtx) {
    let deadline = now_millis() + IMAGE_SETTLE_MS;
    apply_position_frame(ctx, 0, deadline);
}

#[cfg(target_arch = "wasm32")]
fn apply_position_frame(ctx: PositionApplyCtx, attempt: u32, deadline: f64) {
    // より新しい章の描画が始まっていたら、画像待ちループに入る前にも
    // 破棄する（古い描画のループが無駄に回り続けるのを防ぐ）。
    if ctx.generation != ctx.current_generation.get() {
        return;
    }
    let Some(window) = web_sys::window() else {
        return;
    };
    // 画像の読み込みで本文の高さが変わると、同じ scrollTop が別の内容を指す。
    // 画像が DOM に現れて確定するまで（上限付きで）位置の適用を待つ。
    if !images_settled(ctx.expected_images) && now_millis() < deadline {
        let next = ctx.clone();
        let callback = Closure::once_into_js(move || {
            apply_position_frame(next, attempt, deadline);
        });
        let _ = window
            .set_timeout_with_callback_and_timeout_and_arguments_0(callback.unchecked_ref(), 0);
        return;
    }
    let callback = Closure::once_into_js(move || {
        // より新しい章の描画が始まっていたら、古い適用は破棄する。
        // 連続した章移動で前の章の先頭リセットが新しい章に効くのを防ぐ。
        if ctx.generation != ctx.current_generation.get() {
            return;
        }
        match ctx.fragment.as_deref() {
            Some(fragment) => scroll_to_fragment(fragment),
            None => match ctx.scroll_top {
                Some(offset) if offset > 0 => set_content_scroll_top(offset),
                _ => scroll_content_to_top(),
            },
        }
        let actual = content_scroll_top();
        let wanted = ctx.scroll_top.unwrap_or(0);
        // 要求した位置に届いていない場合は再試行する。スクロール位置は
        // レイアウトが終わるまで反映されないことがあるため。
        // フラグメント移動でも、画像のレイアウト確定前に scrollIntoView が
        // 走ると 0 のままになるため再試行する。
        let fragment_not_applied = ctx.fragment.is_some() && actual == 0;
        if ctx.fragment.is_some() && fragment_not_applied && attempt < 12 {
            // 画像のレイアウトが確定する前に scrollTop を設定すると 0 のまま
            // になるため、1 フレーム相当の間隔を空けて再試行する。
            let next = ctx.clone();
            let retry = Closure::once_into_js(move || {
                apply_position_frame(next, attempt + 1, deadline);
            });
            if let Some(retry_window) = web_sys::window() {
                let _ = retry_window.set_timeout_with_callback_and_timeout_and_arguments_0(
                    retry.unchecked_ref(),
                    16,
                );
            }
            return;
        }
        if ctx.fragment.is_none() && actual != wanted && attempt < 3 {
            apply_position_frame(ctx.clone(), attempt + 1, deadline);
            return;
        }
        // フラグメント移動のときだけ、適用後の位置を保存する。
        // None の場合は render_chapter で既に保存済み。
        if ctx.fragment.is_some()
            && let Some(key) = ctx.position_key.borrow().as_ref()
        {
            save_position(
                key,
                &SavedPosition {
                    chapter_index: ctx.chapter_index,
                    fragment: ctx.fragment.clone(),
                    scroll_top: actual,
                    saved_at: 0.0,
                },
            );
        }
        // 巨大画像を検出しても表示は変えず、警告だけを出す。
        if let Some(message) = find_large_image() {
            (ctx.set_status)(message);
        }
        if ctx.downscale_images {
            let urls = std::rc::Rc::clone(&ctx.object_urls);
            let quality = ctx.jpeg_quality;
            wasm_bindgen_futures::spawn_local(async move {
                downscale_large_images(urls, quality).await;
            });
        }
        ctx.last_apply_time.set(now_millis());
        ctx.applying.set(false);
    });
    let _ =
        window.set_timeout_with_callback_and_timeout_and_arguments_0(callback.unchecked_ref(), 0);
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

/// SVG からスクリプト実行と外部リソース参照の経路を除去したバイト列を返す。
/// パース・シリアライズに失敗した場合は `None`（fail-closed）。
#[cfg(target_arch = "wasm32")]
fn sanitize_svg_bytes(bytes: &[u8]) -> Option<Vec<u8>> {
    let text = String::from_utf8(bytes.to_vec()).ok()?;
    let parser = web_sys::DomParser::new().ok()?;
    let document = parser
        .parse_from_string(&text, web_sys::SupportedType::ImageSvgXml)
        .ok()?;
    let root = document.document_element()?;

    // XML 処理命令（<?xml-stylesheet?> 等）を削除する。Blob URL を直接
    // 開いたときの外部 CSS 読み込みによる情報漏洩を防ぐ。
    let children = document.child_nodes();
    for index in 0..children.length() {
        let Some(node) = children.get(index) else {
            continue;
        };
        if node.node_type() == web_sys::Node::PROCESSING_INSTRUCTION_NODE {
            let _ = document.remove_child(&node);
        }
    }

    // スクリプト要素・foreignObject・SMIL を削除する。foreignObject は HTML の
    // スクリプトを混入でき、SMIL は後から属性を書き換えられる。画像としては
    // 不要なので丸ごと落とす。
    for selector in [
        "script",
        "foreignObject",
        "set",
        "animate",
        "animateMotion",
        "animateTransform",
    ] {
        if let Ok(nodes) = document.query_selector_all(selector) {
            for index in 0..nodes.length() {
                if let Some(node) = nodes.get(index)
                    && let Some(element) = node.dyn_ref::<web_sys::Element>()
                {
                    element.remove();
                }
            }
        }
    }

    // 全要素の on* 属性と javascript: リンクを削除する。
    if let Ok(all) = document.query_selector_all("*") {
        for index in 0..all.length() {
            let Some(node) = all.get(index) else {
                continue;
            };
            let Some(element) = node.dyn_ref::<web_sys::Element>() else {
                continue;
            };
            let mut removable: Vec<String> = Vec::new();
            for name in js_sys::Array::from(&element.get_attribute_names()).iter() {
                let name = match name.as_string() {
                    Some(name) => name.to_ascii_lowercase(),
                    None => continue,
                };
                let value = element
                    .get_attribute(&name)
                    .unwrap_or_default()
                    .to_ascii_lowercase();
                // ブラウザは javascript: の途中の空白・制御文字を無視するため、
                // 判定前に取り除く（"java\nscript:" のような回避を防ぐ）。
                let normalized: String = value
                    .chars()
                    .filter(|character| !character.is_whitespace() && !character.is_control())
                    .collect();
                let is_event = name.starts_with("on");
                let is_link = name == "href" || name.ends_with(":href");
                // javascript: に加えて、外部スキームも削除する。Blob URL を
                // 直接開いたときのネットワーク要求（IP・閲覧時刻の漏洩）を防ぐ。
                let is_external = ["javascript:", "http:", "https:", "data:", "blob:", "file:"]
                    .iter()
                    .any(|scheme| normalized.starts_with(scheme));
                if is_event || (is_link && is_external) {
                    removable.push(name);
                }
            }
            for name in removable {
                let _ = element.remove_attribute(&name);
            }
        }
    }

    let clean = root.outer_html();
    if clean.is_empty() {
        return None;
    }
    Some(clean.into_bytes())
}

/// バイト列から Blob URL を作る。呼び出し側が寿命を管理する。
#[cfg(target_arch = "wasm32")]
fn object_url_for(bytes: &[u8], href: &str) -> Result<String, JsValue> {
    let mime = mime_for_path(href);
    // SVG は Blob URL を直接開かれるとアプリのオリジンで評価されるため、
    // スクリプトの経路を除去してから Blob にする。
    let payload = if mime == "image/svg+xml" {
        match sanitize_svg_bytes(bytes) {
            Some(safe) => safe,
            None => return Err(JsValue::from_str("Failed to sanitize SVG.")),
        }
    } else {
        bytes.to_vec()
    };
    let parts = js_sys::Array::new();
    parts.push(&js_sys::Uint8Array::from(payload.as_slice()));
    let options = web_sys::BlobPropertyBag::new();
    options.set_type(mime);
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
                    // デコードを非同期にして本文の描画をブロックしない。
                    // `loading="lazy"` は付けない（位置補正が `complete` を
                    // 待つため、画面外の画像が永久に未完了になる）。
                    let _ = image.set_attribute("decoding", "async");
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

/// 章を描画するコールバックの型。(章インデックス, フラグメント, スクロール位置)。
#[cfg(target_arch = "wasm32")]
type RenderChapter = std::rc::Rc<dyn Fn(usize, Option<String>, Option<i32>)>;

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

/// 保存時刻をローカルタイムの `YYYY-MM-DD HH:MM` に整形する。
/// `saved_at` が 0（旧データ）の場合は空文字を返す。
#[cfg(target_arch = "wasm32")]
fn format_saved_at(saved_at: f64) -> String {
    if saved_at <= 0.0 {
        return String::new();
    }
    let date = js_sys::Date::new(&JsValue::from_f64(saved_at));
    let pad = |value: u32| -> String { format!("{value:02}") };
    let year = date.get_full_year();
    let month = pad(date.get_month() + 1);
    let day = pad(date.get_date());
    let hour = pad(date.get_hours());
    let minute = pad(date.get_minutes());
    format!("{year:04}-{month}-{day} {hour}:{minute}")
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
    if let Some(window) = web_sys::window() {
        let _ = js_sys::Reflect::set(
            &window,
            &JsValue::from_str("__prune_before"),
            &JsValue::from_f64(remaining.len() as f64),
        );
    }
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
    if let Some(window) = web_sys::window() {
        let _ = js_sys::Reflect::set(
            &window,
            &JsValue::from_str("__prune_after"),
            &JsValue::from_f64(list_positions(storage).len() as f64),
        );
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
    /// OS の prefers-color-scheme に追従するか。
    #[serde(rename = "followSystem", default)]
    follow_system: bool,
    /// 章移動のスワイプ閾値（px）。既定 50、範囲 30〜120。
    #[serde(rename = "swipeThreshold", default = "default_swipe_threshold")]
    swipe_threshold: usize,
    /// 縦移動をスクロールとみなす閾値（px）。既定 30、範囲 10〜80。
    #[serde(
        rename = "verticalScrollThreshold",
        default = "default_vertical_scroll_threshold"
    )]
    vertical_scroll_threshold: usize,
    /// 巨大画像を Canvas で縮小するか（既定オフ）。
    #[serde(rename = "downscaleImages", default)]
    downscale_images: bool,
    /// 縮小 JPEG の品質（パーセント）。既定 85、範囲 50〜100。
    #[serde(rename = "jpegQuality", default = "default_jpeg_quality")]
    jpeg_quality: usize,
}

#[cfg(target_arch = "wasm32")]
fn default_jpeg_quality() -> usize {
    85
}

#[cfg(target_arch = "wasm32")]
fn default_swipe_threshold() -> usize {
    50
}

#[cfg(target_arch = "wasm32")]
fn default_vertical_scroll_threshold() -> usize {
    30
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
            follow_system: false,
            swipe_threshold: 50,
            vertical_scroll_threshold: 30,
            downscale_images: false,
            jpeg_quality: 85,
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
            follow_system: self.follow_system,
            swipe_threshold: self.swipe_threshold.clamp(30, 120),
            vertical_scroll_threshold: self.vertical_scroll_threshold.clamp(10, 80),
            downscale_images: self.downscale_images,
            jpeg_quality: self.jpeg_quality.clamp(50, 100),
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
        // 設定キーが無い初回起動に限り、OS がダークなら追従を有効にする。
        // 設定キーが存在する場合は後方互換のためデフォルトのまま。
        return ReaderSettings {
            follow_system: system_prefers_dark(),
            ..ReaderSettings::default()
        };
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

/// OS の prefers-color-scheme: dark を検知する。
#[cfg(target_arch = "wasm32")]
fn system_prefers_dark() -> bool {
    let Some(window) = web_sys::window() else {
        return false;
    };
    let Ok(mq) = window.match_media("(prefers-color-scheme: dark)") else {
        return false;
    };
    match mq {
        Some(mq) => mq.matches(),
        None => false,
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

    // 移動履歴も含める。値は localStorage から取得した JSON をそのまま使う。
    let mut history = serde_json::Map::new();
    for index in 0..length {
        let Ok(Some(key)) = storage.key(index) else {
            continue;
        };
        if !key.starts_with(HISTORY_KEY_PREFIX) {
            continue;
        }
        if let Ok(Some(value)) = storage.get_item(&key) {
            // 値が SavedHistory として読めるものだけ含める。
            match serde_json::from_str::<SavedHistory>(&value) {
                Ok(_) => match serde_json::from_str::<serde_json::Value>(&value) {
                    Ok(json) => {
                        history.insert(key, json);
                    }
                    Err(_) => continue,
                },
                Err(_) => continue,
            }
        }
    }

    let mut root = serde_json::Map::new();
    root.insert(
        "version".to_string(),
        serde_json::Value::from(EXPORT_VERSION),
    );
    root.insert(
        "positions".to_string(),
        serde_json::Value::Object(positions),
    );
    root.insert(
        "settings".to_string(),
        settings.unwrap_or(serde_json::Value::Null),
    );
    root.insert("history".to_string(), serde_json::Value::Object(history));

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
    let _ = anchor.set_attribute("download", &export_filename());
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

/// JSON 文字列を検証して取り込む。(読書位置の件数, 履歴の件数) を返す。
#[cfg(target_arch = "wasm32")]
fn import_positions(raw: &str) -> Result<(usize, usize), String> {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(raw) else {
        return Err("JSON の解析に失敗しました".to_string());
    };
    let serde_json::Value::Object(map) = value else {
        return Err("JSON はオブジェクトである必要があります".to_string());
    };
    // 未知のバージョンは何も取り込まず拒否する（fail-closed）。
    // version が無い旧形式はそのまま受け付ける。
    if let Some(version) = map.get("version")
        && version.as_u64() != Some(EXPORT_VERSION as u64)
    {
        return Err(format!("Unsupported export version: {version}."));
    }
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
        let Ok(incoming) = serde_json::from_str::<SavedPosition>(&text) else {
            continue;
        };
        // 既存の値が新しい場合は上書きしない（マージ）。
        if let Ok(Some(existing_raw)) = storage.get_item(key)
            && let Ok(existing) = serde_json::from_str::<SavedPosition>(&existing_raw)
            && existing.saved_at >= incoming.saved_at
        {
            continue;
        }
        if storage.set_item(key, &text).is_ok() {
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
            follow_system: settings
                .get("followSystem")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            swipe_threshold: settings
                .get("swipeThreshold")
                .and_then(|v| v.as_u64())
                .map(|v| v as usize)
                .unwrap_or_else(|| load_settings().swipe_threshold),
            vertical_scroll_threshold: settings
                .get("verticalScrollThreshold")
                .and_then(|v| v.as_u64())
                .map(|v| v as usize)
                .unwrap_or_else(|| load_settings().vertical_scroll_threshold),
            downscale_images: settings
                .get("downscaleImages")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            jpeg_quality: settings
                .get("jpegQuality")
                .and_then(|v| v.as_u64())
                .map(|v| v as usize)
                .unwrap_or_else(|| load_settings().jpeg_quality),
        }
        .normalized();
        save_settings(&merged);
    }

    // 移動履歴の取り込み。値が SavedHistory として読めるものだけ書き込む。
    let mut imported_history = 0_usize;
    if let Some(serde_json::Value::Object(history)) = map.get("history") {
        for (key, value) in history {
            if !key.starts_with(HISTORY_KEY_PREFIX) {
                continue;
            }
            let text = match serde_json::to_string(value) {
                Ok(text) => text,
                Err(_) => continue,
            };
            if serde_json::from_str::<SavedHistory>(&text).is_err() {
                continue;
            }
            if storage.set_item(key, &text).is_ok() {
                imported_history += 1;
            }
        }
    }

    if imported == 0 && imported_history == 0 && positions.is_empty() && settings.is_none() {
        return Err("取り込めるデータがありませんでした".to_string());
    }
    Ok((imported, imported_history))
}

/// 履歴を保存する localStorage のキーの接頭辞。
#[cfg(target_arch = "wasm32")]
const HISTORY_KEY_PREFIX: &str = "openrustbooks:history:";

/// エクスポート JSON の形式バージョン。未知のバージョンは受け付けない。
#[cfg(target_arch = "wasm32")]
const EXPORT_VERSION: u32 = 1;

/// 履歴の 1 エントリを JSON に変換するための構造体。
#[cfg(target_arch = "wasm32")]
#[derive(serde::Deserialize, serde::Serialize, Clone)]
struct SavedHistoryEntry {
    #[serde(rename = "chapterIndex")]
    chapter_index: usize,
    fragment: Option<String>,
    #[serde(rename = "scrollTop", default)]
    scroll_top: i32,
    /// 旧データは 0 として読む（後方互換）。
    #[serde(rename = "savedAt", default)]
    saved_at: f64,
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
                saved_at: entry.saved_at,
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
    /// その位置を訪れた時刻（エポックミリ秒）。
    saved_at: f64,
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

    #[derive(Deserialize, Clone)]
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
    // 確認ダイアログの対象。"entry" / "clear" / ""（閉じている）。
    let (delete_target, set_delete_target) = signal(String::new());
    // 履歴一覧の UI 用。(履歴インデックス, 章インデックス, フラグメント, 保存時刻)。
    let (history_entries, set_history_entries) =
        signal(Vec::<(usize, usize, Option<String>, f64)>::new());
    let can_back = Memo::new(move |_| history_state.get().1 > 0);
    let can_forward = Memo::new(move |_| {
        let (len, index) = history_state.get();
        index + 1 < len
    });
    let (chapters, set_chapters) = signal(Vec::<(usize, String, Option<String>)>::new());
    let (chapter_filter, set_chapter_filter) = signal(String::new());
    // spine の href を章インデックス順に保持する（landmarks / page-list の解決用）。
    let (chapter_hrefs, set_chapter_hrefs) = signal(Vec::<String>::new());
    let (landmarks, set_landmarks) = signal(Vec::<TocEntry>::new());
    let (landmark_options, set_landmark_options) = signal(Vec::<(usize, String)>::new());
    let (page_list, set_page_list) = signal(Vec::<TocEntry>::new());
    let (current, set_current) = signal(0_usize);
    let (content, set_content) = signal(String::new());
    let (status, set_status) = signal(String::from("Choose an EPUB file to begin."));
    let (cover_url, set_cover_url) = signal(None::<String>);
    // 本文の言語（EPUB の metadata.language）。スクリーンリーダーの発音用。
    let (content_lang, set_content_lang) = signal(None::<String>);
    // remote-resources 宣言を持つ本を開いたときの警告。
    let (remote_resources_warning, set_remote_resources_warning) = signal(None::<String>);
    let initial_settings = load_settings();
    let (font_scale, set_font_scale) = signal(initial_settings.font_scale);
    let (line_height, set_line_height) = signal(initial_settings.line_height);
    let (dark, set_dark) = signal(initial_settings.dark);
    let (follow_system, set_follow_system) = signal(initial_settings.follow_system);
    let (swipe_threshold, set_swipe_threshold) = signal(initial_settings.swipe_threshold);
    let (vertical_scroll_threshold, set_vertical_scroll_threshold) =
        signal(initial_settings.vertical_scroll_threshold);
    let (downscale_images, set_downscale_images) = signal(initial_settings.downscale_images);
    let (jpeg_quality, set_jpeg_quality) = signal(initial_settings.jpeg_quality);
    // OS の prefers-color-scheme の現在値。OS 側の変更で更新する。
    let (system_dark, set_system_dark) = signal(system_prefers_dark());
    // 実際に適用するダーク表示。follow_system が true なら OS の設定に追従する。
    let effective_dark = Memo::new(move |_| {
        if follow_system.get() {
            system_dark.get()
        } else {
            dark.get()
        }
    });
    // ダーク表示をアプリ全体（<html>）へ反映する。
    Effect::new(move || {
        apply_dark_class(effective_dark.get());
    });

    // OS の prefers-color-scheme が変わったら、follow_system が true の
    // ときだけダーク表示を更新する。
    if let Some(window) = web_sys::window()
        && let Ok(Some(mq)) = window.match_media("(prefers-color-scheme: dark)")
    {
        let callback = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            set_system_dark.set(system_prefers_dark());
        }) as Box<dyn FnMut(web_sys::Event)>);
        let _ = mq.add_event_listener_with_callback("change", callback.as_ref().unchecked_ref());
        // Closure をリークしないよう保持する（ページ寿命まで）。
        callback.forget();
    }

    let scroll_save_timer = Rc::new(Cell::new(None::<i32>));
    // 位置適用中はスクロール保存のデバウンスを無効にする。
    let applying_position = Rc::new(Cell::new(false));
    // 位置適用の完了時刻。適用直後のスクロール保存を抑制する。
    let last_apply_time = Rc::new(Cell::new(0.0_f64));
    // 章描画の世代カウンタ。描画のたびに進めて古い適用を無効化する。
    let current_generation = Rc::new(Cell::new(0_u32));
    let render_chapter: RenderChapter = {
        let book = Rc::clone(&book);
        let object_urls = Rc::clone(&object_urls);
        let position_key = Rc::clone(&position_key_state);
        let current_fragment = Rc::clone(&current_fragment);
        let scroll_save_timer = Rc::clone(&scroll_save_timer);
        let applying_position = Rc::clone(&applying_position);
        let last_apply_time = Rc::clone(&last_apply_time);
        let current_generation = Rc::clone(&current_generation);
        let set_status = set_status;
        Rc::new(move |index, fragment, scroll_top| {
            // 章の切り替えで古いスクロールのデバウンス保存をキャンセルする。
            if let Some(window) = web_sys::window()
                && let Some(handle) = scroll_save_timer.take()
            {
                window.clear_timeout_with_handle(handle);
            }
            applying_position.set(true);
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
                    // DOM の更新前にスクロール位置をリセットする（ブラウザの
                    // スクロール保持を避けるため）。
                    if fragment.is_none() {
                        scroll_content_to_top();
                    }
                    set_content.set(html);
                    set_status.set(String::new());
                    // 位置を意図した値で即座に保存する（DOM の更新を待たない）。
                    if fragment.is_none()
                        && let Some(key) = position_key.borrow().as_ref()
                    {
                        save_position(
                            key,
                            &SavedPosition {
                                chapter_index: index,
                                fragment: None,
                                scroll_top: scroll_top.unwrap_or(0),
                                saved_at: 0.0,
                            },
                        );
                    }
                    let generation = current_generation.get().wrapping_add(1);
                    current_generation.set(generation);
                    apply_position_after_render(PositionApplyCtx {
                        chapter_index: index,
                        fragment,
                        scroll_top,
                        expected_images,
                        position_key: Rc::clone(&position_key),
                        applying: Rc::clone(&applying_position),
                        last_apply_time: Rc::clone(&last_apply_time),
                        generation,
                        current_generation: Rc::clone(&current_generation),
                        set_status: Rc::new({
                            let set_status = set_status;
                            move |message: String| set_status.set(message)
                        }),
                        downscale_images: downscale_images.get(),
                        object_urls: Rc::clone(&object_urls),
                        jpeg_quality: jpeg_quality.get(),
                    });
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
            let entries: Vec<(usize, usize, Option<String>, f64)> = history
                .borrow()
                .iter()
                .enumerate()
                .map(|(index, entry)| {
                    (
                        index,
                        entry.chapter_index,
                        entry.fragment.clone(),
                        entry.saved_at,
                    )
                })
                .collect();
            set_history_entries.set(entries);
        })
    };

    // 現在位置を記録して移動先を履歴に積む（再描画は呼び出し側が行う）。
    let push_history_entry = {
        let history = Rc::clone(&history);
        let history_index = Rc::clone(&history_index);
        let sync_history = Rc::clone(&sync_history);
        let history_key_state = Rc::clone(&history_key_state);
        Rc::new(
            move |index: usize, fragment: Option<String>, scroll_top: i32| {
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
                        fragment,
                        scroll_top,
                        saved_at: now_millis(),
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
            },
        )
    };

    // 章を移動し、履歴に積む。
    let navigate_to: Rc<dyn Fn(usize, Option<String>)> = {
        let push_history_entry = Rc::clone(&push_history_entry);
        let render_chapter = Rc::clone(&render_chapter);
        Rc::new(move |index, fragment| {
            push_history_entry(index, fragment.clone(), 0);
            render_chapter(index, fragment, None);
        })
    };

    // 指定した履歴インデックスへジャンプする。戻る・進む・一覧選択で使う。
    let jump_to_history: Rc<dyn Fn(usize)> = {
        let history = Rc::clone(&history);
        let history_index = Rc::clone(&history_index);
        let sync_history = Rc::clone(&sync_history);
        let render_chapter = Rc::clone(&render_chapter);
        let history_key_state = Rc::clone(&history_key_state);
        Rc::new(move |new_index| {
            let entry = history.borrow().get(new_index).cloned();
            let Some(entry) = entry else {
                return;
            };
            let index = history_index.get();
            // 移動前の位置を現在エントリへ記録する。
            if let Some(previous) = history.borrow_mut().get_mut(index)
                && previous.chapter_index == current.get()
            {
                previous.fragment = None;
                previous.scroll_top = content_scroll_top();
            }
            history_index.set(new_index);
            sync_history();
            if let Some(key) = history_key_state.borrow().as_ref() {
                save_history(key, &history.borrow(), history_index.get());
            }
            render_chapter(entry.chapter_index, entry.fragment, Some(entry.scroll_top));
        })
    };

    let back_action: Rc<dyn Fn()> = {
        let history_index = Rc::clone(&history_index);
        let jump_to_history = Rc::clone(&jump_to_history);
        Rc::new(move || {
            let index = history_index.get();
            if index == 0 {
                return;
            }
            jump_to_history(index - 1);
        })
    };

    let forward_action: Rc<dyn Fn()> = {
        let history = Rc::clone(&history);
        let history_index = Rc::clone(&history_index);
        let jump_to_history = Rc::clone(&jump_to_history);
        Rc::new(move || {
            let index = history_index.get();
            if index + 1 >= history.borrow().len() {
                return;
            }
            jump_to_history(index + 1);
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
    let on_history_select = {
        let jump_to_history = Rc::clone(&jump_to_history);
        move |event: web_sys::Event| {
            let Some(select) = event
                .target()
                .and_then(|target| target.dyn_into::<web_sys::HtmlSelectElement>().ok())
            else {
                return;
            };
            let Ok(new_index) = select.value().parse::<usize>() else {
                return;
            };
            jump_to_history(new_index);
        }
    };
    let clear_history: Rc<dyn Fn()> = {
        let history = Rc::clone(&history);
        let history_index = Rc::clone(&history_index);
        let sync_history = Rc::clone(&sync_history);
        let history_key_state = Rc::clone(&history_key_state);
        let current_fragment = Rc::clone(&current_fragment);
        Rc::new(move || {
            // 現在位置だけを残して履歴を消す。
            let entry = HistoryEntry {
                chapter_index: current.get(),
                fragment: current_fragment.borrow().clone(),
                scroll_top: content_scroll_top(),
                saved_at: now_millis(),
            };
            *history.borrow_mut() = vec![entry];
            history_index.set(0);
            sync_history();
            if let Some(key) = history_key_state.borrow().as_ref() {
                save_history(key, &history.borrow(), history_index.get());
            }
        })
    };

    let delete_entry: Rc<dyn Fn()> = {
        let history = Rc::clone(&history);
        let history_index = Rc::clone(&history_index);
        let sync_history = Rc::clone(&sync_history);
        let history_key_state = Rc::clone(&history_key_state);
        Rc::new(move || {
            let mut entries = history.borrow_mut();
            if entries.len() <= 1 {
                return;
            }
            let index = history_index.get();
            if index >= entries.len() {
                return;
            }
            entries.remove(index);
            // 削除した位置に次のエントリが繰り上がる。最後を消した場合は
            // 1 つ前へ戻す。
            let next_index = if index >= entries.len() {
                entries.len() - 1
            } else {
                index
            };
            history_index.set(next_index);
            let raw = entries.clone();
            drop(entries);
            sync_history();
            if let Some(key) = history_key_state.borrow().as_ref() {
                save_history(key, &raw, next_index);
            }
        })
    };

    // 削除ボタンは確認ダイアログを開くだけにする。
    let dialog_ref: NodeRef<leptos::html::Dialog> = NodeRef::new();
    let open_delete_dialog = move |target: &str| {
        set_delete_target.set(target.to_string());
        if let Some(dialog) = dialog_ref.get() {
            let _ = dialog.show_modal();
        }
    };
    let on_history_clear = {
        let open = open_delete_dialog;
        move |_: web_sys::MouseEvent| open("clear")
    };
    let on_history_delete = {
        let open = open_delete_dialog;
        move |_: web_sys::MouseEvent| open("entry")
    };
    let on_confirm_delete = {
        let clear = Rc::clone(&clear_history);
        let delete = Rc::clone(&delete_entry);
        move |_: web_sys::MouseEvent| {
            match delete_target.get().as_str() {
                "clear" => clear(),
                "entry" => delete(),
                _ => {}
            }
            set_delete_target.set(String::new());
            if let Some(dialog) = dialog_ref.get() {
                dialog.close();
            }
        }
    };
    let on_cancel_delete = move |_: web_sys::MouseEvent| {
        set_delete_target.set(String::new());
        if let Some(dialog) = dialog_ref.get() {
            dialog.close();
        }
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
            set_chapter_filter.set(String::new());
            set_chapter_hrefs.set(Vec::new());
            set_landmarks.set(Vec::new());
            set_landmark_options.set(Vec::new());
            set_page_list.set(Vec::new());
            *book.borrow_mut() = None;
            for url in object_urls.borrow_mut().drain(..) {
                let _ = web_sys::Url::revoke_object_url(&url);
            }
            if let Some(url) = cover_url_state.borrow_mut().take() {
                let _ = web_sys::Url::revoke_object_url(&url);
            }
            set_cover_url.set(None);
            set_content_lang.set(None);
            set_remote_resources_warning.set(None);

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
                let hrefs: Vec<String> = spine.iter().map(|(_, href)| href.clone()).collect();
                let options: Vec<(usize, String, Option<String>)> = spine
                    .into_iter()
                    .map(|(index, href)| {
                        let (title, fragment, depth) =
                            toc_entries.remove(&href).unwrap_or((href.clone(), None, 0));
                        let indent = "　".repeat(depth.min(8));
                        (index, format!("{indent}{title}"), fragment)
                    })
                    .collect();

                let landmarks = opened
                    .landmarks()
                    .ok()
                    .and_then(|value| serde_wasm_bindgen::from_value::<Vec<TocEntry>>(value).ok())
                    .unwrap_or_default();

                let page_list = opened
                    .page_list()
                    .ok()
                    .and_then(|value| serde_wasm_bindgen::from_value::<Vec<TocEntry>>(value).ok())
                    .unwrap_or_default();

                let language = opened.language();
                let has_remote_resources = opened.has_remote_resources();
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
                                    saved_at: entry.saved_at,
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
                            saved_at: now_millis(),
                        }],
                        0,
                    )
                });

                *history.borrow_mut() = history_entries;
                history_index.set(history_idx);
                sync_history();

                *book.borrow_mut() = Some(opened);
                set_content_lang.set(language);
                set_remote_resources_warning.set(if has_remote_resources {
                    Some(String::from("This EPUB declares remote resources."))
                } else {
                    None
                });
                set_chapters.set(options);
                set_chapter_hrefs.set(hrefs);
                set_landmarks.set(landmarks.clone());
                let lopts: Vec<(usize, String)> = landmarks
                    .iter()
                    .enumerate()
                    .map(|(index, entry)| (index, entry.title.clone()))
                    .collect();
                set_landmark_options.set(lopts);
                set_page_list.set(page_list);
                render_chapter(start_index, start_fragment, Some(start_scroll));
                // 章描画で status がクリアされるため、その後に警告を出す。
                if let Some(message) = remote_resources_warning.get() {
                    set_status.set(message);
                }
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
        let swipe_threshold = swipe_threshold;
        let vertical_scroll_threshold = vertical_scroll_threshold;
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
            if (dy.abs() as usize) >= vertical_scroll_threshold.get() {
                return;
            }
            if (dx.abs() as usize) < swipe_threshold.get() {
                return;
            }
            if dx < 0.0 {
                next_action();
            } else {
                prev_action();
            }
        }
    };

    let on_landmark_select = {
        let navigate_to = Rc::clone(&navigate_to);
        move |event: web_sys::Event| {
            let Some(select) = event
                .target()
                .and_then(|target| target.dyn_into::<web_sys::HtmlSelectElement>().ok())
            else {
                return;
            };
            let Ok(index) = select.value().parse::<usize>() else {
                return;
            };
            let entry = landmarks.get().get(index).cloned();
            let Some(entry) = entry else {
                return;
            };
            let path = entry.href.split('#').next().unwrap_or(&entry.href);
            let target_index = chapter_hrefs.get().iter().position(|href| {
                href.rsplit('/').next().unwrap_or(href) == path.rsplit('/').next().unwrap_or(path)
            });
            if let Some(target_index) = target_index {
                navigate_to(target_index, entry.fragment);
            }
        }
    };

    let on_page_go = {
        let navigate_to = Rc::clone(&navigate_to);
        move |_| {
            let Some(window) = web_sys::window() else {
                return;
            };
            let Some(document) = window.document() else {
                return;
            };
            let Ok(Some(element)) = document.query_selector("#reader-page-input") else {
                return;
            };
            let Ok(input) = element.dyn_into::<web_sys::HtmlInputElement>() else {
                return;
            };
            let page = input.value();
            let entry = page_list
                .get()
                .iter()
                .find(|entry| entry.title == page)
                .cloned();
            let Some(entry) = entry else {
                set_status.set(format!("Page {} not found.", page));
                return;
            };
            let path = entry.href.split('#').next().unwrap_or(&entry.href);
            let target_index = chapter_hrefs.get().iter().position(|href| {
                href.rsplit('/').next().unwrap_or(href) == path.rsplit('/').next().unwrap_or(path)
            });
            if let Some(target_index) = target_index {
                navigate_to(target_index, entry.fragment);
            } else {
                set_status.set(format!("Page {} not found.", page));
            }
        }
    };

    let on_content_click = {
        let book = Rc::clone(&book);
        let navigate_to = Rc::clone(&navigate_to);
        let push_history_entry = Rc::clone(&push_history_entry);
        let current_fragment = Rc::clone(&current_fragment);
        let history = Rc::clone(&history);
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
                // 同じフラグメントへの再移動なら履歴を増やさない。
                if current_fragment.borrow().as_deref() != Some(fragment) {
                    push_history_entry(current.get(), Some(fragment.to_string()), 0);
                    *current_fragment.borrow_mut() = Some(fragment.to_string());
                }
                scroll_to_fragment(fragment);
                // スクロール後の実位置を新しい履歴エントリへ反映する。
                if let Some(entry) = history.borrow_mut().last_mut() {
                    entry.scroll_top = content_scroll_top();
                }
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
    let on_scroll = {
        let position_key = Rc::clone(&position_key_state);
        let scroll_save_timer = Rc::clone(&scroll_save_timer);
        let applying_position = Rc::clone(&applying_position);
        let last_apply_time = Rc::clone(&last_apply_time);
        move |_event: web_sys::Event| {
            if applying_position.get() {
                return;
            }
            // 位置適用直後は保存しない（スクロールが落ち着くのを待つ）。
            if now_millis() - last_apply_time.get() < 250.0 {
                return;
            }
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
            follow_system: follow_system.get(),
            swipe_threshold: swipe_threshold.get(),
            vertical_scroll_threshold: vertical_scroll_threshold.get(),
            downscale_images: downscale_images.get(),
            jpeg_quality: jpeg_quality.get(),
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
            // 手動で切り替えたら OS 追従は解除する。
            if follow_system.get() {
                set_follow_system.set(false);
            }
            set_dark.set(!dark.get());
            persist();
        })
    };
    let dark_auto_action: Rc<dyn Fn()> = {
        let persist = Rc::clone(&persist_settings);
        Rc::new(move || {
            let next = !follow_system.get();
            set_follow_system.set(next);
            if next {
                // 自動に切り替えたら、現在の OS 設定を反映する。
                set_system_dark.set(system_prefers_dark());
            }
            persist();
        })
    };
    let on_swipe_threshold_change = {
        let persist = Rc::clone(&persist_settings);
        move |event: web_sys::Event| {
            let Some(input) = event
                .target()
                .and_then(|target| target.dyn_into::<web_sys::HtmlInputElement>().ok())
            else {
                return;
            };
            let value = input.value().parse::<usize>().unwrap_or(50).clamp(30, 120);
            set_swipe_threshold.set(value);
            persist();
        }
    };
    let on_vertical_scroll_threshold_change = {
        let persist = Rc::clone(&persist_settings);
        move |event: web_sys::Event| {
            let Some(input) = event
                .target()
                .and_then(|target| target.dyn_into::<web_sys::HtmlInputElement>().ok())
            else {
                return;
            };
            let value = input.value().parse::<usize>().unwrap_or(30).clamp(10, 80);
            set_vertical_scroll_threshold.set(value);
            persist();
        }
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
    let on_dark_auto = {
        let action = Rc::clone(&dark_auto_action);
        move |_: web_sys::MouseEvent| action()
    };
    let on_jpeg_quality_change = {
        let persist = Rc::clone(&persist_settings);
        move |event: web_sys::Event| {
            let Some(input) = event
                .target()
                .and_then(|target| target.dyn_into::<web_sys::HtmlInputElement>().ok())
            else {
                return;
            };
            let value = input.value().parse::<usize>().unwrap_or(85).clamp(50, 100);
            set_jpeg_quality.set(value);
            persist();
        }
    };
    let on_downscale_toggle = {
        let persist = Rc::clone(&persist_settings);
        move |_: web_sys::MouseEvent| {
            set_downscale_images.set(!downscale_images.get());
            persist();
        }
    };
    let on_reset_settings = move |_: web_sys::MouseEvent| {
        set_font_scale.set(DEFAULT_FONT_SCALE);
        set_line_height.set(DEFAULT_LINE_HEIGHT);
        set_dark.set(false);
        set_follow_system.set(false);
        set_swipe_threshold.set(50);
        set_vertical_scroll_threshold.set(30);
        set_downscale_images.set(false);
        set_jpeg_quality.set(85);
        save_settings(&ReaderSettings::default());
        set_status.set("Display settings reset.".to_string());
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
                Ok((count, history_count)) => {
                    set_status.set(format!(
                        "Imported {count} reading positions and {history_count} histories."
                    ));
                }
                Err(error) => set_status.set(error),
            }
        });
    };
    // view マクロ内の比較で `>` を使わないよう、条件を外で計算する。
    let font_larger_disabled = Memo::new(move |_| font_scale.get() + 1 >= FONT_SCALES.len());

    // キーボード操作。フォーカスが入力欄にあるときは無視する。
    // ショートカット一覧ダイアログ（? キーとボタンから開く）。
    let help_dialog: NodeRef<leptos::html::Dialog> = NodeRef::new();
    let open_help = move || {
        if let Some(dialog) = help_dialog.get() {
            let _ = dialog.show_modal();
        }
    };
    let on_help_click = {
        let open = open_help;
        move |_: web_sys::MouseEvent| open()
    };
    let key_handler = {
        let prev_action = Rc::clone(&prev_action);
        let next_action = Rc::clone(&next_action);
        let back_action = Rc::clone(&back_action);
        let forward_action = Rc::clone(&forward_action);
        let font_smaller_action = Rc::clone(&font_smaller_action);
        let font_larger_action = Rc::clone(&font_larger_action);
        let dark_action = Rc::clone(&dark_action);
        let help_dialog = help_dialog;
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
                "?" => {
                    if let Some(dialog) = help_dialog.get() {
                        let _ = dialog.show_modal();
                    }
                    true
                }
                "/" => {
                    if let Some(window) = web_sys::window()
                        && let Some(document) = window.document()
                        && let Some(input) = document
                            .query_selector("#reader-chapter-filter")
                            .ok()
                            .flatten()
                        && let Some(html_input) = input.dyn_into::<web_sys::HtmlInputElement>().ok()
                    {
                        html_input.focus();
                    }
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

    // 章タイトルの部分一致で絞り込む。現在の章は選択状態を保つため残す。
    let filtered_chapters = Memo::new(move |_| {
        let filter = chapter_filter.get().trim().to_lowercase();
        let active = current.get();
        chapters
            .get()
            .into_iter()
            .filter(|(index, title, _)| {
                *index == active || filter.is_empty() || title.to_lowercase().contains(&filter)
            })
            .collect::<Vec<_>>()
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
            class:reader-dark=move || effective_dark.get()
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
                <button id="reader-theme-auto" class="btn" title="Follow the system color scheme" aria-label="Follow the system color scheme" aria-pressed=move || if follow_system.get() { "true" } else { "false" } on:click=on_dark_auto>{move || if follow_system.get() { "Auto:on" } else { "Auto:off" }}</button>
                <button id="reader-downscale" class="btn" title="Downscale oversized images" aria-label="Downscale oversized images" aria-pressed=move || if downscale_images.get() { "true" } else { "false" } on:click=on_downscale_toggle>{move || if downscale_images.get() { "Downscale:on" } else { "Downscale:off" }}</button>
                <input id="reader-jpeg-quality" class="reader-chapter-select" type="number" min="50" max="100" aria-label="JPEG quality" title="JPEG quality (%)" on:change=on_jpeg_quality_change style="max-width: 6rem;" prop:value=move || jpeg_quality.get().to_string()/>
                <button id="reader-clear-data" class="btn" title="Clear saved reading positions" aria-label="Clear saved reading positions" on:click=on_clear_data>"Clear data"</button>
                <button id="reader-reset-settings" class="btn" title="Reset display settings" aria-label="Reset display settings" on:click=on_reset_settings>"Reset settings"</button>
                <button id="reader-help" class="btn" title="Keyboard shortcuts (?)" aria-label="Keyboard shortcuts" aria-keyshortcuts="?" on:click=on_help_click>"?"</button>
                <button id="reader-export" class="btn" title="Export saved reading positions" aria-label="Export saved reading positions" on:click=on_export>"Export"</button>
                <button id="reader-import" class="btn" title="Import saved reading positions" aria-label="Import saved reading positions" on:click=on_import_click>"Import"</button>
                <input id="reader-import-file" class="reader-file-input" type="file" accept=".json,application/json" on:change=on_import_change/>
                <button id="reader-back" class="btn" aria-label="Go back" aria-keyshortcuts="Alt+ArrowLeft" on:click=on_history_back disabled=move || !can_back.get()>"Back"</button>
                <button id="reader-forward" class="btn" aria-label="Go forward" aria-keyshortcuts="Alt+ArrowRight" on:click=on_history_forward disabled=move || !can_forward.get()>"Forward"</button>
                <select id="reader-history" class="reader-chapter-select" aria-label="History" on:change=on_history_select prop:disabled=move || history_state.get().0 <= 1>
                    <For
                        each=move || history_entries.get()
                        key=|(index, chapter_index, fragment, _)| (*index, *chapter_index, fragment.clone())
                        children=move |(index, chapter_index, fragment, saved_at)| {
                            let label = chapters
                                .get()
                                .get(chapter_index)
                                .map(|(_, title, _)| title.trim_start().to_string())
                                .unwrap_or_else(|| format!("Chapter {}", chapter_index + 1));
                            let label = match fragment {
                                Some(fragment) => format!("{label}#{fragment}"),
                                None => label,
                            };
                            // 旧データ（savedAt=0）は日時を省略する。
                            let time = format_saved_at(saved_at);
                            let label = if time.is_empty() {
                                label
                            } else {
                                format!("{label} — {time}")
                            };
                            view! {
                                <option value=index.to_string() prop:selected=move || history_state.get().1 == index>
                                    {label}
                                </option>
                            }
                        }
                    />
                </select>
                <button class="btn" aria-label="Previous chapter" aria-keyshortcuts="[" on:click=on_previous disabled=move || current.get() == 0>"◀"</button>
                <input id="reader-chapter-filter" class="reader-chapter-select" type="search" aria-label="Filter chapters" placeholder="Filter chapters" on:input=move |event: web_sys::Event| {
                    if let Some(input) = event.target().and_then(|target| target.dyn_into::<web_sys::HtmlInputElement>().ok()) {
                        set_chapter_filter.set(input.value());
                    }
                } style="max-width: 10rem;"/>
                <select id="reader-chapter-select" class="reader-chapter-select" aria-label="Chapter" on:change=on_select prop:disabled=move || chapters.get().is_empty()>
                    <For
                        each=move || filtered_chapters.get()
                        key=|(index, _, _)| *index
                        children=move |(index, title, _)| view! {
                            <option value=index.to_string() prop:selected=move || current.get() == index>
                                {title}
                            </option>
                        }
                    />
                </select>
                <select id="reader-landmarks" class="reader-chapter-select" aria-label="Landmarks" on:change=on_landmark_select prop:disabled=move || landmark_options.get().is_empty()>
                    <option value="" disabled selected>"Landmarks"</option>
                    <For
                        each=move || landmark_options.get()
                        key=|(index, _)| *index
                        children=move |(index, label)| view! {
                            <option value=index.to_string()>{label.clone()}</option>
                        }
                    />
                </select>
                <button id="reader-history-delete" class="btn" aria-label="Delete history entry" title="Delete the selected history entry" on:click=on_history_delete disabled=move || history_state.get().0 <= 1>"Delete"</button>
                <button id="reader-history-clear" class="btn" aria-label="Clear history" title="Clear navigation history" on:click=on_history_clear disabled=move || history_state.get().0 <= 1>"Clear"</button>
                <input id="reader-page-input" class="reader-chapter-select" type="text" aria-label="Page" placeholder="Page" prop:disabled=move || page_list.get().is_empty() style="max-width: 6rem;"/>
                <input id="reader-swipe-threshold" class="reader-chapter-select" type="number" min="30" max="120" aria-label="Swipe threshold" title="Swipe threshold (px)" on:change=on_swipe_threshold_change style="max-width: 6rem;" prop:value=move || swipe_threshold.get().to_string()/>
                <input id="reader-vertical-threshold" class="reader-chapter-select" type="number" min="10" max="80" aria-label="Vertical scroll threshold" title="Vertical scroll threshold (px)" on:change=on_vertical_scroll_threshold_change style="max-width: 6rem;" prop:value=move || vertical_scroll_threshold.get().to_string()/>
                <button id="reader-page-go" class="btn" aria-label="Go to page" on:click=on_page_go prop:disabled=move || page_list.get().is_empty()>"Go"</button>
                <button class="btn" aria-label="Next chapter" aria-keyshortcuts="]" on:click=on_next disabled=move || {
                    let index = current.get();
                    !chapters.get().iter().any(|(chapter, _, _)| *chapter > index)
                }>
                    "▶"
                </button>
                <span id="reader-chapter-title" class="reader-chapter-title">{move || current_title.get()}</span>
            </div>
            <p class="reader-status" role="status" aria-live="polite">{move || status.get()}</p>
            // 削除の確認ダイアログ。ネイティブ <dialog> のモーダル機能を使う。
            <dialog id="reader-delete-dialog" node_ref=dialog_ref aria-labelledby="reader-delete-title">
                <h2 id="reader-delete-title">{move || if delete_target.get() == "clear" { "Clear all history?" } else { "Delete this history entry?" }}</h2>
                <p>{move || if delete_target.get() == "clear" {
                    "The current position will remain and all other history will be removed."
                } else {
                    "The selected history entry will be removed."
                }}</p>
                <div class="dialog-actions">
                    <button type="button" class="btn" autofocus on:click=on_cancel_delete>"Cancel"</button>
                    <button type="button" class="btn btn-danger" id="reader-delete-confirm" on:click=on_confirm_delete>"Delete"</button>
                </div>
            </dialog>
            // キーボードショートカットの一覧。
            <dialog id="reader-help-dialog" node_ref=help_dialog aria-labelledby="reader-help-title">
                <h2 id="reader-help-title">"Keyboard shortcuts"</h2>
                <ul class="shortcut-list">
                    <li><kbd>"["</kbd> " / " <kbd>"]"</kbd> " — Previous / next chapter"</li>
                    <li><kbd>"+"</kbd> " / " <kbd>"-"</kbd> " — Larger / smaller text"</li>
                    <li><kbd>"d"</kbd> " — Toggle dark mode"</li>
                    <li><kbd>"Alt+←"</kbd> " / " <kbd>"Alt+→"</kbd> " — Back / forward"</li>
                    <li><kbd>"/"</kbd> " — Focus chapter filter"</li>
                    <li><kbd>"?"</kbd> " — Show this help"</li>
                </ul>
                <div class="dialog-actions">
                    <button type="button" class="btn" autofocus on:click=move |_| {
                        if let Some(dialog) = help_dialog.get() { dialog.close(); }
                    }>"Close"</button>
                </div>
            </dialog>
            <div class="reader-body">
                <Show when=move || cover_url.get().is_some()>
                    <aside class="reader-sidebar">
                        <img class="reader-cover" src=move || cover_url.get().unwrap_or_default() alt="Cover"/>
                    </aside>
                </Show>
                <div class="reader-content" role="region" aria-label="Chapter content" aria-labelledby="reader-chapter-title" tabindex="0" on:click=on_content_click on:scroll=on_scroll on:touchstart=on_touch_start on:touchend=on_touch_end>
                    <div class="document-content reader-html" inner_html=move || content.get() lang=move || content_lang.get()></div>
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
