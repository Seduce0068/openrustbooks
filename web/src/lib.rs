//! openrustbooks-web: Web application (Wasm) for openrustbooks.
//!
//! This crate compiles to WebAssembly and provides the browser-based
//! version of the application using Leptos.

use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::hooks::use_params_map;
use leptos_router::path;
#[cfg(target_arch = "wasm32")]
use openrustbooks_core::WasmEpubBook;
use openrustbooks_core::init;
#[cfg(target_arch = "wasm32")]
use openrustbooks_core::sanitize::sanitize_epub_html;
use openrustbooks_core::wasm::render_markdown_wasm;
#[cfg(target_arch = "wasm32")]
use openrustbooks_core::wasm::sanitize_svg_wasm;
// Leptos の非公開 re-export ではなく、直接依存の send_wrapper クレートを使う（ADR-0166 / TD-M07）。
use send_wrapper::SendWrapper;
use wasm_bindgen::prelude::*;
// インポート JSON の構造制限は Value を組み立てずにストリーミングで走査する。
// wasm32 とネイティブの単体テストの両方で使うため、宣言側も同じ条件でゲートする。
#[cfg(any(target_arch = "wasm32", test))]
use serde::de::DeserializeSeed;

/// E2E の起動診断用に Wasm の実行段階へタイムスタンプを付ける。
/// wasm-bindgen の init() が完了したか、Leptos のマウントまで進んだかを
/// ブラウザ側から判別できるようにする（本番の挙動には影響しない）。
#[cfg(target_arch = "wasm32")]
fn mark_boot_stage(name: &str) {
    if let Some(window) = web_sys::window() {
        let _ = js_sys::Reflect::set(
            &window,
            &JsValue::from_str(name),
            &JsValue::from_f64(js_sys::Date::now()),
        );
    }
}

// Initialize panic hook and tracing for Wasm
#[wasm_bindgen(start)]
pub fn main() {
    init();
    leptos::logging::log!("openrustbooks-web starting...");
    #[cfg(target_arch = "wasm32")]
    mark_boot_stage("__orbRustStartedAt");

    // Mount the Leptos app
    mount_to_body(|| view! { <App/> });

    #[cfg(target_arch = "wasm32")]
    mark_boot_stage("__orbRustMountedAt");
    leptos::logging::log!("openrustbooks-web mounted");
}

/// Main application component
#[component]
fn App() -> impl IntoView {
    view! {
        <Router>
            <Routes fallback=|| view! { <p class="not-found">"ページが見つかりません"</p> }>
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
                <nav aria-label="メインナビゲーション">
                    <a href="/reader/sample" title="サンプル文書をリーダーで開く" aria-label="サンプル文書をリーダーで開く">"リーダー（サンプル）"</a>
                    <a href="/editor/sample" title="サンプル文書をエディターで編集" aria-label="サンプル文書をエディターで編集">"エディター（サンプル）"</a>
                    <a href="/settings" title="設定を開く" aria-label="設定を開く">"設定"</a>
                    <a href="/about" title="このアプリについて表示" aria-label="このアプリについて表示">"このアプリについて"</a>
                </nav>
            </header>
            <main class="app-main">
                <section class="document-browser">
                    <h2>"文書"</h2>
                    <div class="toolbar">
                        <button
                            class="btn btn-primary"
                            disabled=true
                            title="新規ファイル（この初版では利用できません）"
                            aria-label="新規ファイル（この初版では利用できません）"
                        >
                            "新規ファイル"
                        </button>
                        <button
                            class="btn btn-secondary"
                            disabled=true
                            title="フォルダーを開く（この初版では利用できません）"
                            aria-label="フォルダーを開く（この初版では利用できません）"
                        >
                            "フォルダーを開く"
                        </button>
                        <button
                            class="btn btn-secondary"
                            disabled=true
                            title="ファイルを開く（この初版では利用できません）"
                            aria-label="ファイルを開く（この初版では利用できません）"
                        >
                            "ファイルを開く"
                        </button>
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
                    <a href="/" title="ホームへ戻る" aria-label="ホームへ戻る">"← ホーム"</a>
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
                    <a href="/" title="ホームへ戻る" aria-label="ホームへ戻る">"← ホーム"</a>
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
                    <a href="/" title="ホームへ戻る" aria-label="ホームへ戻る">"← ホーム"</a>
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
                    <a href="/" title="ホームへ戻る" aria-label="ホームへ戻る">"← ホーム"</a>
                </nav>
            </header>
            <main class="app-main">
                <div class="about-content">
                    <h2>"openrustbooks"</h2>
                    <p>"Version: " {env!("CARGO_PKG_VERSION")}</p>
                    <p>"Rust で作られた文書リーダー / エディターです"</p>
                    <p>"EPUB reading: available"</p>
                    <p>"TXT / Markdown / PDF / ZIP: planned"</p>
                    <hr/>
                    <p>"Built with: Tauri, Leptos, Rust"</p>
                    <p id="about-license">"License: AGPL-3.0-or-later"</p>
                    <p>
                        <a href="#about-license" title="ライセンス情報へ移動" aria-label="ライセンス情報へ移動">
                            "ライセンス情報を見る"
                        </a>
                        " / "
                        <a
                            href="https://www.gnu.org/licenses/agpl-3.0.html"
                            target="_blank"
                            rel="noopener noreferrer"
                            title="AGPL-3.0 の全文を新しいタブで開く"
                            aria-label="AGPL-3.0 の全文を新しいタブで開く"
                        >
                            "AGPL-3.0 全文"
                        </a>
                    </p>
                </div>
            </main>
        </div>
    }
}

/// File tree component
///
/// 初版ではファイル一覧を接続していない。以前の固定ダミー項目は
/// クリック可能な誤解を与えるため削除し、状態を説明する固定文言だけを
/// 表示する（ADR-0127 の未配線方針に合わせる）。
#[component]
fn FileTree() -> impl IntoView {
    view! {
        <p class="file-tree-placeholder">"ファイル一覧は未接続です"</p>
    }
}

/// 最小構成のエディタ（ADR-0127: CodeMirror は導入しない）。
///
/// `on:input` で textarea の値を `content` シグナルへ反映し、`<Preview/>`
/// が同じシグナルを読むためリアルタイムに更新される。`prop:value` は
/// 意図的に使わない: 同じ文字列を再代入するとブラウザがキャレットを
/// 末尾へ戻すため、textarea 自身を入力値の唯一の保持元とする（ADR-0161）。
/// そのため下書きの復元時は `textarea.set_value` と `set_content` の両方を行う。
/// 上限超の下書きを復元した場合は本文をそのまま戻し、固定文言の警告だけを
/// status へ出す（本文・サイズ値は出さない）。
/// Save はブラウザ内の localStorage へ下書きを保存し、失敗は ADR-0154 の
/// 共通通知で利用者へ伝える。入力後は 800ms のデバウンスでも自動保存し、
/// 保存が完了するまで status に未保存状態を出す（R6 TD-H05）。
/// 他のデスクトップ専用操作は disabled にする。
#[component]
fn Editor(doc_id: Signal<String>) -> impl IntoView {
    let (content, set_content) = signal(String::new());
    let (status, set_status) = signal(String::new());

    // 保存失敗（ADR-0154 / 0164）のうち Editor 向けだけを受け取り、
    // 利用者向けの status に表示する。Reader 向けイベントは受け取らない。
    // dispatch_event は同期なので、Save ハンドラの report_save_failure の
    // 直後にこのリスナーが走る。本文はイベントにも status にも載せない。
    if let Some(window) = web_sys::window() {
        let event_name = SaveErrorTarget::Editor.event_name();
        let on_save_error = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            set_status.set(SAVE_ERROR_MESSAGE.to_string());
        }) as Box<dyn FnMut(web_sys::Event)>);
        let _ = window
            .add_event_listener_with_callback(event_name, on_save_error.as_ref().unchecked_ref());
        // SPA 再マウントで古い signal への set が残らないよう、Owner 破棄時に解除する。
        // on_cleanup は Send + Sync を要求するため、!Send な Window/Closure を
        // SendWrapper で包んだ Option で保持し、remove の後に drop する。
        let listener = SendWrapper::new(Some((window, event_name, on_save_error)));
        on_cleanup(move || {
            if let Some((window, event_name, callback)) = listener.take() {
                let _ = window.remove_event_listener_with_callback(
                    event_name,
                    callback.as_ref().unchecked_ref(),
                );
            }
        });
    }

    // 入力を EDITOR_AUTOSAVE_DELAY_MS だけデバウンスして自動保存する。
    // タイマーは doc_id 変更時と Owner 破棄時に必ず解除し、別ドキュメントへ
    // 本文を書き込まないようにする。世代カウンタで、解除直前にキューされた
    // 古いコールバックの発火も無効化する。
    let autosave_timer = std::rc::Rc::new(std::cell::Cell::new(None::<i32>));
    let autosave_generation = std::rc::Rc::new(std::cell::Cell::new(0_u32));
    // 直前の自動保存で試行した本文。同一本文の再試行で console.error が
    // 連発するのを防ぐ（上限超・保存失敗の両方で同じ本文を保持する）。
    let autosave_last_text = std::rc::Rc::new(std::cell::RefCell::new(None::<String>));

    {
        let timer = std::rc::Rc::clone(&autosave_timer);
        let generation = std::rc::Rc::clone(&autosave_generation);
        let last_text = std::rc::Rc::clone(&autosave_last_text);
        Effect::new(move |_| {
            let _ = doc_id.get(); // doc_id 変更を購読するためだけに読む
            if let Some(window) = web_sys::window()
                && let Some(handle) = timer.take()
            {
                window.clear_timeout_with_handle(handle);
            }
            generation.set(generation.get() + 1);
            // 別文書へ切り替えたら直前の自動保存本文も破棄する。残っていると
            // 同じ本文を入力したときに自動保存がスキップされる（lost update）。
            *last_text.borrow_mut() = None;
        });
    }

    {
        let timer = std::rc::Rc::clone(&autosave_timer);
        // on_cleanup は Send + Sync を要求するため、!Send な Rc を
        // SendWrapper で包んで保持する（既存のイベントリスナーと同じ方式）。
        let state = SendWrapper::new(timer);
        on_cleanup(move || {
            if let Some(window) = web_sys::window()
                && let Some(handle) = (**state).take()
            {
                window.clear_timeout_with_handle(handle);
            }
        });
    }

    // view! 内の各ハンドラへ個別に clone を渡し、move による所有権競合を避ける。
    let save_click_timer = std::rc::Rc::clone(&autosave_timer);
    let save_click_last_text = std::rc::Rc::clone(&autosave_last_text);
    let input_timer = std::rc::Rc::clone(&autosave_timer);
    let input_generation = std::rc::Rc::clone(&autosave_generation);
    let input_last_text = std::rc::Rc::clone(&autosave_last_text);

    // エディタ表示時に下書きを復元する。prop:value を使わない契約を保つため
    // textarea 自身へ set_value し、シグナルも同じ本文で揃える。
    Effect::new(move |_| {
        let Some(draft) = load_editor_draft(&doc_id.get()) else {
            return;
        };
        set_content.set(draft.clone());
        // 上限超の下書きでも本文はそのまま復元し、固定文言だけで警告する。
        // 本文・サイズ値は status / console へ出さない。
        if draft.len() > EDITOR_DRAFT_MAX_BYTES {
            set_status.set(EDITOR_DRAFT_TOO_LARGE_RESTORED_MESSAGE.to_string());
        }
        let Some(window) = web_sys::window() else {
            return;
        };
        let Some(document) = window.document() else {
            return;
        };
        let Some(element) = document.get_element_by_id("editor-input") else {
            return;
        };
        if let Ok(textarea) = element.dyn_into::<web_sys::HtmlTextAreaElement>() {
            textarea.set_value(&draft);
        }
    });

    view! {
        <div class="editor-container" data-doc-id=move || doc_id.get()>
            <div class="editor-toolbar">
                <button
                    class="btn"
                    title="Save draft（ブラウザー内に下書きを保存 / browser local）"
                    aria-label="Save draft（下書きを保存）"
                    on:click=move |_| {
                        // 手動保存が保留中の自動保存に置き換わるためタイマーを解除する。
                        // これで「下書きを保存しました」が自動保存の文言へ上書きされない。
                        if let Some(window) = web_sys::window()
                            && let Some(handle) = save_click_timer.take()
                        {
                            window.clear_timeout_with_handle(handle);
                        }
                        let id = doc_id.get();
                        let text = content.get();
                        if save_editor_draft(&id, &text) {
                            // 手動保存した本文を自動保存の比較基準へ同期する。
                            // 保存済みのため、同一本文の自動保存はスキップされる。
                            *save_click_last_text.borrow_mut() = Some(text.clone());
                            set_status.set("下書きを保存しました".to_string());
                        } else if text.len() > EDITOR_DRAFT_MAX_BYTES {
                            // 本文・サイズ値は status へ出さず、理由は固定文言だけで伝える。
                            set_status.set(EDITOR_DRAFT_TOO_LARGE_MESSAGE.to_string());
                        } else {
                            set_status.set(SAVE_ERROR_MESSAGE.to_string());
                        }
                    }
                >
                    "下書きを保存"
                </button>
                <button
                    class="btn"
                    disabled=true
                    title="AI 生成（desktop only, not implemented in the browser build）"
                    aria-label="AI 生成（デスクトップ版のみ・ブラウザー版では未実装）"
                >
                    "AI 生成"
                </button>
                <button
                    class="btn"
                    disabled=true
                    title="AI 推敲（desktop only, not implemented in the browser build）"
                    aria-label="AI 推敲（デスクトップ版のみ・ブラウザー版では未実装）"
                >
                    "AI 推敲"
                </button>
                <button
                    class="btn"
                    disabled=true
                    title="静的検査（desktop only, not implemented in the browser build）"
                    aria-label="静的検査（デスクトップ版のみ・ブラウザー版では未実装）"
                >
                    "静的検査"
                </button>
                <select
                    class="format-select"
                    disabled=true
                    title="書式選択（desktop only, not implemented in the browser build）"
                    aria-label="書式選択（デスクトップ版のみ・ブラウザー版では未実装）"
                >
                    <option value="markdown">"Markdown"</option>
                    <option value="txt">"プレーンテキスト"</option>
                </select>
            </div>
            <p class="editor-status" role="status" aria-live="polite">{move || status.get()}</p>
            <textarea
                id="editor-input"
                class="editor-pane"
                aria-label="文書エディター"
                placeholder="ここに入力するとプレビューへ反映されます…"
                on:input=move |event: web_sys::Event| {
                    if let Some(textarea) = event
                        .target()
                        .and_then(|target| target.dyn_into::<web_sys::HtmlTextAreaElement>().ok())
                    {
                        set_content.set(textarea.value());
                        // 保存が完了するまでは未保存状態を伝える。aria-live="polite" の
                        // 既存領域なので、同じ文言の再 set は読み上げの無駄を避け
                        // スキップする（modern-web-guidance: accessibility）。
                        if status.get() != EDITOR_UNSAVED_MESSAGE {
                            set_status.set(EDITOR_UNSAVED_MESSAGE.to_string());
                        }

                        // 古いタイマーを解除してから作り直す（デバウンス）。
                        if let Some(window) = web_sys::window() {
                            if let Some(handle) = input_timer.take() {
                                window.clear_timeout_with_handle(handle);
                            }
                            let generation = input_generation.get();
                            let timer = std::rc::Rc::clone(&input_timer);
                            let generation_cell = std::rc::Rc::clone(&input_generation);
                            let last_text = std::rc::Rc::clone(&input_last_text);
                            let id = doc_id.get();
                            let callback = Closure::once_into_js(move || {
                                timer.set(None); // 発火済みなのでハンドルを破棄する
                                // doc_id 変更 / cleanup で無効化された古い発火は捨てる。
                                if generation_cell.get() != generation || doc_id.get() != id {
                                    return;
                                }
                                let text = content.get();
                                // 同一本文の再試行は保存も通知もしない（失敗時の
                                // console.error 連発を防ぐ）。
                                if last_text.borrow().as_deref() == Some(text.as_str()) {
                                    return;
                                }
                                *last_text.borrow_mut() = Some(text.clone());
                                // 上限超は自動保存しない（setItem も共通通知も出さず、
                                // 手動 Save と同じ固定文言だけを表示する）。
                                if text.len() > EDITOR_DRAFT_MAX_BYTES {
                                    if status.get() != EDITOR_DRAFT_TOO_LARGE_MESSAGE {
                                        set_status.set(EDITOR_DRAFT_TOO_LARGE_MESSAGE.to_string());
                                    }
                                    return;
                                }
                                if save_editor_draft(&id, &text)
                                    && status.get() != EDITOR_AUTOSAVED_MESSAGE
                                {
                                    set_status.set(EDITOR_AUTOSAVED_MESSAGE.to_string());
                                }
                                // 失敗時は save_editor_draft の共通通知（イベント
                                // リスナーが SAVE_ERROR_MESSAGE を status へ出す）に任せ、
                                // 自動保存固有の成功文言は出さない。
                            });
                            if let Ok(handle) = window
                                .set_timeout_with_callback_and_timeout_and_arguments_0(
                                    callback.unchecked_ref(),
                                    EDITOR_AUTOSAVE_DELAY_MS,
                                )
                            {
                                input_timer.set(Some(handle));
                            }
                        }
                    }
                }
            ></textarea>
            <div class="preview-pane">
                <Preview content=content set_content=set_content/>
            </div>
        </div>
    }
}

/// core の `has_scheme` と同じ規則でスキーム付き URL かどうかを判定する。
///
/// RFC 3986 の scheme（先頭は英字、以後は英数か `+` `-` `.`）を `:` の前から
/// 検出する。`http://` などの前方一致だけでは `file:` / `data:` / `javascript:`
/// を見逃すため、判定を core と同じ規則に揃える。
#[cfg(any(target_arch = "wasm32", test))]
fn has_external_scheme(value: &str) -> bool {
    let Some((prefix, _)) = value.split_once(':') else {
        return false;
    };
    let mut chars = prefix.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    first.is_ascii_alphabetic()
        && chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
}

/// `#fragment` の `%XX` を復号する。不正な並び・不正な UTF-8 なら `None`。
#[cfg(any(target_arch = "wasm32", test))]
fn decode_fragment(fragment: &str) -> Option<String> {
    let bytes = fragment.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut position = 0;
    while position < bytes.len() {
        if bytes[position] == b'%' {
            if position + 2 >= bytes.len() {
                return None;
            }
            let (Some(high), Some(low)) = (
                hex_digit(bytes[position + 1]),
                hex_digit(bytes[position + 2]),
            ) else {
                return None;
            };
            decoded.push(high * 16 + low);
            position += 3;
        } else {
            decoded.push(bytes[position]);
            position += 1;
        }
    }
    String::from_utf8(decoded).ok()
}

/// 16 進 1 桁を数値にする。
#[cfg(any(target_arch = "wasm32", test))]
fn hex_digit(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
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

/// `resolve_images()` が Blob URL を設定した `<img>` に付与する描画識別子。
/// error リスナーが現在の描画の画像かを判定するために使う（ADR-0225）。
#[cfg(target_arch = "wasm32")]
const IMAGE_RENDER_GENERATION_ATTRIBUTE: &str = "data-render-generation";
#[cfg(target_arch = "wasm32")]
const IMAGE_RENDER_CHAPTER_ATTRIBUTE: &str = "data-render-chapter";
/// 読み込み失敗の `error` を受け取った画像に付与する印。
#[cfg(target_arch = "wasm32")]
const IMAGE_ERROR_ATTRIBUTE: &str = "data-image-error";

/// error イベントの対象が現在の描画で生成された Blob URL 画像かどうか。
///
/// 章切替や Blob URL の revoke で古い画像の error が届いても、生成時の
/// generation / chapter が現在値と一致しなければ記録しない。
#[cfg(target_arch = "wasm32")]
fn is_current_render_image(
    image: &web_sys::HtmlImageElement,
    generation: u32,
    chapter_index: usize,
) -> bool {
    let matches_generation = image
        .get_attribute(IMAGE_RENDER_GENERATION_ATTRIBUTE)
        .and_then(|value| value.parse::<u32>().ok())
        == Some(generation);
    let matches_chapter = image
        .get_attribute(IMAGE_RENDER_CHAPTER_ATTRIBUTE)
        .and_then(|value| value.parse::<usize>().ok())
        == Some(chapter_index);
    matches_generation && matches_chapter
}

/// HTML 中の `<img>` のうち、`src` 属性を持つものだけを数える。
///
/// `resolve_images()` は解決できなかった画像から `src` を削除するため、
/// `<img` を単純に数えると実際より多く見積もり、位置復元が上限まで
/// 無駄に待つ。属性名の大文字小文字・`=` 前後の空白・クォートを許容して
/// タグ単位で走査する。属性値（クォート内）の文字は属性名として数えない。
#[cfg(any(target_arch = "wasm32", test))]
fn count_images_with_src(html: &str) -> usize {
    /// HTML の属性区切り空白（U+0009/000A/000C/000D/0020）。
    fn is_space(byte: u8) -> bool {
        matches!(byte, b'\t' | b'\n' | 0x0c | b'\r' | b' ')
    }

    let bytes = html.as_bytes();
    let mut count = 0_usize;
    let mut cursor = 0_usize;
    while cursor + 4 <= bytes.len() {
        if !bytes[cursor..cursor + 4].eq_ignore_ascii_case(b"<img") {
            cursor += 1;
            continue;
        }
        let name_end = cursor + 4;
        // `<image>` などの別タグを誤検出しない。
        if name_end >= bytes.len()
            || !(is_space(bytes[name_end]) || bytes[name_end] == b'/' || bytes[name_end] == b'>')
        {
            cursor = name_end;
            continue;
        }
        // タグ末尾を探す。クォート内の `>` は属性値の一部なので終端にしない。
        let mut end = name_end;
        let mut quote = None;
        while end < bytes.len() {
            let byte = bytes[end];
            if let Some(quoted) = quote {
                if byte == quoted {
                    quote = None;
                }
            } else if byte == b'"' || byte == b'\'' {
                quote = Some(byte);
            } else if byte == b'>' {
                break;
            }
            end += 1;
        }
        // 属性名 `src` を探す。直前が空白で、直後が区切りなら一致。
        // クォート内は属性値なので、そこに " src " があっても無視する。
        let mut position = name_end;
        let mut quote = None;
        let mut has_src = false;
        while position < end {
            let byte = bytes[position];
            if let Some(quoted) = quote {
                if byte == quoted {
                    quote = None;
                }
                position += 1;
                continue;
            }
            if byte == b'"' || byte == b'\'' {
                quote = Some(byte);
                position += 1;
                continue;
            }
            if is_space(byte)
                && position + 4 <= bytes.len()
                && bytes[position + 1..position + 4].eq_ignore_ascii_case(b"src")
            {
                let after = position + 4;
                if after >= end
                    || is_space(bytes[after])
                    || matches!(bytes[after], b'=' | b'/' | b'>')
                {
                    has_src = true;
                    break;
                }
            }
            position += 1;
        }
        if has_src {
            count += 1;
        }
        cursor = end.saturating_add(1).min(bytes.len());
    }
    count
}

#[cfg(test)]
mod image_count_tests {
    use super::count_images_with_src;

    #[test]
    fn ignores_src_text_inside_quoted_attributes() {
        // 回帰: alt/title の値に " src " があっても画像として数えない。
        assert_eq!(count_images_with_src(r#"<img alt="image src here">"#), 0);
        assert_eq!(count_images_with_src(r#"<img title='a src b'>"#), 0);
        assert_eq!(count_images_with_src(r#"<img alt="x src=y">"#), 0);
        // クォート内の ">" でタグが終わったと誤判定しないことも確認する。
        assert_eq!(count_images_with_src(r#"<img alt="a > b" src="x">"#), 1);
    }

    #[test]
    fn counts_only_images_with_a_src_attribute() {
        assert_eq!(count_images_with_src(r#"<img src="x">"#), 1);
        assert_eq!(count_images_with_src(r#"<IMG SRC = 'x'>"#), 1);
        assert_eq!(count_images_with_src(r#"<img alt="x">"#), 0);
        assert_eq!(count_images_with_src(r#"<img>"#), 0);
        assert_eq!(count_images_with_src(r#"<img data-src="x" srcdoc="y">"#), 0);
        // src なし画像が混在しても、src を持つものだけを数える。
        assert_eq!(
            count_images_with_src(r#"<img alt="image src here"><img src="x">"#),
            1
        );
    }
}

/// 本文の画像が DOM に現れ、すべて読み込みを終えたか判定する。
///
/// `expected` は設定した HTML のうち `src` を持つ `img` の数。本文の更新は
/// 非同期に DOM へ反映されるため、数が揃うまでは未確定として扱う。`src` が
/// 無い画像（解決できず除去したもの）は数えず、待たない。
///
/// 完了判定は `complete()` だけでは不十分。`complete()` は `src` 未設定・空・
/// broken・画像読み込み無効でも `true` を返すため、通常は自然サイズが未確定の
/// 画像を見逃さないよう `natural_width() > 0` も要求する（位置復元は自然サイズに
/// 依存）。ただし壊れた画像は `complete()` が `true` でも自然幅が 0 のままで、
/// 上限まで待たされるため、document の error リスナーが失敗を記録した画像だけ
/// 自然幅 0 を許容する（ADR-0225）。
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
        // 通常画像は自然幅が確定するまで待つ。error 済みの画像だけを豁免する。
        let errored = image.get_attribute(IMAGE_ERROR_ATTRIBUTE).as_deref() == Some("1");
        if !image.complete() || (image.natural_width() == 0 && !errored) {
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

/// 縮小後の Blob URL を `src` に設定し、実際に読み込まれた後に完了シグナルを付与する。
///
/// `load` リスナーを `src` の差し替えより前に登録することで、差し替えた画像の
/// 読み込み完了を取りこぼさない。E2E は `data-downscaled="1"` を待つ。
#[cfg(target_arch = "wasm32")]
fn set_downscaled_src(image: &web_sys::HtmlImageElement, url: &str) {
    let target = image.clone();
    let on_load = Closure::once_into_js(move || {
        let _ = target.set_attribute("data-downscaled", "1");
    });
    let _ = image.add_event_listener_with_callback("load", on_load.unchecked_ref());
    let _ = image.set_attribute("src", url);
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
    attempt: u32,
) {
    const DOWNSCALE_THRESHOLD: u32 = 4000;
    const MIN_MAX_DIMENSION: u32 = 2000;
    const NATURAL_SIZE_RETRY_LIMIT: u32 = 20;
    const NATURAL_SIZE_RETRY_MS: i32 = 100;
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
    // 自然サイズが未確定の巨大画像候補が一枚でもあれば、今回はスキップして
    // 呼び出し単位で高々一度だけ再スケジュールする。
    let mut needs_retry = false;

    for index in 0..images.length() {
        let Some(node) = images.get(index) else {
            continue;
        };
        let Ok(image) = node.dyn_into::<web_sys::HtmlImageElement>() else {
            continue;
        };
        // 縮小済みの画像は対象外（再試行時の二重縮小を防ぐ）。
        if image.get_attribute("data-downscaled").as_deref() == Some("1") {
            continue;
        }
        let width = image.natural_width();
        let height = image.natural_height();
        // 4000px 超の候補でも、自然サイズが未確定（読み込み中）なら縮小できない。
        // 永久スキップを避けるため、今回は見送って後で再スケジュールする。
        if width == 0 || height == 0 {
            needs_retry = true;
            continue;
        }
        if width <= DOWNSCALE_THRESHOLD && height <= DOWNSCALE_THRESHOLD {
            continue;
        }
        // 表示幅が自然幅に依存する画像は縮小しない（レイアウト変化を避ける）。
        if width <= content_width {
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
        // 元画像の MIME は resolve_images が付けた属性から得る。blob URL の
        // fetch はハングし得るため使わない。属性が無い/不正なら PNG を既定とし、
        // JPEG / WebP は属性経由で同じ形式のまま再エンコードする。
        let mime = image
            .get_attribute("data-source-mime")
            .filter(|value| matches!(value.as_str(), "image/jpeg" | "image/png" | "image/webp"))
            .unwrap_or_else(|| "image/png".to_string());
        let urls = std::rc::Rc::clone(&object_urls);
        let target = image.clone();
        let callback = Closure::once_into_js(move |blob: web_sys::Blob| {
            if let Ok(url) = web_sys::Url::create_object_url_with_blob(&blob) {
                set_downscaled_src(&target, &url);
                urls.borrow_mut().push(url);
                if !old_url.is_empty() {
                    let _ = web_sys::Url::revoke_object_url(&old_url);
                }
            }
        });
        let callback: js_sys::Function = callback.unchecked_into();
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
    }

    if needs_retry && attempt < NATURAL_SIZE_RETRY_LIMIT {
        let urls = std::rc::Rc::clone(&object_urls);
        let retry = Closure::once_into_js(move || {
            let urls = std::rc::Rc::clone(&urls);
            wasm_bindgen_futures::spawn_local(async move {
                downscale_large_images(urls, jpeg_quality, attempt + 1).await;
            });
        });
        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
            retry.unchecked_ref(),
            NATURAL_SIZE_RETRY_MS,
        );
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
                "大きな画像を検出しました（{width}x{height}）。警告のみで縮小は実行されません。より小さい画像を使用してください。"
            ));
        }
    }
    None
}

/// OS/ブラウザで「動きを減らす」設定が有効かどうか。
///
/// E2E が emulation を動的に切り替えるため、起動時にキャッシュせず
/// 位置を適用するたびに毎回読み直す。
#[cfg(target_arch = "wasm32")]
fn prefers_reduced_motion() -> bool {
    let Some(window) = web_sys::window() else {
        return false;
    };
    match window.match_media("(prefers-reduced-motion: reduce)") {
        Ok(Some(query)) => query.matches(),
        Ok(None) | Err(_) => false,
    }
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
        // Reduce では画像確定待ちの後に 1 回だけ適用し、この再試行ループを
        // 回さない（位置保存や後始末は従来どおり必ず通す）。
        let reduced_motion = prefers_reduced_motion();
        let fragment_not_applied = ctx.fragment.is_some() && actual == 0;
        if !reduced_motion && ctx.fragment.is_some() && fragment_not_applied && attempt < 12 {
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
        if !reduced_motion && ctx.fragment.is_none() && actual != wanted && attempt < 3 {
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
                downscale_large_images(urls, quality, 0).await;
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
///
/// 実装は core の [`sanitize_svg_wasm`] に一本化した（旧 DomParser 手動実装は
/// 削除）。fail-closed の契約も core 側が担う。
#[cfg(target_arch = "wasm32")]
fn sanitize_svg_bytes(bytes: &[u8]) -> Option<Vec<u8>> {
    sanitize_svg_wasm(bytes)
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

/// SVG ルート要素の `width` / `height` を自然寸法として読む。
///
/// 数値のみ許容し、0・上限超え・単位付き・viewBox のみの場合は `None`
/// （属性を付けない）。ADR-0136 の方針に従い、絶対サイズがブラウザ依存の
/// viewBox からは寸法を推測しない。
#[cfg(target_arch = "wasm32")]
fn svg_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    let text = String::from_utf8(bytes.to_vec()).ok()?;
    let parser = web_sys::DomParser::new().ok()?;
    let document = parser
        .parse_from_string(&text, web_sys::SupportedType::ImageSvgXml)
        .ok()?;
    let root = document.document_element()?;
    let width = parse_svg_dimension(&root.get_attribute("width")?)?;
    let height = parse_svg_dimension(&root.get_attribute("height")?)?;
    Some((width, height))
}

#[cfg(target_arch = "wasm32")]
fn parse_svg_dimension(value: &str) -> Option<u32> {
    let value = value.trim().parse::<u32>().ok()?;
    if value == 0 || value > openrustbooks_core::MAX_DIMENSION {
        return None;
    }
    Some(value)
}

/// 解決できた画像に自然寸法の `width` / `height` を付与する（ADR-0136）。
///
/// ラスター形式は core の pure 関数（`imagesize`）でヘッダーから取得する。
/// SVG は `imagesize` の対象外なのでルート要素の属性を読み、img 側の既存
/// 属性は上書きしない。取得できない場合は何も付けない（fail-closed）。
/// 付けた属性は縦横比の予約のみに使い、警告・縮小判定は naturalWidth に
/// 依存した既存ロジックのまま変更しない。
#[cfg(target_arch = "wasm32")]
fn apply_image_dimensions(image: &web_sys::HtmlImageElement, bytes: &[u8], mime: &str) {
    let dimensions = if mime == "image/svg+xml" {
        svg_dimensions(bytes)
    } else {
        openrustbooks_core::wasm::image_dimensions_wasm(bytes).and_then(|values| {
            match values.as_slice() {
                [width, height] => Some((*width, *height)),
                _ => None,
            }
        })
    };
    let Some((width, height)) = dimensions else {
        return;
    };
    let respect_existing = mime == "image/svg+xml";
    if !respect_existing || !image.has_attribute("width") {
        let _ = image.set_attribute("width", &width.to_string());
    }
    if !respect_existing || !image.has_attribute("height") {
        let _ = image.set_attribute("height", &height.to_string());
    }
}

/// `src` を削除した画像からは予約寸法も落とす（ゴーストボックス対策）。
#[cfg(target_arch = "wasm32")]
fn clear_image_dimensions(image: &web_sys::HtmlImageElement) {
    let _ = image.remove_attribute("width");
    let _ = image.remove_attribute("height");
}

/// サニタイズ済み HTML の画像を Blob URL に置き換える。
///
/// 解決できない画像は `src` を削除し、alt テキストだけを残す。Blob URL を設定
/// した画像には描画の世代・章を記録し、error リスナーが現在の描画だけに失敗の
/// 印を付けられるようにする（ADR-0225）。
#[cfg(target_arch = "wasm32")]
fn resolve_images(
    book: &WasmEpubBook,
    index: usize,
    generation: u32,
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
        let mime = mime_for_path(&src);
        match book.chapter_resource(index, src.clone()) {
            Ok(bytes) => match object_url_for(&bytes, &src) {
                Ok(url) => {
                    // デコードを非同期にして本文の描画をブロックしない。
                    // `loading="lazy"` は付けない（位置補正が `complete` を
                    // 待つため、画面外の画像が永久に未完了になる）。
                    let _ = image.set_attribute("decoding", "async");
                    // 縮小時の MIME 判定は blob URL の fetch に依存せず、この属性から得る
                    // （fetch はハングし得るため）。
                    let _ = image.set_attribute("data-source-mime", mime);
                    // error リスナーが現在の描画の画像だけを記録できるように、
                    // Blob URL を設定した img に世代と章を埋め込む。
                    let _ = image
                        .set_attribute(IMAGE_RENDER_GENERATION_ATTRIBUTE, &generation.to_string());
                    let _ = image.set_attribute(IMAGE_RENDER_CHAPTER_ATTRIBUTE, &index.to_string());
                    apply_image_dimensions(&image, &bytes, mime);
                    image.set_attribute("src", &url)?;
                    urls.push(url);
                }
                Err(_) => {
                    image.remove_attribute("src").ok();
                    clear_image_dimensions(&image);
                }
            },
            Err(_) => {
                image.remove_attribute("src").ok();
                clear_image_dimensions(&image);
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

/// 保存失敗を利用者へ通知する共通メッセージ（機微情報は含めない）。
const SAVE_ERROR_MESSAGE: &str =
    "保存に失敗しました。ブラウザの設定または空き容量を確認してください。";

/// 保存失敗通知の送信先（ADR-0164 のリスナー競合を解消する）。
///
/// 送信先ごとにイベント名を分ける。リスナーは Owner 破棄時に解除するが、
/// 同じイベントを両画面が購読すると status が競合するため送信先で分ける。
/// イベントへ載せるのは送信先を表すイベント名だけで、本文・値・キー名は
/// 含めない（ADR-0154 の redaction 方針を維持）。
#[derive(Clone, Copy)]
enum SaveErrorTarget {
    /// Reader の保存失敗通知は wasm ビルドだけが発行する（native の Reader は stub）。
    #[cfg(target_arch = "wasm32")]
    Reader,
    Editor,
}

impl SaveErrorTarget {
    /// 各画面のリスナーが購読するイベント名。
    fn event_name(self) -> &'static str {
        match self {
            #[cfg(target_arch = "wasm32")]
            Self::Reader => "openrustbooks:save-error:reader",
            Self::Editor => "openrustbooks:save-error:editor",
        }
    }
}

/// Reader の操作失敗を console.error へ操作名だけの固定文言で通知する。
///
/// 生エラー（JsValue / CoreError の Debug や to_string）はパス・本文・URL を
/// 含み得るため渡さない（ADR-0098 / 0103 / 0154）。
#[cfg(target_arch = "wasm32")]
fn report_reader_error(operation: &str) {
    web_sys::console::error_1(&JsValue::from_str(&format!("openrustbooks: {operation}")));
}

/// ファイル読込失敗の status 固定文言。
#[cfg(target_arch = "wasm32")]
const FILE_READ_FAILED_MESSAGE: &str = "ファイルの読み込みに失敗しました。もう一度お試しください。";
/// EPUB オープン失敗の status 固定文言。
#[cfg(target_arch = "wasm32")]
const EPUB_OPEN_FAILED_MESSAGE: &str =
    "EPUB を開けませんでした。形式が未対応かファイルが破損している可能性があります。";
/// 章一覧の読込失敗の status 固定文言。
#[cfg(target_arch = "wasm32")]
const CHAPTER_LIST_FAILED_MESSAGE: &str = "章一覧を読み込めませんでした。";
/// 章描画の失敗の status 固定文言。
#[cfg(target_arch = "wasm32")]
const CHAPTER_RENDER_FAILED_MESSAGE: &str = "章を表示できませんでした。";
/// リンク解決の失敗の status 固定文言。
#[cfg(target_arch = "wasm32")]
const LINK_RESOLUTION_FAILED_MESSAGE: &str = "リンクを解決できませんでした。";
/// 目次の解析失敗の status 固定文言（生エラー・href は出さない）。
#[cfg(target_arch = "wasm32")]
const TOC_PARSE_FAILED_MESSAGE: &str = "目次を読み込めませんでした。";
/// ランドマークの解析失敗の status 固定文言。
#[cfg(target_arch = "wasm32")]
const LANDMARK_PARSE_FAILED_MESSAGE: &str = "ランドマークを読み込めませんでした。";
/// ページ一覧の解析失敗の status 固定文言。
#[cfg(target_arch = "wasm32")]
const PAGE_LIST_PARSE_FAILED_MESSAGE: &str = "ページ一覧を読み込めませんでした。";
/// 現在値保護でバックアップの表示設定を復元しなかった場合の status 固定文言。
/// 設定値・キー・パスは載せない。
#[cfg(target_arch = "wasm32")]
const SETTINGS_PRESERVED_MESSAGE: &str =
    "表示設定は現在の値を保持しました。バックアップの表示設定は復元されていません。";

/// localStorage の保存失敗をコンソールと status に通知する。
///
/// wasm では tracing の出力が確認しづらいため `console.error` を使う。
/// save 関数は `set_status` を直接持たないので、window へのカスタムイベントで
/// 送信先の画面へ伝え、そこで status を更新する。通常の `Event` を使い、
/// 本文・値・キー名はイベントにも固定文言にも載せない。
fn report_save_failure(context: &str, target: SaveErrorTarget) {
    web_sys::console::error_1(&JsValue::from_str(&format!(
        "openrustbooks: {context} の保存に失敗しました"
    )));
    if let Some(window) = web_sys::window()
        && let Ok(event) = web_sys::Event::new(target.event_name())
    {
        let _ = window.dispatch_event(&event);
    }
}

// 容量超過で削除した件数・残件数・再保存結果を Reader のリスナーへ渡す一時領域。
//
// `dispatch_event` は同期なので、ペイロードは thread_local に置き、リスナーが
// 読み取った時点でクリアする。イベント自体は常に素の `Event` のままで、
// キー名・ファイル名・生エラーは一切載せない（数値と成否だけを渡す）。
// `Event` 型をこの領域へ保持しないこと（ADR-0164 の event 分離を崩さない）。
#[cfg(target_arch = "wasm32")]
thread_local! {
    static POSITION_PRUNE_NOTICE: std::cell::Cell<Option<(usize, usize, bool)>> =
        std::cell::Cell::new(None);
}

/// リスナーが削除通知のペイロードを読み取り、同時にクリアする。
#[cfg(target_arch = "wasm32")]
fn take_position_prune_notice() -> Option<(usize, usize, bool)> {
    POSITION_PRUNE_NOTICE.take()
}

/// 削除通知のペイロードから利用者向け status 文言を作る（数値のみ）。
#[cfg(target_arch = "wasm32")]
fn position_prune_status(deleted: usize, remaining: usize, saved: bool) -> String {
    if saved {
        format!(
            "空き容量を確保するため古い読書位置を {deleted} 件削除しました（残り {remaining} 件）。バックアップはエクスポートから保存できます。"
        )
    } else {
        format!(
            "空き容量不足で読書位置を {deleted} 件削除しましたが保存できませんでした（残り {remaining} 件）。"
        )
    }
}

/// 容量超過による古い位置の削除結果を console と Reader の status へ通知する。
///
/// `SaveErrorTarget::Reader` と同じカスタムイベントで status へ届けるが、
/// 文言は保存失敗の固定文言とは別で、数値（削除件数・残件数・成否）だけを
/// thread_local 経由で渡す。`dispatch_event` 後にペイロードを必ずクリアし、
/// リスナーが居ない場合も次回通知へ持ち越さない。
#[cfg(target_arch = "wasm32")]
fn report_position_prune(deleted: usize, remaining: usize, saved: bool) {
    if saved {
        web_sys::console::log_1(&JsValue::from_str(&format!(
            "openrustbooks: 読書位置を {deleted} 件削除しました（残り {remaining} 件）"
        )));
    } else {
        web_sys::console::error_1(&JsValue::from_str(&format!(
            "openrustbooks: 空き容量不足で読書位置を {deleted} 件削除しましたが保存できませんでした（残り {remaining} 件）"
        )));
    }
    POSITION_PRUNE_NOTICE.set(Some((deleted, remaining, saved)));
    if let Some(window) = web_sys::window()
        && let Ok(event) = web_sys::Event::new(SaveErrorTarget::Reader.event_name())
    {
        let _ = window.dispatch_event(&event);
    }
    // dispatch_event は同期。リスナーが take 済みでも必ずクリアする。
    POSITION_PRUNE_NOTICE.set(None);
}

/// インポート JSON のサイズ上限（8 MiB）。
///
/// core::io::DEFAULT_MAX_READ_BYTES（256 MiB）は EPUB などのバイナリを
/// ストリーミングで読むための上限であり、そのまま JSON に流用すると
/// `serde_json::Value` の展開で元テキストに加えて数百 MB のツリーを確保し、
/// ブラウザタブのヒープを枯渇させ得る。したがって core の上限からは導出せず、
/// バックアップ JSON 専用の実用値として固定する（R3-SE01 / ADR-0317）。
#[cfg(any(target_arch = "wasm32", test))]
const IMPORT_JSON_MAX_BYTES: f64 = 8.0 * 1024.0 * 1024.0;

/// 上限超過のインポートを拒否する際の固定文言。
/// パス・サイズ値・ファイル名・キー・値は一切出さない（R3-IV01）。
#[cfg(any(target_arch = "wasm32", test))]
const IMPORT_JSON_TOO_LARGE_MESSAGE: &str = "ファイルが大きすぎます。";

/// 構造制限の固定上限。正規のエクスポートはトップレベル 5 キー、
/// positions / settings は高々数千件、深さは 4 程度なので、これらは
/// 十分に余裕のある拒否閾値である。上限値は利用者向け文言へ出さない。
#[cfg(any(target_arch = "wasm32", test))]
const IMPORT_JSON_MAX_TOP_LEVEL_KEYS: usize = 64;
#[cfg(any(target_arch = "wasm32", test))]
const IMPORT_JSON_MAX_SECTION_KEYS: usize = 10_000;
#[cfg(any(target_arch = "wasm32", test))]
const IMPORT_JSON_MAX_SETTINGS_KEYS: usize = 64;
#[cfg(any(target_arch = "wasm32", test))]
const IMPORT_JSON_MAX_STRING_BYTES: usize = 2 * 1024 * 1024;
#[cfg(any(target_arch = "wasm32", test))]
const IMPORT_JSON_MAX_DEPTH: usize = 16;
/// ノード総数の上限。バイト上限内でも `[null,...]` のような入力で
/// Value のノード数が膨らむため、全体の確保量もここで抑える。
#[cfg(any(target_arch = "wasm32", test))]
const IMPORT_JSON_MAX_NODES: usize = 20_000;

/// `file.text()` の前に呼ぶ fail-closed なサイズ検査。
#[cfg(any(target_arch = "wasm32", test))]
fn import_json_exceeds_limit(size: f64) -> bool {
    size > IMPORT_JSON_MAX_BYTES
}

/// 構造走査中のコンテナ種別。トップレベルの positions / settings だけを
/// 個別のキー上限で検査し、それ以外は一律のセクション上限を適用する。
#[cfg(any(target_arch = "wasm32", test))]
#[derive(Clone, Copy)]
enum ImportJsonSection {
    Root,
    Positions,
    Settings,
    Other,
}

/// `serde_json::Value` を組み立てる前に構造を走査するビジター。
///
/// 各ノードの確保をカウントし、キー数・文字列長・深さ・ノード総数が
/// 固定上限を超えた時点で打ち切る。エラー文言は内部で捨てられ、
/// 利用者には `IMPORT_JSON_TOO_LARGE_MESSAGE` だけが返る。
#[cfg(any(target_arch = "wasm32", test))]
struct ImportJsonStructureVisitor<'a> {
    depth: usize,
    section: ImportJsonSection,
    budget: &'a std::cell::Cell<usize>,
}

#[cfg(any(target_arch = "wasm32", test))]
impl<'a> ImportJsonStructureVisitor<'a> {
    /// 1 ノード分の予算を消費する。
    fn charge(&self) -> Result<(), String> {
        let used = self.budget.get();
        if used >= IMPORT_JSON_MAX_NODES {
            return Err("node budget exceeded".to_string());
        }
        self.budget.set(used + 1);
        Ok(())
    }

    /// キー・値の文字列長を検査する。
    fn check_string(value: &str) -> Result<(), String> {
        if value.len() > IMPORT_JSON_MAX_STRING_BYTES {
            return Err("string length exceeded".to_string());
        }
        Ok(())
    }
}

#[cfg(any(target_arch = "wasm32", test))]
impl<'de, 'a> serde::de::Visitor<'de> for ImportJsonStructureVisitor<'a> {
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a JSON value within import limits")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        if self.depth > IMPORT_JSON_MAX_DEPTH {
            return Err(serde::de::Error::custom("depth exceeded"));
        }
        self.charge().map_err(serde::de::Error::custom)?;
        let key_limit = match self.section {
            ImportJsonSection::Root => IMPORT_JSON_MAX_TOP_LEVEL_KEYS,
            ImportJsonSection::Settings => IMPORT_JSON_MAX_SETTINGS_KEYS,
            ImportJsonSection::Positions | ImportJsonSection::Other => IMPORT_JSON_MAX_SECTION_KEYS,
        };
        let mut count = 0_usize;
        while let Some(key) = map.next_key::<String>()? {
            count += 1;
            if count > key_limit {
                return Err(serde::de::Error::custom("key count exceeded"));
            }
            Self::check_string(&key).map_err(serde::de::Error::custom)?;
            let child_section = match self.section {
                ImportJsonSection::Root if key == "positions" => ImportJsonSection::Positions,
                ImportJsonSection::Root if key == "settings" => ImportJsonSection::Settings,
                _ => ImportJsonSection::Other,
            };
            map.next_value_seed(ImportJsonStructureSeed {
                depth: self.depth + 1,
                section: child_section,
                budget: self.budget,
            })?;
        }
        Ok(())
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        if self.depth > IMPORT_JSON_MAX_DEPTH {
            return Err(serde::de::Error::custom("depth exceeded"));
        }
        self.charge().map_err(serde::de::Error::custom)?;
        let mut count = 0_usize;
        while seq
            .next_element_seed(ImportJsonStructureSeed {
                depth: self.depth + 1,
                section: ImportJsonSection::Other,
                budget: self.budget,
            })?
            .is_some()
        {
            count += 1;
            if count > IMPORT_JSON_MAX_SECTION_KEYS {
                return Err(serde::de::Error::custom("element count exceeded"));
            }
        }
        Ok(())
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.charge().map_err(E::custom)?;
        Self::check_string(value).map_err(E::custom)
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_str(&value)
    }

    fn visit_u64<E>(self, _value: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.charge().map_err(E::custom)
    }

    fn visit_i64<E>(self, _value: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.charge().map_err(E::custom)
    }

    fn visit_f64<E>(self, _value: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.charge().map_err(E::custom)
    }

    fn visit_bool<E>(self, _value: bool) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.charge().map_err(E::custom)
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.charge().map_err(E::custom)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.charge().map_err(E::custom)
    }
}

/// 再帰的な構造走査のためのシード。
#[cfg(any(target_arch = "wasm32", test))]
struct ImportJsonStructureSeed<'a> {
    depth: usize,
    section: ImportJsonSection,
    budget: &'a std::cell::Cell<usize>,
}

#[cfg(any(target_arch = "wasm32", test))]
impl<'de, 'a> DeserializeSeed<'de> for ImportJsonStructureSeed<'a> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        deserializer.deserialize_any(ImportJsonStructureVisitor {
            depth: self.depth,
            section: self.section,
            budget: self.budget,
        })
    }
}

/// 構造走査の失敗種別。構文エラーは既存の「JSON の解析に失敗しました」へ、
/// 上限超過は `IMPORT_JSON_TOO_LARGE_MESSAGE` へ変換するため区別する。
#[cfg(any(target_arch = "wasm32", test))]
#[derive(Debug, PartialEq, Eq)]
enum ImportJsonStructureError {
    Syntax,
    Limit,
}

/// `serde_json::Value` 化の前に、構造が固定上限内かどうかを判定する。
/// Value を一切組み立てないため、拒否時のヒープ確保を抑えられる。
/// 上限超過は `Limit`、JSON として不正な構文は `Syntax` を返す。
#[cfg(any(target_arch = "wasm32", test))]
fn import_json_structure_allowed(raw: &str) -> Result<(), ImportJsonStructureError> {
    let budget = std::cell::Cell::new(0_usize);
    let mut deserializer = serde_json::Deserializer::from_str(raw);
    let result = ImportJsonStructureSeed {
        depth: 1,
        section: ImportJsonSection::Root,
        budget: &budget,
    }
    .deserialize(&mut deserializer);
    match result {
        Ok(()) => Ok(()),
        // 上限超過は `serde::de::Error::custom` 由来なので serde_json では
        // Category::Data になる。それ以外は構文エラーとして扱い、既存の
        // 解析エラー文言を維持する（E2E の文言・check 名は変更しない）。
        Err(error) => match error.classify() {
            serde_json::error::Category::Data => Err(ImportJsonStructureError::Limit),
            serde_json::error::Category::Syntax
            | serde_json::error::Category::Eof
            | serde_json::error::Category::Io => Err(ImportJsonStructureError::Syntax),
        },
    }
}

/// 下書きの最大サイズ（UTF-8 バイト数）。これを超える本文は保存しない。
const EDITOR_DRAFT_MAX_BYTES: usize = 1_048_576;

/// 下書きが上限を超えたときの固定文言。本文・サイズ値は載せない。
const EDITOR_DRAFT_TOO_LARGE_MESSAGE: &str = "下書きが大きすぎて保存できません";

/// 上限超の下書きを復元したときの固定文言。本文・サイズ値は載せない。
const EDITOR_DRAFT_TOO_LARGE_RESTORED_MESSAGE: &str =
    "保存済みの下書きが大きすぎます。編集して保存してください。";

/// 入力から自動保存までの固定遅延（ms）。Web 側から設定が渡されていないため
/// 固定値を使う（設定連携は将来の契約変更として ADR に記録する）。
/// 連続入力で status の aria-live 通知が溢れないようデバウンスする。
const EDITOR_AUTOSAVE_DELAY_MS: i32 = 800;

/// 入力直後に未保存状態を伝える文言。
const EDITOR_UNSAVED_MESSAGE: &str = "未保存の変更があります";

/// 自動保存に成功したことを伝える文言。
const EDITOR_AUTOSAVED_MESSAGE: &str = "下書きを自動保存しました";

/// エディタの下書きを保存する localStorage キーの接頭辞。
const EDITOR_DRAFT_KEY_PREFIX: &str = "openrustbooks:editor:";

/// ドキュメント ID から下書きキーを作る。
fn editor_draft_key(doc_id: &str) -> String {
    format!("{EDITOR_DRAFT_KEY_PREFIX}{doc_id}")
}

/// 下書き本文を読み込む。localStorage が使えない・キーが無い場合は `None`。
fn load_editor_draft(doc_id: &str) -> Option<String> {
    let storage = web_sys::window()?.local_storage().ok()??;
    storage.get_item(&editor_draft_key(doc_id)).ok()?
}

/// 下書き本文を保存する。本文は status / console に出さず、失敗時は
/// ADR-0154 の共通通知（カスタムイベント + console.error）だけを発行する。
fn save_editor_draft(doc_id: &str, content: &str) -> bool {
    // 上限超は set_item を一切呼ばずに失敗通知だけ出す（本文・サイズは渡さない）。
    if content.len() > EDITOR_DRAFT_MAX_BYTES {
        report_save_failure("下書き", SaveErrorTarget::Editor);
        return false;
    }
    let Some(window) = web_sys::window() else {
        report_save_failure("下書き", SaveErrorTarget::Editor);
        return false;
    };
    let Ok(Some(storage)) = window.local_storage() else {
        report_save_failure("下書き", SaveErrorTarget::Editor);
        return false;
    };
    if storage.set_item(&editor_draft_key(doc_id), content).is_ok() {
        true
    } else {
        report_save_failure("下書き", SaveErrorTarget::Editor);
        false
    }
}

/// 読書位置を保存する。localStorage が使えない場合は通知して諦める。
///
/// 保存のたびに古いエントリを掃除する。容量超過で失敗した場合は件数を
/// 減らして再試行し、削除件数・残件数・再保存結果を通知する。通常の
/// 掃除（90 日 / 40 件）の挙動は変更しない。
#[cfg(target_arch = "wasm32")]
fn save_position(key: &str, position: &SavedPosition) {
    let Some(window) = web_sys::window() else {
        report_save_failure("読書位置", SaveErrorTarget::Reader);
        return;
    };
    let Ok(Some(storage)) = window.local_storage() else {
        report_save_failure("読書位置", SaveErrorTarget::Reader);
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

    // 容量超過。古い順に減らして再試行する。remove_item が成功した件数だけを
    // 数え、再試行の成否と残件数を利用者へ通知する（キー名・生エラーは渡さない）。
    let mut deleted = 0usize;
    let mut saved = false;
    for keep in [POSITION_MIN_KEEP, 1] {
        let mut entries: Vec<(String, f64)> = list_positions(&storage)
            .into_iter()
            .filter(|(entry_key, _)| entry_key != key)
            .collect();
        sort_newest_first(&mut entries);
        for (old_key, _) in entries.into_iter().skip(keep) {
            if storage.remove_item(&old_key).is_ok() {
                deleted += 1;
            }
        }
        if storage.set_item(key, &raw).is_ok() {
            saved = true;
            break;
        }
    }
    let remaining = list_positions(&storage).len();
    report_position_prune(deleted, remaining, saved);
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

/// 表示設定を保存する。成否を返し、全ての失敗経路で通知する（ADR-0165）。
#[cfg(target_arch = "wasm32")]
fn save_settings(settings: &ReaderSettings) -> bool {
    let Some(window) = web_sys::window() else {
        report_save_failure("表示設定", SaveErrorTarget::Reader);
        return false;
    };
    let Ok(Some(storage)) = window.local_storage() else {
        report_save_failure("表示設定", SaveErrorTarget::Reader);
        return false;
    };
    let Ok(raw) = serde_json::to_string(settings) else {
        report_save_failure("表示設定", SaveErrorTarget::Reader);
        return false;
    };
    if storage.set_item(SETTINGS_KEY, &raw).is_err() {
        report_save_failure("表示設定", SaveErrorTarget::Reader);
        return false;
    }
    true
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

/// 保存した読書位置をすべて削除し、(削除件数, 1 件でも失敗したか) を返す。
#[cfg(target_arch = "wasm32")]
fn clear_reading_positions() -> (usize, bool) {
    let Some(window) = web_sys::window() else {
        return (0, false);
    };
    let Ok(Some(storage)) = window.local_storage() else {
        return (0, false);
    };
    let Ok(length) = storage.length() else {
        return (0, false);
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
    let mut failed = false;
    for key in keys {
        if storage.remove_item(&key).is_ok() {
            removed += 1;
        } else {
            failed = true;
        }
    }
    if failed {
        // 失敗が複数件でも通知は 1 回だけ。キー名・値・件数・生エラーは載せない。
        report_save_failure("全消去", SaveErrorTarget::Reader);
    }
    (removed, failed)
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
    // エディタの下書きも含める。1 MiB 超の値は復元側で受け付けないため、
    // エクスポート側でも除外する（ADR-0167 の上限を維持）。
    let mut drafts = serde_json::Map::new();
    for index in 0..length {
        let Ok(Some(key)) = storage.key(index) else {
            continue;
        };
        if !key.starts_with(EDITOR_DRAFT_KEY_PREFIX) {
            continue;
        }
        if let Ok(Some(value)) = storage.get_item(&key)
            && value.len() <= EDITOR_DRAFT_MAX_BYTES
        {
            drafts.insert(key, serde_json::Value::String(value));
        }
    }
    let count = positions.len() + drafts.len();
    if count == 0 {
        // 保存済み位置・下書きが無い場合は空ファイルを出力しない
        // （呼び出し側が警告を表示する）。
        return 0;
    }

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
    root.insert("drafts".to_string(), serde_json::Value::Object(drafts));

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
///
/// 下書きは復元するが、上記の件数には含めない。
/// 位置／履歴／下書きの set_item が失敗した場合は `failed` に失敗件数を書き、
/// 呼び出し側が成功件数と失敗件数だけを固定文言で告知する（キー・値は出さない）。
/// バックアップに表示設定があるのに現在値保護で復元しなかった場合は
/// `settings_preserved` に true を書き、呼び出し側が固定文言で告知する。
/// 通知自体はこの関数が 1 回だけ発行する。
#[cfg(target_arch = "wasm32")]
fn import_positions(
    raw: &str,
    failed: &mut usize,
    settings_preserved: &mut bool,
) -> Result<(usize, usize), String> {
    // `Value` 展開の前に元テキスト長を再検査し、構造も固定上限で拒否する。
    // ハンドラ側の file.size() 検査に加えて、読み取り後の実長と構造を
    // ここでも確認することで TOCTOU とヒープ枯渇を防ぐ（R3-SE01）。
    // 拒否時は固定文言だけを返し、サイズ・キー・値・パスは出さない。
    if import_json_exceeds_limit(raw.len() as f64) {
        return Err(IMPORT_JSON_TOO_LARGE_MESSAGE.to_string());
    }
    // 構文エラーは既存の解析エラー文言へ、上限超過だけ固定文言へ変換する。
    // どちらもキー・値・サイズ・パスは出さない。
    match import_json_structure_allowed(raw) {
        Ok(()) => {}
        Err(ImportJsonStructureError::Syntax) => {
            return Err("JSON の解析に失敗しました".to_string());
        }
        Err(ImportJsonStructureError::Limit) => {
            return Err(IMPORT_JSON_TOO_LARGE_MESSAGE.to_string());
        }
    }
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
        return Err("未対応のエクスポート形式です。".to_string());
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
        } else {
            *failed += 1;
        }
    }

    // 表示設定の取り込み。
    // 現在の設定が localStorage に存在する場合は維持する。ADR-0032 では
    // 設定を無条件上書きとしていたが、ADR-0151 の監査で古いバックアップに
    // よる現在設定の破壊リスクが指摘されたため、存在する値は守る。
    // 現在の設定が無い初回起動・クリア後のみインポート値で復元する。
    // 設定の明示的な復元オプションは今回は UI を用意しないため未配線。
    let settings_obj = settings.clone();
    let current_settings_exist = storage.get_item(SETTINGS_KEY).ok().flatten().is_some();
    if let Some(serde_json::Value::Object(settings)) = settings_obj {
        if current_settings_exist {
            // 現在値を守るため復元しない。設定値は status に出さず、
            // 呼び出し側が固定文言で理由を告知する。
            *settings_preserved = true;
        } else {
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
    }

    // 移動履歴の取り込み。値が SavedHistory として読めるものだけ書き込む。
    // 現在同じキーの履歴が存在する場合は維持する（設定と同様に ADR-0151 の
    // 古いバックアップによる破壊リスクへの防御）。履歴の savedAt による
    // マージは今回行わず、現在値の維持で防御する。
    let mut imported_history = 0_usize;
    if let Some(serde_json::Value::Object(history)) = map.get("history") {
        for (key, value) in history {
            if !key.starts_with(HISTORY_KEY_PREFIX) {
                continue;
            }
            // 現在の履歴が存在する場合は上書きしない。
            if storage.get_item(key).ok().flatten().is_some() {
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
            } else {
                *failed += 1;
            }
        }
    }

    // エディタ下書きの取り込み。現在同じキーが存在する場合は維持し
    // （ADR-0157）、値が文字列で 1 MiB 以下のものだけ復元する（ADR-0167）。
    // 復元件数は位置・履歴の戻り値に含めず、set_item の失敗は failed
    // の件数にだけ反映する。下書きだけのバックアップを「データなし」と誤判定
    // しないよう、空判定用に成功件数だけ数える。
    let mut imported_drafts = 0_usize;
    if let Some(serde_json::Value::Object(drafts)) = map.get("drafts") {
        for (key, value) in drafts {
            if !key.starts_with(EDITOR_DRAFT_KEY_PREFIX) {
                continue;
            }
            let serde_json::Value::String(text) = value else {
                continue;
            };
            if text.len() > EDITOR_DRAFT_MAX_BYTES {
                continue;
            }
            if storage.get_item(key).ok().flatten().is_some() {
                continue;
            }
            if storage.set_item(key, text).is_ok() {
                imported_drafts += 1;
            } else {
                *failed += 1;
            }
        }
    }

    if *failed > 0 {
        // 位置・履歴・下書きの set_item 失敗は複数件でも 1 回だけ通知する。
        // キー名・値・生エラーは載せない（件数は呼び出し側の status 固定文言だけ）。
        report_save_failure("インポート", SaveErrorTarget::Reader);
    }

    if imported == 0
        && imported_history == 0
        && imported_drafts == 0
        && positions.is_empty()
        && settings.is_none()
    {
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

/// 履歴を保存する。localStorage が使えない場合は通知して諦める。
#[cfg(target_arch = "wasm32")]
fn save_history(key: &str, history: &[HistoryEntry], index: usize) {
    let Some(window) = web_sys::window() else {
        report_save_failure("履歴", SaveErrorTarget::Reader);
        return;
    };
    let Ok(Some(storage)) = window.local_storage() else {
        report_save_failure("履歴", SaveErrorTarget::Reader);
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
        if storage.set_item(key, &raw).is_err() {
            report_save_failure("履歴", SaveErrorTarget::Reader);
        }
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

    /// 読み込む EPUB のサイズ上限（core の `DEFAULT_MAX_READ_BYTES` から導出）。
    /// 表示用の MiB 値も同じ単一情報源から計算する（256 MiB は不変）。
    const MAX_EPUB_FILE_BYTES: f64 = openrustbooks_core::io::DEFAULT_MAX_READ_BYTES as f64;
    const MAX_EPUB_FILE_MIB: u64 = openrustbooks_core::io::DEFAULT_MAX_READ_BYTES / (1024 * 1024);
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

    #[derive(Deserialize)]
    struct CoverImage {
        href: String,
        bytes: Vec<u8>,
    }

    let book = Rc::new(RefCell::new(None::<WasmEpubBook>));
    // 非同期 EPUB ロードの世代。新しいファイル選択ごとにインクリメントし、
    // 完了が遅れた古いタスクが UI state を上書きするのを防ぐ（R6 PR-H03/TD-H03）。
    let load_generation = Rc::new(Cell::new(0_u64));
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
    let (status, set_status) = signal(String::from("EPUB ファイルを選択してください。"));
    // 巨大画像の自然サイズは画像の読み込み完了時に確定する。位置適用の
    // セトリング待ちには 2 秒の上限があるため、それを過ぎて読み込まれる
    // 画像を取りこぼさないよう load イベントでも警告を検出する。
    // load はバブルしないため document でキャプチャする。
    if let Some(window) = web_sys::window()
        && let Some(document) = window.document()
    {
        let on_image_load = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            if let Some(message) = find_large_image() {
                set_status.set(message);
            }
        }) as Box<dyn FnMut(web_sys::Event)>);
        let _ = document.add_event_listener_with_callback_and_bool(
            "load",
            on_image_load.as_ref().unchecked_ref(),
            true,
        );
        // 再マウント時に古い document の load リスナーが残らないよう cleanup で解除する。
        // on_cleanup は Send + Sync を要求するため、!Send な Document/Closure を
        // SendWrapper で包んだ Option で保持し、remove の後に drop する。
        let listener = SendWrapper::new(Some((document, on_image_load)));
        on_cleanup(move || {
            if let Some((document, callback)) = listener.take() {
                let _ = document.remove_event_listener_with_callback_and_bool(
                    "load",
                    callback.as_ref().unchecked_ref(),
                    true,
                );
            }
        });
    }
    // save 関数からの Reader 向け保存失敗通知だけを受け取り status に表示する。
    // Editor 向けイベントは受け取らず、status を競合させない（ADR-0164）。
    // 容量超過の削除通知は同じイベントで届くが、thread_local の数値ペイロードが
    // あれば専用文言、なければ既存の保存失敗文言を使う（ペイロードは読取りで消える）。
    if let Some(window) = web_sys::window() {
        let event_name = SaveErrorTarget::Reader.event_name();
        let on_save_error = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            let message = match take_position_prune_notice() {
                Some((deleted, remaining, saved)) => {
                    position_prune_status(deleted, remaining, saved)
                }
                None => SAVE_ERROR_MESSAGE.to_string(),
            };
            set_status.set(message);
        }) as Box<dyn FnMut(web_sys::Event)>);
        let _ = window
            .add_event_listener_with_callback(event_name, on_save_error.as_ref().unchecked_ref());
        // SPA 再マウントで古い signal への set が残らないよう、Owner 破棄時に解除する。
        // on_cleanup は Send + Sync を要求するため、!Send な Window/Closure を
        // SendWrapper で包んだ Option で保持し、remove の後に drop する。
        let listener = SendWrapper::new(Some((window, event_name, on_save_error)));
        on_cleanup(move || {
            if let Some((window, event_name, callback)) = listener.take() {
                let _ = window.remove_event_listener_with_callback(
                    event_name,
                    callback.as_ref().unchecked_ref(),
                );
            }
        });
    }
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
        // 再マウント時に古い MediaQueryList のリスナーが残らないよう cleanup で解除する。
        // on_cleanup は Send + Sync を要求するため、!Send な MediaQueryList/Closure を
        // SendWrapper で包んだ Option で保持し、remove の後に drop する。
        let listener = SendWrapper::new(Some((mq, callback)));
        on_cleanup(move || {
            if let Some((mq, callback)) = listener.take() {
                let _ = mq.remove_event_listener_with_callback(
                    "change",
                    callback.as_ref().unchecked_ref(),
                );
            }
        });
    }

    let scroll_save_timer = Rc::new(Cell::new(None::<i32>));
    // 位置適用中はスクロール保存のデバウンスを無効にする。
    let applying_position = Rc::new(Cell::new(false));
    // 位置適用の完了時刻。適用直後のスクロール保存を抑制する。
    let last_apply_time = Rc::new(Cell::new(0.0_f64));
    // 章描画の世代カウンタ。描画のたびに進めて古い適用を無効化する。
    let current_generation = Rc::new(Cell::new(0_u32));
    // 現在描画中の章。古い画像の error を無視するため世代と併せて保持する。
    let current_chapter = Rc::new(Cell::new(0_usize));
    // 壊れた画像は complete() が true のまま自然幅 0 になり、位置復元が
    // 上限（2 秒）まで待たされる。error はバブルしないため document で
    // キャプチャし、現在の描画の画像にだけ失敗の印を付ける（ADR-0225）。
    if let Some(window) = web_sys::window()
        && let Some(document) = window.document()
    {
        let error_generation = Rc::clone(&current_generation);
        let error_chapter = Rc::clone(&current_chapter);
        let on_image_error = Closure::wrap(Box::new(move |event: web_sys::Event| {
            let Some(target) = event.target() else {
                return;
            };
            let Ok(image) = target.dyn_into::<web_sys::HtmlImageElement>() else {
                return;
            };
            if is_current_render_image(&image, error_generation.get(), error_chapter.get()) {
                let _ = image.set_attribute(IMAGE_ERROR_ATTRIBUTE, "1");
            }
        }) as Box<dyn FnMut(web_sys::Event)>);
        let _ = document.add_event_listener_with_callback_and_bool(
            "error",
            on_image_error.as_ref().unchecked_ref(),
            true,
        );
        // 再マウント時に古い document のリスナーが残らないよう cleanup で解除する。
        let listener = SendWrapper::new(Some((document, on_image_error)));
        on_cleanup(move || {
            if let Some((document, callback)) = listener.take() {
                let _ = document.remove_event_listener_with_callback_and_bool(
                    "error",
                    callback.as_ref().unchecked_ref(),
                    true,
                );
            }
        });
    }
    let render_chapter: RenderChapter = {
        let book = Rc::clone(&book);
        let object_urls = Rc::clone(&object_urls);
        let position_key = Rc::clone(&position_key_state);
        let current_fragment = Rc::clone(&current_fragment);
        let scroll_save_timer = Rc::clone(&scroll_save_timer);
        let applying_position = Rc::clone(&applying_position);
        let last_apply_time = Rc::clone(&last_apply_time);
        let current_generation = Rc::clone(&current_generation);
        let current_chapter = Rc::clone(&current_chapter);
        Rc::new(move |index, fragment, scroll_top| {
            // 章の切り替えで古いスクロールのデバウンス保存をキャンセルする。
            if let Some(window) = web_sys::window()
                && let Some(handle) = scroll_save_timer.take()
            {
                window.clear_timeout_with_handle(handle);
            }
            applying_position.set(true);
            *current_fragment.borrow_mut() = fragment.clone();
            // 描画の開始時に世代と章を進め、古い画像の error / 位置適用を
            // 無効化する。resolve_images にも同じ世代を渡して属性に埋め込む。
            let generation = current_generation.get().wrapping_add(1);
            current_generation.set(generation);
            current_chapter.set(index);
            let result = book
                .borrow()
                .as_ref()
                .ok_or_else(|| String::from("開いている EPUB がありません。"))
                .and_then(|book| {
                    let html = book.chapter_html(index).map_err(|_| {
                        report_reader_error("Chapter rendering failed");
                        String::from(CHAPTER_RENDER_FAILED_MESSAGE)
                    })?;
                    let safe = sanitize_epub_html(&html);
                    resolve_images(book, index, generation, &safe).map_err(|_| {
                        report_reader_error("Image resolution failed");
                        String::from(CHAPTER_RENDER_FAILED_MESSAGE)
                    })
                });
            match result {
                Ok((html, urls)) => {
                    for url in object_urls.borrow_mut().drain(..) {
                        let _ = web_sys::Url::revoke_object_url(&url);
                    }
                    *object_urls.borrow_mut() = urls;
                    // `set_content` は html を move するため、先に読み込む画像数を数える。
                    // src を削除された画像は待たないため、src を持つ img だけを数える。
                    let expected_images = count_images_with_src(&html);
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
                // ここに来る文字列は上の map_err で固定文言化済み（生エラーは渡さない）。
                Err(message) => set_status.set(message),
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
        let load_generation = Rc::clone(&load_generation);
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
                set_status.set("ファイルを読み込めませんでした。".to_string());
                return;
            };
            let Some(file) = input.files().and_then(|files| files.item(0)) else {
                return;
            };
            if file.size() > MAX_EPUB_FILE_BYTES {
                set_status.set(format!(
                    "この EPUB が大きすぎます（上限: {MAX_EPUB_FILE_MIB} MiB）。"
                ));
                return;
            }
            // 有効なファイルが選ばれた時点で世代を進め、この選択に対応する
            // 非同期タスクだけを残す（不正な選択で進行中のロードを破棄しない）。
            // gen は Rust 2024 の予約語のため load_gen という名前を使う。
            load_generation.set(load_generation.get() + 1);
            let load_gen = load_generation.get();
            let key = position_key(&file);
            *position_key_state.borrow_mut() = Some(key.clone());
            // 別の本を開いたら履歴をリセットする。
            *history.borrow_mut() = Vec::new();
            history_index.set(0);
            sync_history();
            *history_key_state.borrow_mut() = Some(history_key(&file));
            set_status.set(format!("{} を開いています…", file.name()));
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
            let load_generation = Rc::clone(&load_generation);
            let cover_url_state = Rc::clone(&cover_url_state);
            let render_chapter = Rc::clone(&render_chapter);
            let history = Rc::clone(&history);
            let history_index = Rc::clone(&history_index);
            let sync_history = Rc::clone(&sync_history);
            spawn_local(async move {
                // 古い選択のタスクは、ここで即座に終了する。
                if load_gen != load_generation.get() {
                    return;
                }
                // array_buffer() の前に on_file_change で file.size() を
                // MAX_EPUB_FILE_BYTES（256 MiB）と比較済み。上限超は既に return
                // しているため、ここでは巨大ファイルを読み込まない。
                let buffer = match JsFuture::from(file.array_buffer()).await {
                    Ok(buffer) => buffer,
                    Err(_) => {
                        // await 中に新しい選択が入っていれば status を書かない。
                        report_reader_error("File read failed");
                        if load_gen == load_generation.get() {
                            set_status.set(FILE_READ_FAILED_MESSAGE.to_string());
                        }
                        return;
                    }
                };
                if load_gen != load_generation.get() {
                    return;
                }
                let bytes = Uint8Array::new(&buffer).to_vec();
                // 読み取り中にサイズが増えた場合も拒否する（TOCTOU 対策）。
                if bytes.len() as f64 > MAX_EPUB_FILE_BYTES {
                    if load_gen == load_generation.get() {
                        set_status.set(format!(
                            "この EPUB が大きすぎます（上限: {MAX_EPUB_FILE_MIB} MiB）。"
                        ));
                    }
                    return;
                }
                let opened = match WasmEpubBook::new(bytes) {
                    Ok(opened) => opened,
                    Err(_) => {
                        if load_gen != load_generation.get() {
                            return;
                        }
                        report_reader_error("EPUB open failed");
                        set_status.set(EPUB_OPEN_FAILED_MESSAGE.to_string());
                        return;
                    }
                };
                if load_gen != load_generation.get() {
                    return;
                }

                let count = opened.chapter_count();
                if count == 0 {
                    if load_gen != load_generation.get() {
                        return;
                    }
                    set_status.set("読み込み可能な章がありません。".to_string());
                    return;
                }
                let mut spine = Vec::with_capacity(count);
                for index in 0..count {
                    match opened.chapter_href(index) {
                        Ok(href) => spine.push((index, href)),
                        Err(_) => {
                            if load_gen != load_generation.get() {
                                return;
                            }
                            report_reader_error("Chapter list failed");
                            set_status.set(CHAPTER_LIST_FAILED_MESSAGE.to_string());
                            return;
                        }
                    }
                }

                // nav の解析失敗は「情報源なし（空配列）」と区別して固定文言を status へ出す。
                // 生エラー・href・タイトルは console / status のどちらにも載せない。
                let mut nav_warnings: Vec<&'static str> = Vec::new();
                let mut parse_nav = |result: Result<JsValue, JsValue>,
                                     operation: &'static str,
                                     message: &'static str|
                 -> Vec<TocEntry> {
                    match result.ok().and_then(|value| {
                        serde_wasm_bindgen::from_value::<Vec<TocEntry>>(value).ok()
                    }) {
                        Some(entries) => entries,
                        None => {
                            report_reader_error(operation);
                            nav_warnings.push(message);
                            Vec::new()
                        }
                    }
                };
                let toc = parse_nav(opened.toc(), "TOC parse failed", TOC_PARSE_FAILED_MESSAGE);
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

                let landmarks = parse_nav(
                    opened.landmarks(),
                    "Landmark parse failed",
                    LANDMARK_PARSE_FAILED_MESSAGE,
                );

                let page_list = parse_nav(
                    opened.page_list(),
                    "Page list parse failed",
                    PAGE_LIST_PARSE_FAILED_MESSAGE,
                );

                let language = opened.language();
                let has_remote_resources = opened.has_remote_resources();
                // ここから先は UI state を書くので、直前に世代を確認する。
                // （cover の object URL 生成もここで行うため、無駄な URL を作らない。）
                if load_gen != load_generation.get() {
                    return;
                }
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

                // 履歴・book などの UI state 適用直前に再確認する。
                if load_gen != load_generation.get() {
                    return;
                }
                *history.borrow_mut() = history_entries;
                history_index.set(history_idx);
                sync_history();

                *book.borrow_mut() = Some(opened);
                set_content_lang.set(language);
                set_remote_resources_warning.set(if has_remote_resources {
                    Some(String::from(
                        "この EPUB はリモートリソースを宣言しています。",
                    ))
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
                // render_chapter は object_urls を差し替えるため、古いタスクから呼ばせない。
                if load_gen != load_generation.get() {
                    return;
                }
                render_chapter(start_index, start_fragment, Some(start_scroll));
                // 章描画で status がクリアされるため、その後に警告を出す。
                // 既存のリモートリソース警告を優先し、nav 失敗は先頭の 1 件だけ告知する。
                if let Some(message) = remote_resources_warning.get() {
                    set_status.set(message);
                } else if let Some(message) = nav_warnings.first() {
                    set_status.set((*message).to_string());
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
                set_status.set(format!("ページ {} が見つかりません。", page));
                return;
            };
            let path = entry.href.split('#').next().unwrap_or(&entry.href);
            let target_index = chapter_hrefs.get().iter().position(|href| {
                href.rsplit('/').next().unwrap_or(href) == path.rsplit('/').next().unwrap_or(path)
            });
            if let Some(target_index) = target_index {
                navigate_to(target_index, entry.fragment);
            } else {
                set_status.set(format!("ページ {} が見つかりません。", page));
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
            if let Some(raw_fragment) = href.strip_prefix('#') {
                // 未復号の文字列を位置指定子として使わない。復号失敗は
                // リンクを開かせず fail-closed で報告する（core と同じ規則）。
                let Some(fragment) = decode_fragment(raw_fragment) else {
                    set_status.set("このリンクは開けません。".to_string());
                    return;
                };
                // 同じフラグメントへの再移動なら履歴を増やさない。
                if current_fragment.borrow().as_deref() != Some(fragment.as_str()) {
                    push_history_entry(current.get(), Some(fragment.clone()), 0);
                    *current_fragment.borrow_mut() = Some(fragment.clone());
                }
                scroll_to_fragment(&fragment);
                // スクロール後の実位置を新しい履歴エントリへ反映する。
                if let Some(entry) = history.borrow_mut().last_mut() {
                    entry.scroll_top = content_scroll_top();
                }
                if let Some(key) = position_key.borrow().as_ref() {
                    save_position(
                        key,
                        &SavedPosition {
                            chapter_index: current.get(),
                            fragment: Some(fragment),
                            scroll_top: content_scroll_top(),
                            saved_at: 0.0,
                        },
                    );
                }
                return;
            }
            if has_external_scheme(&href) {
                // スキームの検出規則は core の `has_scheme` と同じ。外部リンクとして
                // 開くのは http(s) / mailto だけに限り、file: / data: / javascript:
                // などは章リンクとして解決させず拒否する（fail-closed）。
                let scheme = href
                    .split_once(':')
                    .map(|(prefix, _)| prefix.to_ascii_lowercase())
                    .unwrap_or_default();
                if !matches!(scheme.as_str(), "http" | "https" | "mailto") {
                    set_status.set("このリンクは開けません。".to_string());
                    return;
                }
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
                Err(_) => {
                    report_reader_error("Link resolution failed");
                    set_status.set(LINK_RESOLUTION_FAILED_MESSAGE.to_string());
                    return;
                }
            };
            let Some(link) = link else {
                set_status.set("このリンクは開けません。".to_string());
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
        // 失敗時は report_save_failure のリスナーが status を更新済みなので
        // 成功文言で上書きしない（ADR-0165）。
        if save_settings(&ReaderSettings::default()) {
            set_status.set("表示設定をリセットしました。".to_string());
        }
    };
    let on_clear_data = move |_| {
        // confirm が使える環境ではキャンセルを尊重する。window が無い / confirm が
        // 例外を投げる環境では None となり、既存どおり削除する（fail-safe）。
        let confirmed = (|| {
            web_sys::window()?
                .confirm_with_message(
                    "保存済みの読書位置をすべて削除します。この操作は取り消せず、元に戻せません。続けますか？",
                )
                .ok()
        })();
        if confirmed == Some(false) {
            return;
        }
        let (removed, failed) = clear_reading_positions();
        if failed {
            // 保存失敗通知がリスナー経由で status に反映済みなので上書きしない。
            return;
        }
        set_status.set(format!("保存済みの読書位置を {removed} 件削除しました。"));
    };
    let on_export = move |_: web_sys::MouseEvent| {
        let exported = export_positions();
        if exported == 0 {
            set_status.set("エクスポートする読書位置がありません。".to_string());
        } else {
            set_status.set(format!("読書位置を {exported} 件エクスポートしました。"));
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
        // 選択後は常にクリアし、同じファイルを再選択できるようにする。
        input.set_value("");
        // 巨大 JSON を text() で読む前にサイズを検査する（fail-closed）。
        // 上限超は confirm を出さず、固定文言だけを status に出す。
        if import_json_exceeds_limit(file.size()) {
            set_status.set(IMPORT_JSON_TOO_LARGE_MESSAGE.to_string());
            return;
        }
        // confirm が使える環境ではキャンセルを尊重する。window が無い / confirm が
        // 例外を投げる環境では None となり、既存どおり取り込む（fail-safe）。
        let confirmed = (|| {
            web_sys::window()?
                .confirm_with_message(
                    "バックアップを取り込みます。現在の読書位置を上書きする場合があります。上書きしたデータは元に戻せません。続けますか？",
                )
                .ok()
        })();
        if confirmed == Some(false) {
            return;
        }
        let file_clone = file.clone();
        spawn_local(async move {
            let Ok(buffer) = JsFuture::from(file_clone.text()).await else {
                set_status.set("インポートファイルを読み込めませんでした。".to_string());
                return;
            };
            let Some(raw) = buffer.as_string() else {
                set_status.set("インポートファイルを読み込めませんでした。".to_string());
                return;
            };
            // 読み取り中にサイズが増えた場合も拒否する（TOCTOU 対策）。
            if import_json_exceeds_limit(raw.len() as f64) {
                set_status.set(IMPORT_JSON_TOO_LARGE_MESSAGE.to_string());
                return;
            }
            let mut failed = 0usize;
            let mut settings_preserved = false;
            let result = import_positions(&raw, &mut failed, &mut settings_preserved);
            match result {
                Ok((count, history_count)) => {
                    // 成功件数と失敗件数だけを固定文言で返す（キー・値・パスは出さない）。
                    // 既存 E2E が期待する成功文言は維持し、失敗がある場合のみ末尾に追加する。
                    let mut message = format!(
                        "読書位置を {count} 件、履歴を {history_count} 件インポートしました。"
                    );
                    if failed > 0 {
                        // 保存失敗通知がリスナー経由で出ていても、成功件数と失敗件数を
                        // 利用者へ残すため呼び出し側で固定文言を上書きする。
                        message.push_str(&format!(" {failed} 件失敗しました。"));
                    }
                    if settings_preserved {
                        // 現在値保護で設定を復元しなかった理由を固定文言で告知する。
                        message.push(' ');
                        message.push_str(SETTINGS_PRESERVED_MESSAGE);
                    }
                    set_status.set(message);
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
        move |event: web_sys::KeyboardEvent| {
            // モーダル表示中は背景のショートカットを止める（Esc は dialog に任せる）。
            if let Some(window) = web_sys::window()
                && let Some(document) = window.document()
                && let Ok(dialogs) = document.query_selector_all("dialog[open]")
                && dialogs.length() > 0
            {
                return;
            }
            // IME の変換中はキーを奪わない（確定時に誤操作しないため）。
            if event.is_composing() {
                return;
            }
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
                        let _ = html_input.focus();
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
    // SPA 再マウント時に、章描画／ファイル切替で生成した object URL を確実に
    // 解放する。on_cleanup は Send + Sync を要求するため、!Send な
    // Rc<RefCell<...>> を SendWrapper で包んでキャプチャする（ADR-0166）。
    // 章再描画・ファイル切替時の revoke は維持し、ここでは残余のみを扱う。
    let cleanup_object_urls = SendWrapper::new(Rc::clone(&object_urls));
    let cleanup_cover_url_state = SendWrapper::new(Rc::clone(&cover_url_state));
    let key_listener = window_event_listener(leptos::ev::keydown, key_handler);
    on_cleanup(move || {
        key_listener.remove();
        for url in cleanup_object_urls.borrow_mut().drain(..) {
            let _ = web_sys::Url::revoke_object_url(&url);
        }
        if let Some(url) = cleanup_cover_url_state.borrow_mut().take() {
            let _ = web_sys::Url::revoke_object_url(&url);
        }
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
                <label class="btn btn-primary" for="epub-file">"EPUB を開く"</label>
                <input id="epub-file" class="reader-file-input" type="file" accept=".epub,application/epub+zip" aria-label="EPUB ファイルを選択" aria-describedby="epub-file-help" on:change=on_file_change/>
                <span id="epub-file-help" class="visually-hidden">"Tab キーでこの入力へ移動し、Enter キーでファイル選択ダイアログを開きます。矢印キーでファイルを選べます。"</span>
                <button id="reader-font-smaller" class="btn" title="文字を小さくする" aria-label="Smaller text" aria-keyshortcuts="-" on:click=on_font_smaller disabled=move || font_scale.get() == 0>"A-"</button>
                <button id="reader-font-larger" class="btn" title="文字を大きくする" aria-label="Larger text" aria-keyshortcuts="+" on:click=on_font_larger disabled=move || font_larger_disabled.get()>"A+"</button>
                <button id="reader-line-spacing" class="btn" title="行間を切り替える" aria-label="行間を切り替える" on:click=on_line_spacing>"行間"</button>
                // aria-pressed はボタン操作対象のダーク表示が実際に有効か（effective_dark）を示す。
                // 可視ラベルも同じ実効値に揃え、OS 追従中に状態表示と告知が食い違わないようにする。
                // 追従の ON/OFF は隣の #reader-theme-auto が follow_system で担う。
                <button id="reader-theme" class="btn" title="ダーク表示を切り替える" aria-label="ダーク表示を切り替える" aria-pressed=move || if effective_dark.get() { "true" } else { "false" } aria-keyshortcuts="d" on:click=on_toggle_dark>{move || if effective_dark.get() { "ライト表示" } else { "ダーク表示" }}</button>
                <button id="reader-theme-auto" class="btn" title="OS の配色設定に追従する" aria-label="OS の配色に追従する" aria-pressed=move || if follow_system.get() { "true" } else { "false" } on:click=on_dark_auto>{move || if follow_system.get() { "自動:on" } else { "自動:off" }}</button>
                <button id="reader-downscale" class="btn" title="巨大画像を縮小する" aria-label="巨大画像を縮小する" aria-pressed=move || if downscale_images.get() { "true" } else { "false" } on:click=on_downscale_toggle>{move || if downscale_images.get() { "縮小:on" } else { "縮小:off" }}</button>
                <input id="reader-jpeg-quality" class="reader-chapter-select" type="number" min="50" max="100" aria-label="JPEG 品質" title="JPEG 品質 (%)" on:change=on_jpeg_quality_change style="max-width: 6rem;" prop:value=move || jpeg_quality.get().to_string()/>
                <button id="reader-clear-data" class="btn btn-danger" title="読書データを削除（元に戻せません）" aria-label="読書データを削除（元に戻せません）" on:click=on_clear_data>"読書データを削除"</button>
                <button id="reader-reset-settings" class="btn" title="表示設定をリセットする" aria-label="表示設定をリセットする" on:click=on_reset_settings>"表示設定をリセット"</button>
                <button id="reader-help" class="btn" title="キーボードショートカット (?)" aria-label="キーボードショートカット" aria-keyshortcuts="?" on:click=on_help_click>"?"</button>
                <button id="reader-export" class="btn" title="保存済みの読書位置を書き出す" aria-label="保存済みの読書位置を書き出す" on:click=on_export>"書き出し"</button>
                <button id="reader-import" class="btn" title="読書位置のバックアップを読み込む" aria-label="読書位置のバックアップを読み込む" on:click=on_import_click>"読み込み"</button>
                <input id="reader-import-file" class="reader-file-input" type="file" accept=".json,application/json" aria-label="読書位置バックアップファイルを選択" aria-describedby="reader-import-file-help" on:change=on_import_change/>
                <span id="reader-import-file-help" class="visually-hidden">"Tab キーでこの入力へ移動し、Enter キーでファイル選択ダイアログを開きます。既存の読書位置を上書きする場合があります。"</span>
                <button id="reader-back" class="btn" aria-label="前の履歴へ戻る" title="前の履歴へ戻る" aria-keyshortcuts="Alt+ArrowLeft" on:click=on_history_back disabled=move || !can_back.get()>"戻る"</button>
                <button id="reader-forward" class="btn" aria-label="次の履歴へ進む" title="次の履歴へ進む" aria-keyshortcuts="Alt+ArrowRight" on:click=on_history_forward disabled=move || !can_forward.get()>"進む"</button>
                <select id="reader-history" class="reader-chapter-select" aria-label="履歴" title="閲覧履歴" on:change=on_history_select prop:disabled=move || history_state.get().0 <= 1>
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
                <button class="btn" aria-label="前の章へ" title="前の章へ" aria-keyshortcuts="[" on:click=on_previous disabled=move || current.get() == 0>"◀"</button>
                <input id="reader-chapter-filter" class="reader-chapter-select" type="search" aria-label="章を絞り込む" placeholder="章を絞り込む" on:input=move |event: web_sys::Event| {
                    if let Some(input) = event.target().and_then(|target| target.dyn_into::<web_sys::HtmlInputElement>().ok()) {
                        set_chapter_filter.set(input.value());
                    }
                } style="max-width: 10rem;"/>
                <select id="reader-chapter-select" class="reader-chapter-select" aria-label="Chapter" title="章を選択" on:change=on_select prop:disabled=move || chapters.get().is_empty()>
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
                <select id="reader-landmarks" class="reader-chapter-select" aria-label="ランドマーク" title="ランドマーク" on:change=on_landmark_select prop:disabled=move || landmark_options.get().is_empty()>
                    <option value="" disabled selected>"ランドマークを選択"</option>
                    <For
                        each=move || landmark_options.get()
                        key=|(index, _)| *index
                        children=move |(index, label)| view! {
                            <option value=index.to_string()>{label.clone()}</option>
                        }
                    />
                </select>
                <button id="reader-history-delete" class="btn" aria-label="選択した履歴を削除" title="選択した履歴を削除" on:click=on_history_delete disabled=move || history_state.get().0 <= 1>"削除"</button>
                <button id="reader-history-clear" class="btn" aria-label="閲覧履歴を消去" title="現在の位置を残して履歴を消去" on:click=on_history_clear disabled=move || history_state.get().0 <= 1>"履歴を消去"</button>
                <input id="reader-page-input" class="reader-chapter-select" type="text" aria-label="ページ" placeholder="ページ" prop:disabled=move || page_list.get().is_empty() style="max-width: 6rem;"/>
                <input id="reader-swipe-threshold" class="reader-chapter-select" type="number" min="30" max="120" aria-label="横スワイプのしきい値" title="横スワイプのしきい値 (px)" on:change=on_swipe_threshold_change style="max-width: 6rem;" prop:value=move || swipe_threshold.get().to_string()/>
                <input id="reader-vertical-threshold" class="reader-chapter-select" type="number" min="10" max="80" aria-label="縦スクロールのしきい値" title="縦スクロールのしきい値 (px)" on:change=on_vertical_scroll_threshold_change style="max-width: 6rem;" prop:value=move || vertical_scroll_threshold.get().to_string()/>
                <button id="reader-page-go" class="btn" aria-label="ページへ移動" title="ページへ移動" on:click=on_page_go prop:disabled=move || page_list.get().is_empty()>"移動"</button>
                <button class="btn" aria-label="次の章へ" title="次の章へ" aria-keyshortcuts="]" on:click=on_next disabled=move || {
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
                <h2 id="reader-delete-title">{move || if delete_target.get() == "clear" { "すべての閲覧履歴を消去しますか？" } else { "この履歴を削除しますか？" }}</h2>
                <p>{move || if delete_target.get() == "clear" {
                    "現在の位置は残り、それ以外の履歴がすべて削除されます。この操作は元に戻せません。"
                } else {
                    "選択した履歴が削除されます。この操作は元に戻せません。"
                }}</p>
                <div class="dialog-actions">
                    // E2E が textContent === 'Cancel' で安全側ボタンを特定するため、可視文言は変更しない。
                    <button type="button" class="btn" autofocus title="キャンセル（削除を取りやめる）" aria-label="キャンセル（削除を取りやめる）" on:click=on_cancel_delete>"Cancel"</button>
                    <button type="button" class="btn btn-danger" id="reader-delete-confirm" title="削除を確定する（元に戻せません）" on:click=on_confirm_delete>"削除する"</button>
                </div>
            </dialog>
            // キーボードショートカットの一覧。
            <dialog id="reader-help-dialog" node_ref=help_dialog aria-labelledby="reader-help-title">
                <h2 id="reader-help-title">"キーボードショートカット"</h2>
                <ul class="shortcut-list">
                    // E2E が 'Previous' / 'dark mode' で主要ショートカットを検証するため残す。
                    <li><kbd>"["</kbd> " / " <kbd>"]"</kbd> " — Previous / next chapter（前の章 / 次の章へ移動）"</li>
                    <li><kbd>"+"</kbd> " / " <kbd>"-"</kbd> " — 文字を大きく / 小さくする"</li>
                    <li><kbd>"d"</kbd> " — Toggle dark mode（ダーク表示を切り替える）"</li>
                    <li><kbd>"Alt+←"</kbd> " / " <kbd>"Alt+→"</kbd> " — 前の履歴 / 次の履歴へ移動"</li>
                    // E2E が 'chapter filter' という文字列で検証するため英語表記を残す。
                    <li><kbd>"/"</kbd> " — 章フィルター（chapter filter）にフォーカス"</li>
                    <li><kbd>"?"</kbd> " — このヘルプを表示"</li>
                </ul>
                <div class="dialog-actions">
                    <button type="button" class="btn" autofocus on:click=move |_| {
                        if let Some(dialog) = help_dialog.get() { dialog.close(); }
                    }>"閉じる"</button>
                </div>
            </dialog>
            <div class="reader-body">
                <Show when=move || cover_url.get().is_some()>
                    <aside class="reader-sidebar">
                        <img class="reader-cover" src=move || cover_url.get().unwrap_or_default() alt="表紙"/>
                    </aside>
                </Show>
                <div class="reader-content" role="region" aria-labelledby="reader-chapter-title" tabindex="0" on:click=on_content_click on:scroll=on_scroll on:touchstart=on_touch_start on:touchend=on_touch_end>
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
        <div class="preview" inner_html=move || render_markdown_wasm(&content.get())/>
    }
}

/// `get_config` (ConfigView) を Web 側で受け取る DTO。
///
/// desktop の `ConfigView` / 保存時は `ConfigPatch` と同じ公開フィールドだけを持つ。
/// 秘密情報・パスフィールドは開示されない（API Key は `has_api_key` フラグのみ）。
#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct SettingsView {
    /// `ConfigView.revision`。保存時に `ConfigPatch.expected` へコピーする。
    revision: u64,
    general: SettingsGeneralView,
    ai: SettingsAiView,
    lint: SettingsLintView,
    storage: StorageConfigView,
    editor: SettingsEditorView,
    reader: SettingsReaderView,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct SettingsGeneralView {
    language: String,
    theme: String,
    check_updates: bool,
    log_level: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct SettingsOpenAiView {
    base_url: String,
    /// `true` when an API key is configured; the value itself is never sent.
    has_api_key: bool,
    model: String,
    organization: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct SettingsOllamaView {
    base_url: String,
    model: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct SettingsAiView {
    enabled: bool,
    default_provider: String,
    openai: SettingsOpenAiView,
    ollama: SettingsOllamaView,
    max_tokens_per_request: usize,
    prefer_local: bool,
    timeout_secs: u64,
    streaming: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct SettingsLintView {
    enabled: bool,
    enable_rules: Vec<String>,
    disable_rules: Vec<String>,
    auto_fix: bool,
    warnings_as_errors: bool,
}

/// desktop `StorageConfigView` と同形。path / recent_paths は持たない。
#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct StorageConfigView {
    local_roots: Vec<LocalRootView>,
    google_drive: Option<GoogleDriveConfigView>,
}

/// desktop `LocalRootView` と同形。`path` は持たない。
#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct LocalRootView {
    id: String,
    name: String,
    read_only: bool,
}

/// desktop `GoogleDriveConfigView` と同形。secret 値は持たない。
#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct GoogleDriveConfigView {
    client_id: String,
    has_client_secret: bool,
    has_refresh_token: bool,
    root_folder_id: Option<String>,
    use_app_data_folder: bool,
    scopes: Vec<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct SettingsEditorView {
    font_family: String,
    font_size: f32,
    line_height: f32,
    tab_size: usize,
    insert_spaces: bool,
    line_numbers: bool,
    word_wrap: bool,
    minimap: bool,
    vim_mode: bool,
    match_brackets: bool,
    auto_close_brackets: bool,
    indent_with_tabs: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct SettingsReaderView {
    font_family: String,
    font_size: f32,
    line_height: f32,
    theme: String,
    column_width: Option<u32>,
    scroll_mode: String,
    hyphenation: bool,
    vertical_writing: bool,
    show_page_numbers: bool,
    remember_position: bool,
}

/// Tauri IPC への薄いラッパ。
///
/// `window.__TAURI__` が無いブラウザ単体では invoke を呼ばず `Err("desktop only")` を返す。
/// 保存コマンドの接続は ADR 未決定のため、今は読み取り専用の下準備にとどめる。
async fn invoke(command: &str, args: &JsValue) -> Result<JsValue, String> {
    let window = web_sys::window().ok_or_else(|| "desktop only".to_string())?;
    let window: JsValue = window.into();
    let tauri = js_sys::Reflect::get(&window, &JsValue::from_str("__TAURI__"))
        .map_err(|_| "desktop only".to_string())?;
    if tauri.is_undefined() || tauri.is_null() {
        return Err("desktop only".to_string());
    }
    let core = js_sys::Reflect::get(&tauri, &JsValue::from_str("core"))
        .map_err(|_| "desktop only".to_string())?;
    if core.is_undefined() || core.is_null() {
        return Err("desktop only".to_string());
    }
    let invoke_fn = js_sys::Reflect::get(&core, &JsValue::from_str("invoke"))
        .map_err(|_| "desktop only".to_string())?;
    let invoke_fn: js_sys::Function = invoke_fn
        .dyn_into()
        .map_err(|_| "desktop only".to_string())?;
    let promise = invoke_fn
        .call2(&core, &JsValue::from_str(command), args)
        .map_err(|_| "desktop only".to_string())?;
    let promise: js_sys::Promise = promise.dyn_into().map_err(|_| "desktop only".to_string())?;
    wasm_bindgen_futures::JsFuture::from(promise)
        .await
        .map_err(|_| "デスクトップ版の設定を取得できませんでした".to_string())
}

/// 保存コマンド専用のエラー。Promise reject が Object なら `code` だけを取り、
/// 生 `message`（設定値・パス・秘密情報を含み得る）は保持しない（ADR-0269 / 0226）。
#[derive(Debug, Clone, PartialEq, Eq)]
struct CommandError {
    code: String,
}

impl CommandError {
    /// code ごとの固定日本語文言。生 message は status へ出さない。
    fn status_message(&self) -> &'static str {
        match self.code.as_str() {
            "CONFIG_CONFLICT" => {
                "設定が他の更新と競合しました。再読み込みしてからもう一度お試しください。"
            }
            "CONFIG_VALIDATION" => "設定値が正しくありません。入力内容を確認してください。",
            "CONFIG_STORAGE" => {
                "設定の保存に失敗しました。しばらくしてからもう一度お試しください。"
            }
            "AI_PROVIDER" => {
                "AI 機能の処理に失敗しました。しばらくしてからもう一度お試しください。"
            }
            // 未知 code は code 文字列自体を出さず、安全な既定文言へ落とす（現状維持）。
            _ => "設定を保存できませんでした。",
        }
    }
}

#[cfg(test)]
mod command_error_tests {
    use super::CommandError;

    fn error(code: &str) -> CommandError {
        CommandError {
            code: code.to_string(),
        }
    }

    #[test]
    fn status_message_uses_fixed_text_for_known_codes() {
        assert_eq!(
            error("CONFIG_CONFLICT").status_message(),
            "設定が他の更新と競合しました。再読み込みしてからもう一度お試しください。"
        );
        assert_eq!(
            error("CONFIG_VALIDATION").status_message(),
            "設定値が正しくありません。入力内容を確認してください。"
        );
        assert_eq!(
            error("CONFIG_STORAGE").status_message(),
            "設定の保存に失敗しました。しばらくしてからもう一度お試しください。"
        );
        assert_eq!(
            error("AI_PROVIDER").status_message(),
            "AI 機能の処理に失敗しました。しばらくしてからもう一度お試しください。"
        );
    }

    #[test]
    fn unknown_code_falls_back_to_safe_message_without_echoing_code() {
        let secret_code = "sk-leak-host-body";
        let message = error(secret_code).status_message();

        assert_eq!(message, "設定を保存できませんでした。");
        assert!(!message.contains(secret_code));
    }
}

#[cfg(test)]
mod settings_dto_contract_tests {
    use super::SettingsView;

    /// desktop `ConfigView::from_state` の JSON を模した最小 fixture。
    /// 全フィールドを埋め、未知フィールドは入れず、境界の往復を確認する。
    fn config_view_fixture() -> serde_json::Value {
        serde_json::json!({
            "revision": 7,
            "general": {
                "language": "ja",
                "theme": "dark",
                "check_updates": false,
                "log_level": "info"
            },
            "ai": {
                "enabled": false,
                "default_provider": "openai",
                "openai": {
                    "base_url": "https://api.openai.com/v1",
                    "has_api_key": false,
                    "model": "gpt-4o-mini",
                    "organization": null
                },
                "ollama": {
                    "base_url": "http://localhost:11434",
                    "model": "dummy-model"
                },
                "max_tokens_per_request": 4096,
                "prefer_local": false,
                "timeout_secs": 60,
                "streaming": false
            },
            "lint": {
                "enabled": true,
                "enable_rules": ["MD013"],
                "disable_rules": [],
                "auto_fix": false,
                "warnings_as_errors": false
            },
            "storage": {
                "local_roots": [{ "id": "root-1", "name": "library", "read_only": true }],
                "google_drive": null
            },
            "editor": {
                "font_family": "monospace",
                "font_size": 14.0,
                "line_height": 1.5,
                "tab_size": 4,
                "insert_spaces": true,
                "line_numbers": true,
                "word_wrap": false,
                "minimap": false,
                "vim_mode": false,
                "match_brackets": true,
                "auto_close_brackets": true,
                "indent_with_tabs": false
            },
            "reader": {
                "font_family": "serif",
                "font_size": 16.0,
                // f32 で厳密に表現できる値を使い、JSON 往復の等値比較を安定させる。
                "line_height": 1.75,
                "theme": "auto",
                "column_width": null,
                "scroll_mode": "vertical",
                "hyphenation": false,
                "vertical_writing": false,
                "show_page_numbers": true,
                "remember_position": true
            }
        })
    }

    #[test]
    fn settings_view_deserializes_every_config_view_field() {
        let fixture = config_view_fixture();
        let view: SettingsView = serde_json::from_value(fixture.clone()).unwrap();

        assert_eq!(view.revision, 7);
        assert_eq!(view.general.language, "ja");
        assert_eq!(view.general.log_level, "info");
        assert!(!view.ai.enabled);
        assert_eq!(view.ai.openai.model, "gpt-4o-mini");
        assert!(view.ai.openai.organization.is_none());
        assert_eq!(view.ai.ollama.base_url, "http://localhost:11434");
        assert_eq!(view.ai.max_tokens_per_request, 4096);
        assert_eq!(view.lint.enable_rules, vec!["MD013".to_string()]);
        assert_eq!(view.storage.local_roots.len(), 1);
        assert!(view.storage.google_drive.is_none());
        assert_eq!(view.editor.tab_size, 4);
        assert_eq!(view.reader.theme, "auto");
        assert!(view.reader.column_width.is_none());
        assert!(view.reader.remember_position);

        // 既知フィールドが 1 つも落ちず、未知フィールドも増えない往復。
        assert_eq!(serde_json::to_value(&view).unwrap(), fixture);
    }

    #[test]
    fn settings_view_rejects_missing_nested_field() {
        let mut fixture = config_view_fixture();
        fixture["general"]
            .as_object_mut()
            .unwrap()
            .remove("log_level");

        assert!(serde_json::from_value::<SettingsView>(fixture).is_err());
    }
}

#[cfg(test)]
mod import_size_tests {
    use super::{
        IMPORT_JSON_MAX_BYTES, IMPORT_JSON_MAX_DEPTH, IMPORT_JSON_MAX_NODES,
        IMPORT_JSON_MAX_SECTION_KEYS, IMPORT_JSON_MAX_SETTINGS_KEYS, IMPORT_JSON_MAX_STRING_BYTES,
        IMPORT_JSON_MAX_TOP_LEVEL_KEYS, IMPORT_JSON_TOO_LARGE_MESSAGE, ImportJsonStructureError,
        import_json_exceeds_limit, import_json_structure_allowed,
    };

    /// 指定した数の null 値キーを持つ JSON オブジェクトを作る。
    fn object_with_keys(count: usize) -> String {
        let entries = (0..count)
            .map(|index| format!("\"k{index}\":null"))
            .collect::<Vec<_>>()
            .join(",");
        format!("{{{entries}}}")
    }

    #[test]
    fn import_size_limit_is_practical_and_not_derived_from_core() {
        assert_eq!(IMPORT_JSON_MAX_BYTES, 8.0 * 1024.0 * 1024.0);
        const { assert!(IMPORT_JSON_MAX_BYTES < 256.0 * 1024.0 * 1024.0) };
    }

    #[test]
    fn import_size_limit_accepts_boundary_and_rejects_one_byte_over() {
        assert!(!import_json_exceeds_limit(IMPORT_JSON_MAX_BYTES));
        assert!(import_json_exceeds_limit(IMPORT_JSON_MAX_BYTES + 1.0));
        assert!(!import_json_exceeds_limit(0.0));

        // raw.len() ベースの再チェックも境界で一致することを確認する。
        let at_limit = "x".repeat(IMPORT_JSON_MAX_BYTES as usize);
        assert!(!import_json_exceeds_limit(at_limit.len() as f64));
        let mut over_limit = at_limit;
        over_limit.push('x');
        assert!(import_json_exceeds_limit(over_limit.len() as f64));
    }

    #[test]
    fn import_size_error_is_fixed_without_path_size_or_file_name() {
        let message = IMPORT_JSON_TOO_LARGE_MESSAGE;
        assert_eq!(message, "ファイルが大きすぎます。");
        assert!(!message.contains("256"));
        assert!(!message.contains("MiB"));
        assert!(!message.contains('/'));
        assert!(!message.contains("backup.json"));
        // 構造制限の上限値も固定文言へ露出させない。
        for limit in [
            IMPORT_JSON_MAX_BYTES.to_string(),
            IMPORT_JSON_MAX_TOP_LEVEL_KEYS.to_string(),
            IMPORT_JSON_MAX_SECTION_KEYS.to_string(),
            IMPORT_JSON_MAX_SETTINGS_KEYS.to_string(),
            IMPORT_JSON_MAX_STRING_BYTES.to_string(),
            IMPORT_JSON_MAX_DEPTH.to_string(),
            IMPORT_JSON_MAX_NODES.to_string(),
        ] {
            assert!(!message.contains(&limit), "上限値 {limit} が露出しています");
        }
    }

    #[test]
    fn normal_export_payload_is_accepted_by_structure_check() {
        // 通常のエクスポート JSON（version / positions / settings / history / drafts）は
        // 構造制限を通過し、その後の既存 import フローへ渡される。
        let raw = r#"{
            "version": 1,
            "positions": {
                "openrustbooks:position:ch1": "{\"chapterIndex\":1,\"fragment\":\"p2\",\"scrollTop\":42,\"savedAt\":1700000000000.0}"
            },
            "settings": {"fontScale": 125, "lineHeight": 180, "dark": true, "followSystem": false, "swipeThreshold": 50, "verticalScrollThreshold": 60, "downscaleImages": false, "jpegQuality": 80},
            "history": {
                "openrustbooks:history:doc": "{\"entries\":[{\"chapterIndex\":1,\"fragment\":null,\"scrollTop\":0,\"savedAt\":1700000000000.0}],\"index\":0}"
            },
            "drafts": {"openrustbooks:editor:doc": "下書き本文"}
        }"#;
        assert!(import_json_structure_allowed(raw).is_ok());
    }

    #[test]
    fn structure_check_rejects_excessive_top_level_keys() {
        assert!(
            import_json_structure_allowed(&object_with_keys(IMPORT_JSON_MAX_TOP_LEVEL_KEYS))
                .is_ok()
        );
        assert_eq!(
            import_json_structure_allowed(&object_with_keys(IMPORT_JSON_MAX_TOP_LEVEL_KEYS + 1)),
            Err(ImportJsonStructureError::Limit)
        );
    }

    #[test]
    fn structure_check_rejects_excessive_positions_and_settings_keys() {
        for (section, limit) in [
            ("positions", IMPORT_JSON_MAX_SECTION_KEYS),
            ("settings", IMPORT_JSON_MAX_SETTINGS_KEYS),
        ] {
            let within = format!("{{\"{section}\":{}}}", object_with_keys(limit));
            assert!(
                import_json_structure_allowed(&within).is_ok(),
                "section={section}"
            );
            let over = format!("{{\"{section}\":{}}}", object_with_keys(limit + 1));
            assert_eq!(
                import_json_structure_allowed(&over),
                Err(ImportJsonStructureError::Limit),
                "section={section}"
            );
        }
    }

    #[test]
    fn structure_check_rejects_excessive_nesting_depth() {
        let within = format!(
            "{}null{}",
            "[".repeat(IMPORT_JSON_MAX_DEPTH),
            "]".repeat(IMPORT_JSON_MAX_DEPTH)
        );
        assert!(import_json_structure_allowed(&within).is_ok());
        let over = format!(
            "{}null{}",
            "[".repeat(IMPORT_JSON_MAX_DEPTH + 1),
            "]".repeat(IMPORT_JSON_MAX_DEPTH + 1)
        );
        assert_eq!(
            import_json_structure_allowed(&over),
            Err(ImportJsonStructureError::Limit)
        );
    }

    #[test]
    fn structure_check_rejects_excessive_string_length() {
        // 値の文字列が上限超え。JSON 全体は 8 MiB 以下なので構造制限で拒否される。
        let long_value = "x".repeat(IMPORT_JSON_MAX_STRING_BYTES + 1);
        let with_long_value =
            format!("{{\"positions\":{{\"openrustbooks:position:ch1\":\"{long_value}\"}}}}");
        assert_eq!(
            import_json_structure_allowed(&with_long_value),
            Err(ImportJsonStructureError::Limit)
        );

        // キーの文字列も同じ上限で拒否する。
        let long_key = "x".repeat(IMPORT_JSON_MAX_STRING_BYTES + 1);
        let with_long_key = format!("{{\"{long_key}\":null}}");
        assert_eq!(
            import_json_structure_allowed(&with_long_key),
            Err(ImportJsonStructureError::Limit)
        );
    }

    #[test]
    fn structure_check_rejects_excessive_total_nodes() {
        // 各セクションはキー上限内でも、合計のノード数が上限を超えれば拒否する。
        let section = object_with_keys(IMPORT_JSON_MAX_SECTION_KEYS);
        let raw = format!("{{\"positions\":{section},\"history\":{section},\"drafts\":{section}}}");
        assert_eq!(
            import_json_structure_allowed(&raw),
            Err(ImportJsonStructureError::Limit)
        );
    }

    #[test]
    fn structure_check_distinguishes_syntax_errors_from_limit_violations() {
        // 構文エラーは上限超過ではなく Syntax として扱い、既存の
        // 「JSON の解析に失敗しました」文言へ変換される契約を守る。
        for malformed in ["{", "{\"version\":", "{\"version\":1,}", "not json"] {
            assert_eq!(
                import_json_structure_allowed(malformed),
                Err(ImportJsonStructureError::Syntax),
                "malformed={malformed}"
            );
        }
        let over = object_with_keys(IMPORT_JSON_MAX_TOP_LEVEL_KEYS + 1);
        assert_eq!(
            import_json_structure_allowed(&over),
            Err(ImportJsonStructureError::Limit)
        );
    }
}

#[cfg(test)]
mod epub_link_safety_tests {
    use super::{decode_fragment, has_external_scheme};

    #[test]
    fn scheme_detection_matches_core_rules() {
        for value in [
            "https://example.com",
            "http://example.com",
            "mailto:test@example.com",
            "file:///etc/passwd",
            "data:text/html,x",
            "javascript:alert(1)",
            "ftp://example.com/x",
            "a+b-c.d:scheme",
        ] {
            assert!(
                has_external_scheme(value),
                "scheme を検出できません: {value}"
            );
        }
        for value in [
            "ch2.xhtml",
            "ch2.xhtml#part",
            "1abc:not-a-scheme",
            ":no-scheme",
        ] {
            assert!(
                !has_external_scheme(value),
                "相対参照をスキーム付きと誤判定しました: {value}"
            );
        }
    }

    #[test]
    fn fragment_decoding_matches_core_rules() {
        assert_eq!(decode_fragment("part").as_deref(), Some("part"));
        assert_eq!(
            decode_fragment("%E7%AC%AC1%E7%AF%80").as_deref(),
            Some("第1節")
        );
        for value in ["%zz", "%2", "%C3%28"] {
            assert_eq!(
                decode_fragment(value),
                None,
                "復号失敗を検出できません: {value}"
            );
        }
    }

    /// core の公開ヘルパーと web 側の同等実装が同じ規則であることを検証する。
    #[test]
    fn rules_agree_with_core() {
        for value in [
            "https://example.com",
            "http://example.com",
            "mailto:test@example.com",
            "file:///etc/passwd",
            "data:text/html,x",
            "javascript:alert(1)",
            "ftp://example.com/x",
            "a+b-c.d:scheme",
            "ch2.xhtml",
            "ch2.xhtml#part",
            "1abc:not-a-scheme",
            ":no-scheme",
        ] {
            assert_eq!(
                has_external_scheme(value),
                openrustbooks_core::epub::has_scheme(value),
                "スキーム判定が core と一致しません: {value}"
            );
        }
        for value in ["part", "%E7%AC%AC1%E7%AF%80", "100%", "%zz", "%2", "%C3%28"] {
            assert_eq!(
                decode_fragment(value),
                openrustbooks_core::epub::percent_decode(value, "cross-check").ok(),
                "復号結果が core と一致しません: {value}"
            );
        }
    }
}

/// `update_config_patch` 呼び出し用。reflect 経路は `invoke` と同じで、
/// reject から `code` のみを取り出して `CommandError` に変換する。
async fn invoke_update(command: &str, args: &JsValue) -> Result<JsValue, CommandError> {
    let window = web_sys::window().ok_or_else(|| CommandError {
        code: "desktop only".to_string(),
    })?;
    let window: JsValue = window.into();
    let tauri = js_sys::Reflect::get(&window, &JsValue::from_str("__TAURI__")).map_err(|_| {
        CommandError {
            code: "desktop only".to_string(),
        }
    })?;
    if tauri.is_undefined() || tauri.is_null() {
        return Err(CommandError {
            code: "desktop only".to_string(),
        });
    }
    let core =
        js_sys::Reflect::get(&tauri, &JsValue::from_str("core")).map_err(|_| CommandError {
            code: "desktop only".to_string(),
        })?;
    if core.is_undefined() || core.is_null() {
        return Err(CommandError {
            code: "desktop only".to_string(),
        });
    }
    let invoke_fn =
        js_sys::Reflect::get(&core, &JsValue::from_str("invoke")).map_err(|_| CommandError {
            code: "desktop only".to_string(),
        })?;
    let invoke_fn: js_sys::Function = invoke_fn.dyn_into().map_err(|_| CommandError {
        code: "desktop only".to_string(),
    })?;
    let promise = invoke_fn
        .call2(&core, &JsValue::from_str(command), args)
        .map_err(|_| CommandError {
            code: "desktop only".to_string(),
        })?;
    let promise: js_sys::Promise = promise.dyn_into().map_err(|_| CommandError {
        code: "desktop only".to_string(),
    })?;
    wasm_bindgen_futures::JsFuture::from(promise)
        .await
        .map_err(|reason| CommandError {
            code: command_error_code(&reason),
        })
}

/// Tauri の command reject から固定 code を取り出す。
///
/// 実 WebView では `CommandError` が JSON 文字列として reject される場合があり、
/// その際は `Reflect::get(reason, "code")` が失敗する。
/// 文字列化された JSON も解釈して code を取り出し、既定は
/// `CONFIG_INTERNAL` とする（生 message は status へ出さない）。
fn command_error_code(reason: &JsValue) -> String {
    let direct = js_sys::Reflect::get(reason, &JsValue::from_str("code"))
        .ok()
        .and_then(|value| value.as_string());
    if let Some(code) = direct {
        return code;
    }
    let text = reason.as_string().or_else(|| {
        js_sys::JSON::stringify(reason)
            .ok()
            .and_then(|v| v.as_string())
    });
    if let Some(code) = text
        .as_deref()
        .and_then(|text| serde_json::from_str::<serde_json::Value>(text).ok())
        .and_then(|value| {
            value
                .get("code")
                .and_then(|v| v.as_str())
                .map(str::to_string)
        })
    {
        return code;
    }
    "CONFIG_INTERNAL".to_string()
}

/// `get_config` を呼び出して DTO に変換する（ブラウザでは Err）。
async fn fetch_config() -> Result<SettingsView, String> {
    let args: JsValue = js_sys::Object::new().into();
    let value = invoke("get_config", &args).await?;
    serde_wasm_bindgen::from_value(value).map_err(|_| "設定の形式が正しくありません".to_string())
}

/// 設定を保存する。`revision` を `expected` へコピーした `ConfigPatch` JSON を渡し、
/// 成功時は更新後の `ConfigView` を返す（ADR-0253 / 0269）。
async fn save_config(view: &SettingsView) -> Result<SettingsView, CommandError> {
    let mut patch = serde_json::to_value(view).map_err(|_| CommandError {
        code: "CONFIG_INTERNAL".to_string(),
    })?;
    if let Some(object) = patch.as_object_mut() {
        object.insert("expected".to_string(), serde_json::json!(view.revision));
    } else {
        return Err(CommandError {
            code: "CONFIG_INTERNAL".to_string(),
        });
    }
    // Tauri command の引数名は `patch`。実 WebView では `patch` を flat に渡すと
    // "invalid args `patch`" で reject される（R6 WebView dogfood）。
    let args_object = js_sys::Object::new();
    js_sys::Reflect::set(
        &args_object,
        &JsValue::from_str("patch"),
        &serde_wasm_bindgen::to_value(&patch).map_err(|_| CommandError {
            code: "CONFIG_INTERNAL".to_string(),
        })?,
    )
    .map_err(|_| CommandError {
        code: "CONFIG_INTERNAL".to_string(),
    })?;
    let args: JsValue = args_object.into();
    let value = invoke_update("update_config_patch", &args).await?;
    serde_wasm_bindgen::from_value(value).map_err(|_| CommandError {
        code: "CONFIG_INTERNAL".to_string(),
    })
}

fn internal_error() -> CommandError {
    CommandError {
        code: "CONFIG_INTERNAL".to_string(),
    }
}

/// API キー保存 command の呼び出し。秘密値は command 引数としてのみ渡し、
/// signal / status / ログへ保持しない（ADR-0315）。
async fn set_api_key(expected: u64, api_key: String) -> Result<SettingsView, CommandError> {
    let args = js_sys::Object::new();
    js_sys::Reflect::set(
        &args,
        &JsValue::from_str("expected"),
        &JsValue::from_f64(expected as f64),
    )
    .map_err(|_| internal_error())?;
    js_sys::Reflect::set(
        &args,
        &JsValue::from_str("api_key"),
        &JsValue::from_str(&api_key),
    )
    .map_err(|_| internal_error())?;
    let args: JsValue = args.into();
    let value = invoke_update("set_openai_api_key", &args).await?;
    serde_wasm_bindgen::from_value(value).map_err(|_| internal_error())
}

/// 「秘密を削除」command の呼び出し。api_key のみを対象にする（fail-closed）。
async fn clear_secrets() -> Result<SettingsView, CommandError> {
    let targets = js_sys::Object::new();
    for (name, value) in [
        ("api_key", true),
        ("client_secret", false),
        ("refresh_token", false),
    ] {
        js_sys::Reflect::set(
            &targets,
            &JsValue::from_str(name),
            &JsValue::from_bool(value),
        )
        .map_err(|_| internal_error())?;
    }
    let targets: JsValue = targets.into();
    let args = js_sys::Object::new();
    js_sys::Reflect::set(&args, &JsValue::from_str("targets"), &targets)
        .map_err(|_| internal_error())?;
    let args: JsValue = args.into();
    let value = invoke_update("clear_secrets", &args).await?;
    serde_wasm_bindgen::from_value(value).map_err(|_| internal_error())
}

/// 入力イベントから HTMLInputElement の値を取り出す。
fn input_value(event: &web_sys::Event) -> Option<String> {
    event
        .target()
        .and_then(|target| target.dyn_into::<web_sys::HtmlInputElement>().ok())
        .map(|input| input.value())
}

/// API キー input の DOM 参照。値は signal へ入れず、呼び出し時に直接読む。
fn api_key_input_element() -> Option<web_sys::HtmlInputElement> {
    web_sys::window()?
        .document()?
        .get_element_by_id("settings-api-key")?
        .dyn_into::<web_sys::HtmlInputElement>()
        .ok()
}

/// 成功・失敗にかかわらず、呼び出し後に DOM の入力値を必ずクリアする。
fn clear_api_key_input() {
    if let Some(input) = api_key_input_element() {
        input.set_value("");
    }
}

/// API キー保存の共通入口（保存ボタン / Enter の双方から呼ぶ）。
///
/// 入力値は DOM から読んで command へ渡すだけであり、signal や status へは
/// 入れない。呼び出し後は成否にかかわらず DOM 値をクリアする。
fn start_api_key_save(
    editable: RwSignal<bool>,
    saving: RwSignal<bool>,
    revision: RwSignal<u64>,
    config: RwSignal<Option<SettingsView>>,
    status: RwSignal<String>,
) {
    if !editable.get() || saving.get() {
        return;
    }
    let Some(input) = api_key_input_element() else {
        return;
    };
    let api_key = input.value();
    let expected = revision.get();
    saving.set(true);
    wasm_bindgen_futures::spawn_local(async move {
        let result = set_api_key(expected, api_key).await;
        clear_api_key_input();
        match result {
            Ok(updated) => {
                revision.set(updated.revision);
                config.update(|slot| {
                    if let Some(view) = slot {
                        view.ai.openai.has_api_key = updated.ai.openai.has_api_key;
                    }
                });
                status.set("API キーを保存しました。".to_string());
            }
            Err(error) => status.set(error.status_message().to_string()),
        }
        saving.set(false);
    });
}

/// API キー削除の共通入口。`clear_secrets` の api_key 対象のみを呼ぶ。
fn start_api_key_clear(
    editable: RwSignal<bool>,
    saving: RwSignal<bool>,
    revision: RwSignal<u64>,
    config: RwSignal<Option<SettingsView>>,
    status: RwSignal<String>,
) {
    if !editable.get() || saving.get() {
        return;
    }
    saving.set(true);
    wasm_bindgen_futures::spawn_local(async move {
        match clear_secrets().await {
            Ok(updated) => {
                revision.set(updated.revision);
                config.update(|slot| {
                    if let Some(view) = slot {
                        view.ai.openai.has_api_key = updated.ai.openai.has_api_key;
                    }
                });
                clear_api_key_input();
                status.set("API キーを削除しました。".to_string());
            }
            Err(error) => status.set(error.status_message().to_string()),
        }
        saving.set(false);
    });
}

/// 入力イベントから checkbox の checked を取り出す。
fn input_checked(event: &web_sys::Event) -> Option<bool> {
    event
        .target()
        .and_then(|target| target.dyn_into::<web_sys::HtmlInputElement>().ok())
        .map(|input| input.checked())
}

/// change イベントから select の value を取り出す。
fn select_value(event: &web_sys::Event) -> Option<String> {
    event
        .target()
        .and_then(|target| target.dyn_into::<web_sys::HtmlSelectElement>().ok())
        .map(|select| select.value())
}

/// Settings form component。
///
/// browser は fetch 失敗で readonly / disabled / 常時説明のまま（ADR-0193 / 0253）。
/// desktop は `get_config` の IPC 成功を権威に編集可能へ切り替え、
/// `ConfigPatch.expected` による CAS で保存する。
#[component]
fn SettingsForm() -> impl IntoView {
    let status = RwSignal::new(String::new());
    let config = RwSignal::new(Option::<SettingsView>::None);
    let editable = RwSignal::new(false);
    // 保存 in-flight 中の編集を防ぎ、成功レスポンスでローカル編集が
    // 上書きされる lost update を回避する（R6 レビュー / R2-BV01）。
    let saving = RwSignal::new(false);
    let revision = RwSignal::new(0u64);
    let max_tokens_error = RwSignal::new(String::new());
    let timeout_error = RwSignal::new(String::new());

    // Leptos 0.8 に on_mount は無いため、Effect::new（初回のみ実行）で代替する。
    Effect::new(move |_| {
        wasm_bindgen_futures::spawn_local(async move {
            match fetch_config().await {
                Ok(cfg) => {
                    revision.set(cfg.revision);
                    config.set(Some(cfg));
                    // __TAURI__ の存在ではなく IPC 成功を権威にする。
                    editable.set(true);
                }
                Err(_) => {
                    status.set("デスクトップ版のみ利用可能".to_string());
                }
            }
        });
    });

    let on_api_key_save = move |_event: web_sys::MouseEvent| {
        start_api_key_save(editable, saving, revision, config, status);
    };
    let on_api_key_keydown = move |event: web_sys::KeyboardEvent| {
        if event.key() == "Enter" {
            event.prevent_default();
            start_api_key_save(editable, saving, revision, config, status);
        }
    };
    let on_api_key_clear = move |_event: web_sys::MouseEvent| {
        start_api_key_clear(editable, saving, revision, config, status);
    };

    let on_save = move |_event: web_sys::MouseEvent| {
        // prop:disabled が反映される前の連続クリックもここで弾く。
        if saving.get() {
            return;
        }
        saving.set(true);
        let Some(view) = config.get() else {
            status.set("デスクトップ版のみ利用可能".to_string());
            saving.set(false);
            return;
        };
        wasm_bindgen_futures::spawn_local(async move {
            match save_config(&view).await {
                Ok(updated) => {
                    revision.set(updated.revision);
                    config.set(Some(updated));
                    status.set("設定を保存しました。".to_string());
                }
                Err(error) => status.set(error.status_message().to_string()),
            }
            // Rust に finally は無いため、成功・失敗の両方をここで解除する。
            saving.set(false);
        });
    };

    view! {
        <form class="settings-form">
            // ADR-0128 / 0193: ブラウザ版は読み取り専用。readonly のときだけ説明を表示し、
            // fetch_config の成否を伝える status（「デスクトップ版のみ利用可能」）は常時併記する。
            // R6 A05: desktop（editable=true）では browser-only 説明を DOM から除く。
            <Show when=move || !editable.get()>
                <p class="settings-notice">"ブラウザ版は表示のみです。設定の変更・保存はデスクトップ版で行ってください。"</p>
            </Show>
            <p class="settings-notice" role="status" aria-live="polite">{move || status.get()}</p>
            <p class="settings-revision">
                "Revision: "
                <span id="settings-revision">{move || revision.get().to_string()}</span>
            </p>
            <section>
                <h3>"一般設定"</h3>
                <label for="settings-language">"言語"</label>
                <input id="settings-language" type="text" readonly=move || !editable.get() || saving.get() prop:disabled=move || saving.get()
                    prop:value=move || config.get().map(|v| v.general.language).unwrap_or_default()
                    on:input=move |event| {
                        if saving.get() {
                            return;
                        }
                        if let Some(value) = input_value(&event) {
                            config.update(|slot| {
                                if let Some(view) = slot { view.general.language = value; }
                            });
                        }
                    }/>
                <label for="settings-theme">"テーマ"</label>
                <input id="settings-theme" type="text" readonly=move || !editable.get() || saving.get() prop:disabled=move || saving.get()
                    prop:value=move || config.get().map(|v| v.general.theme).unwrap_or_default()
                    on:input=move |event| {
                        if saving.get() {
                            return;
                        }
                        if let Some(value) = input_value(&event) {
                            config.update(|slot| {
                                if let Some(view) = slot { view.general.theme = value; }
                            });
                        }
                    }/>
                <label for="settings-log-level">"ログレベル"</label>
                <input id="settings-log-level" type="text" readonly=move || !editable.get() || saving.get() prop:disabled=move || saving.get()
                    prop:value=move || config.get().map(|v| v.general.log_level).unwrap_or_default()
                    on:input=move |event| {
                        if saving.get() {
                            return;
                        }
                        if let Some(value) = input_value(&event) {
                            config.update(|slot| {
                                if let Some(view) = slot { view.general.log_level = value; }
                            });
                        }
                    }/>
                // ADR-0304: telemetry は未配線の死設定。DTO から除外したため
                // 入力を置かず、固定文言で正直に表示する。
                <p class="settings-notice">"テレメトリ: 未配線（送信機能は実装されていません）"</p>
                // ADR-0222 / R6 A05: updater は未実装（plugin 初期化のみ）のため、
                // check_updates は editable トグルを置かず固定文言で正直に表示する。
                // 値は DTO 往復のため view に残すが、desktop の merge_secrets() が
                // 現在値を復元し JS 側の変更を無視する（ADR-0277）。
                <p class="settings-notice">"更新確認: 未配線（updater は実装されていません）"</p>
            </section>
            <section>
                <h3>"AI 設定"</h3>
                <label for="settings-ai-enabled">"AI 機能を有効にする"</label>
                <input id="settings-ai-enabled" type="checkbox" prop:disabled=move || !editable.get() || saving.get()
                    prop:checked=move || config.get().map(|v| v.ai.enabled).unwrap_or(false)
                    on:change=move |event| {
                        if saving.get() {
                            return;
                        }
                        if let Some(checked) = input_checked(&event) {
                            config.update(|slot| {
                                if let Some(view) = slot { view.ai.enabled = checked; }
                            });
                        }
                    }/>
                <label for="settings-provider">"プロバイダー"</label>
                <select id="settings-provider" prop:disabled=move || !editable.get() || saving.get()
                    prop:value=move || config.get().map(|v| v.ai.default_provider).unwrap_or_default()
                    on:change=move |event| {
                        if saving.get() {
                            return;
                        }
                        if let Some(value) = select_value(&event) {
                            config.update(|slot| {
                                if let Some(view) = slot { view.ai.default_provider = value; }
                            });
                        }
                    }>
                    <option value="openai">"OpenAI"</option>
                    <option value="ollama">"Ollama (Local)"</option>
                </select>
                <label for="settings-openai-base-url">"OpenAI ベース URL"</label>
                <input id="settings-openai-base-url" type="text" readonly=move || !editable.get() || saving.get() prop:disabled=move || saving.get()
                    prop:value=move || config.get().map(|v| v.ai.openai.base_url).unwrap_or_default()
                    on:input=move |event| {
                        if saving.get() {
                            return;
                        }
                        if let Some(value) = input_value(&event) {
                            config.update(|slot| {
                                if let Some(view) = slot { view.ai.openai.base_url = value; }
                            });
                        }
                    }/>
                <label for="settings-openai-model">"OpenAI モデル"</label>
                <input id="settings-openai-model" type="text" readonly=move || !editable.get() || saving.get() prop:disabled=move || saving.get()
                    prop:value=move || config.get().map(|v| v.ai.openai.model).unwrap_or_default()
                    on:input=move |event| {
                        if saving.get() {
                            return;
                        }
                        if let Some(value) = input_value(&event) {
                            config.update(|slot| {
                                if let Some(view) = slot { view.ai.openai.model = value; }
                            });
                        }
                    }/>
                // API キーの値は signal / DTO / status へ残さず、DOM から
                // 専用 command へ渡すだけにする。desktop のみ編集可能。
                <label for="settings-api-key">"API キー"</label>
                <input id="settings-api-key" type="password" placeholder="設定済みの場合は変更不要"
                    readonly=move || !editable.get() || saving.get()
                    prop:disabled=move || !editable.get() || saving.get()
                    on:keydown=on_api_key_keydown/>
                <button id="settings-api-key-save" type="button" class="btn btn-secondary"
                    prop:disabled=move || !editable.get() || saving.get()
                    on:click=on_api_key_save>"API キーを保存"</button>
                <button id="settings-api-key-clear" type="button" class="btn btn-secondary"
                    prop:disabled=move || !editable.get() || saving.get()
                    on:click=on_api_key_clear>"API キーを削除"</button>
                <span id="settings-api-key-state" class="api-key-state">
                    {move || if config.get().map(|v| v.ai.openai.has_api_key).unwrap_or(false) { "設定済み" } else { "未設定" }}
                </span>
                <label for="settings-organization">"組織"</label>
                <input id="settings-organization" type="text" readonly=move || !editable.get() || saving.get() prop:disabled=move || saving.get()
                    prop:value=move || config.get().and_then(|v| v.ai.openai.organization).unwrap_or_default()
                    on:input=move |event| {
                        if saving.get() {
                            return;
                        }
                        if let Some(value) = input_value(&event) {
                            config.update(|slot| {
                                if let Some(view) = slot {
                                    view.ai.openai.organization = if value.is_empty() { None } else { Some(value) };
                                }
                            });
                        }
                    }/>
                <label for="settings-ollama-base-url">"Ollama ベース URL"</label>
                <input id="settings-ollama-base-url" type="text" readonly=move || !editable.get() || saving.get() prop:disabled=move || saving.get()
                    prop:value=move || config.get().map(|v| v.ai.ollama.base_url).unwrap_or_default()
                    on:input=move |event| {
                        if saving.get() {
                            return;
                        }
                        if let Some(value) = input_value(&event) {
                            config.update(|slot| {
                                if let Some(view) = slot { view.ai.ollama.base_url = value; }
                            });
                        }
                    }/>
                <label for="settings-ollama-model">"Ollama モデル"</label>
                <input id="settings-ollama-model" type="text" readonly=move || !editable.get() || saving.get() prop:disabled=move || saving.get()
                    prop:value=move || config.get().map(|v| v.ai.ollama.model).unwrap_or_default()
                    on:input=move |event| {
                        if saving.get() {
                            return;
                        }
                        if let Some(value) = input_value(&event) {
                            config.update(|slot| {
                                if let Some(view) = slot { view.ai.ollama.model = value; }
                            });
                        }
                    }/>
                // ADR-0304: keep_alive_minutes は未配線の死設定。DTO から除外した。
                <p class="settings-notice">"Ollama Keep Alive: 未配線"</p>
                <label for="settings-max-tokens">"最大トークン/要求"</label>
                <input id="settings-max-tokens" type="number" min="1" readonly=move || !editable.get() || saving.get() prop:disabled=move || saving.get()
                    aria-invalid=move || (!max_tokens_error.get().is_empty()).to_string()
                    aria-describedby="settings-max-tokens-error"
                    prop:value=move || config.get().map(|v| v.ai.max_tokens_per_request.to_string()).unwrap_or_default()
                    on:input=move |event| {
                        if saving.get() {
                            return;
                        }
                        if let Some(value) = input_value(&event) {
                            match value.parse::<usize>() {
                                Ok(parsed) if parsed > 0 => {
                                    max_tokens_error.set(String::new());
                                    config.update(|slot| {
                                        if let Some(view) = slot { view.ai.max_tokens_per_request = parsed; }
                                    });
                                }
                                _ => max_tokens_error.set("最大トークン数は 1 以上の整数で入力してください。".to_string()),
                            }
                        }
                    }/>
                <p id="settings-max-tokens-error" class="settings-error" role="alert">{move || max_tokens_error.get()}</p>
                // ADR-0228 / 0304: 月間上限は未配線。config.toml の値は保持するが
                // DTO からは除外したため、固定文言で正直に表示する。
                <p class="settings-notice">"月間上限: 未配線"</p>
                <label for="settings-timeout">"タイムアウト (秒)"</label>
                <input id="settings-timeout" type="number" min="1" readonly=move || !editable.get() || saving.get() prop:disabled=move || saving.get()
                    aria-invalid=move || (!timeout_error.get().is_empty()).to_string()
                    aria-describedby="settings-timeout-error"
                    prop:value=move || config.get().map(|v| v.ai.timeout_secs.to_string()).unwrap_or_default()
                    on:input=move |event| {
                        if saving.get() {
                            return;
                        }
                        if let Some(value) = input_value(&event) {
                            match value.parse::<u64>() {
                                Ok(parsed) if parsed > 0 => {
                                    timeout_error.set(String::new());
                                    config.update(|slot| {
                                        if let Some(view) = slot { view.ai.timeout_secs = parsed; }
                                    });
                                }
                                _ => timeout_error.set("タイムアウトは 1 以上の整数で入力してください。".to_string()),
                            }
                        }
                    }/>
                <p id="settings-timeout-error" class="settings-error" role="alert">{move || timeout_error.get()}</p>
                <label for="settings-prefer-local">"ローカル優先"</label>
                <input id="settings-prefer-local" type="checkbox" prop:disabled=move || !editable.get() || saving.get()
                    prop:checked=move || config.get().map(|v| v.ai.prefer_local).unwrap_or(false)
                    on:change=move |event| {
                        if saving.get() {
                            return;
                        }
                        if let Some(checked) = input_checked(&event) {
                            config.update(|slot| {
                                if let Some(view) = slot { view.ai.prefer_local = checked; }
                            });
                        }
                    }/>
                <label for="settings-streaming">"ストリーミング"</label>
                <input id="settings-streaming" type="checkbox" prop:disabled=move || !editable.get() || saving.get()
                    prop:checked=move || config.get().map(|v| v.ai.streaming).unwrap_or(false)
                    on:change=move |event| {
                        if saving.get() {
                            return;
                        }
                        if let Some(checked) = input_checked(&event) {
                            config.update(|slot| {
                                if let Some(view) = slot { view.ai.streaming = checked; }
                            });
                        }
                    }/>
            </section>
            <section>
                <h3>"Lint 設定"</h3>
                <label for="settings-lint-enabled">"Lint を有効にする"</label>
                <input id="settings-lint-enabled" type="checkbox" prop:disabled=move || !editable.get() || saving.get()
                    prop:checked=move || config.get().map(|v| v.lint.enabled).unwrap_or(false)
                    on:change=move |event| {
                        if saving.get() {
                            return;
                        }
                        if let Some(checked) = input_checked(&event) {
                            config.update(|slot| {
                                if let Some(view) = slot { view.lint.enabled = checked; }
                            });
                        }
                    }/>
                <label for="settings-lint-enable-rules">"有効なルール (カンマ区切り)"</label>
                <input id="settings-lint-enable-rules" type="text" readonly=move || !editable.get() || saving.get() prop:disabled=move || saving.get()
                    prop:value=move || config.get().map(|v| v.lint.enable_rules.join(", ")).unwrap_or_default()
                    on:input=move |event| {
                        if saving.get() {
                            return;
                        }
                        if let Some(value) = input_value(&event) {
                            let rules = split_rules(&value);
                            config.update(|slot| {
                                if let Some(view) = slot { view.lint.enable_rules = rules; }
                            });
                        }
                    }/>
                <label for="settings-lint-disable-rules">"無効なルール (カンマ区切り)"</label>
                <input id="settings-lint-disable-rules" type="text" readonly=move || !editable.get() || saving.get() prop:disabled=move || saving.get()
                    prop:value=move || config.get().map(|v| v.lint.disable_rules.join(", ")).unwrap_or_default()
                    on:input=move |event| {
                        if saving.get() {
                            return;
                        }
                        if let Some(value) = input_value(&event) {
                            let rules = split_rules(&value);
                            config.update(|slot| {
                                if let Some(view) = slot { view.lint.disable_rules = rules; }
                            });
                        }
                    }/>
                <label for="settings-auto-fix">"保存時に自動修正"</label>
                <input id="settings-auto-fix" type="checkbox" prop:disabled=move || !editable.get() || saving.get()
                    prop:checked=move || config.get().map(|v| v.lint.auto_fix).unwrap_or(false)
                    on:change=move |event| {
                        if saving.get() {
                            return;
                        }
                        if let Some(checked) = input_checked(&event) {
                            config.update(|slot| {
                                if let Some(view) = slot { view.lint.auto_fix = checked; }
                            });
                        }
                    }/>
                <label for="settings-warnings-as-errors">"警告をエラーとして扱う"</label>
                <input id="settings-warnings-as-errors" type="checkbox" prop:disabled=move || !editable.get() || saving.get()
                    prop:checked=move || config.get().map(|v| v.lint.warnings_as_errors).unwrap_or(false)
                    on:change=move |event| {
                        if saving.get() {
                            return;
                        }
                        if let Some(checked) = input_checked(&event) {
                            config.update(|slot| {
                                if let Some(view) = slot { view.lint.warnings_as_errors = checked; }
                            });
                        }
                    }/>
            </section>
            <section>
                <h3>"エディター設定"</h3>
                <p class="settings-notice">"エディタ設定: config.toml の設定値です。エディタの動作には未接続です。"</p>
                <label for="settings-editor-font-family">"フォントファミリー"</label>
                <input id="settings-editor-font-family" type="text" readonly=move || !editable.get() || saving.get() prop:disabled=move || saving.get()
                    prop:value=move || config.get().map(|v| v.editor.font_family).unwrap_or_default()
                    on:input=move |event| {
                        if saving.get() {
                            return;
                        }
                        if let Some(value) = input_value(&event) {
                            config.update(|slot| {
                                if let Some(view) = slot { view.editor.font_family = value; }
                            });
                        }
                    }/>
                <label for="settings-editor-font-size">"フォントサイズ"</label>
                <input id="settings-editor-font-size" type="number" min="10" max="24" readonly=move || !editable.get() || saving.get() prop:disabled=move || saving.get()
                    prop:value=move || config.get().map(|v| v.editor.font_size.to_string()).unwrap_or_default()
                    on:input=move |event| {
                        if saving.get() {
                            return;
                        }
                        if let Some(parsed) = input_value(&event).and_then(|value| value.parse::<f32>().ok()) {

                            config.update(|slot| {

                                if let Some(view) = slot { view.editor.font_size = parsed; }

                            });

                        }
                    }/>
                <label for="settings-editor-line-height">"行の高さ"</label>
                <input id="settings-editor-line-height" type="number" step="0.1" readonly=move || !editable.get() || saving.get() prop:disabled=move || saving.get()
                    prop:value=move || config.get().map(|v| v.editor.line_height.to_string()).unwrap_or_default()
                    on:input=move |event| {
                        if saving.get() {
                            return;
                        }
                        if let Some(parsed) = input_value(&event).and_then(|value| value.parse::<f32>().ok()) {

                            config.update(|slot| {

                                if let Some(view) = slot { view.editor.line_height = parsed; }

                            });

                        }
                    }/>
                <label for="settings-editor-tab-size">"タブ幅"</label>
                <input id="settings-editor-tab-size" type="number" min="1" readonly=move || !editable.get() || saving.get() prop:disabled=move || saving.get()
                    prop:value=move || config.get().map(|v| v.editor.tab_size.to_string()).unwrap_or_default()
                    on:input=move |event| {
                        if saving.get() {
                            return;
                        }
                        if let Some(parsed) = input_value(&event).and_then(|value| value.parse::<usize>().ok()) {

                            config.update(|slot| {

                                if let Some(view) = slot { view.editor.tab_size = parsed; }

                            });

                        }
                    }/>
                // ADR-0179 / 0217 / 0304: auto_save_delay は未配線（実装は固定 800ms）。
                // DTO から除外したため固定文言で表示する。
                <p class="settings-notice">"自動保存間隔: 未配線（固定 800ms）"</p>
                <label for="settings-editor-line-numbers">"行番号を表示"</label>
                <input id="settings-editor-line-numbers" type="checkbox" prop:disabled=move || !editable.get() || saving.get()
                    prop:checked=move || config.get().map(|v| v.editor.line_numbers).unwrap_or(false)
                    on:change=move |event| {
                        if saving.get() {
                            return;
                        }
                        if let Some(checked) = input_checked(&event) {
                            config.update(|slot| {
                                if let Some(view) = slot { view.editor.line_numbers = checked; }
                            });
                        }
                    }/>
                <label for="settings-editor-word-wrap">"折り返し"</label>
                <input id="settings-editor-word-wrap" type="checkbox" prop:disabled=move || !editable.get() || saving.get()
                    prop:checked=move || config.get().map(|v| v.editor.word_wrap).unwrap_or(false)
                    on:change=move |event| {
                        if saving.get() {
                            return;
                        }
                        if let Some(checked) = input_checked(&event) {
                            config.update(|slot| {
                                if let Some(view) = slot { view.editor.word_wrap = checked; }
                            });
                        }
                    }/>
                <label for="settings-editor-minimap">"ミニマップ"</label>
                <input id="settings-editor-minimap" type="checkbox" prop:disabled=move || !editable.get() || saving.get()
                    prop:checked=move || config.get().map(|v| v.editor.minimap).unwrap_or(false)
                    on:change=move |event| {
                        if saving.get() {
                            return;
                        }
                        if let Some(checked) = input_checked(&event) {
                            config.update(|slot| {
                                if let Some(view) = slot { view.editor.minimap = checked; }
                            });
                        }
                    }/>
                <label for="settings-editor-insert-spaces">"スペースを挿入"</label>
                <input id="settings-editor-insert-spaces" type="checkbox" prop:disabled=move || !editable.get() || saving.get()
                    prop:checked=move || config.get().map(|v| v.editor.insert_spaces).unwrap_or(false)
                    on:change=move |event| {
                        if saving.get() {
                            return;
                        }
                        if let Some(checked) = input_checked(&event) {
                            config.update(|slot| {
                                if let Some(view) = slot { view.editor.insert_spaces = checked; }
                            });
                        }
                    }/>
                <label for="settings-editor-vim-mode">"Vim モード"</label>
                <input id="settings-editor-vim-mode" type="checkbox" prop:disabled=move || !editable.get() || saving.get()
                    prop:checked=move || config.get().map(|v| v.editor.vim_mode).unwrap_or(false)
                    on:change=move |event| {
                        if saving.get() {
                            return;
                        }
                        if let Some(checked) = input_checked(&event) {
                            config.update(|slot| {
                                if let Some(view) = slot { view.editor.vim_mode = checked; }
                            });
                        }
                    }/>
                <label for="settings-editor-match-brackets">"対応する括弧を強調"</label>
                <input id="settings-editor-match-brackets" type="checkbox" prop:disabled=move || !editable.get() || saving.get()
                    prop:checked=move || config.get().map(|v| v.editor.match_brackets).unwrap_or(false)
                    on:change=move |event| {
                        if saving.get() {
                            return;
                        }
                        if let Some(checked) = input_checked(&event) {
                            config.update(|slot| {
                                if let Some(view) = slot { view.editor.match_brackets = checked; }
                            });
                        }
                    }/>
                <label for="settings-editor-auto-close-brackets">"括弧を自動で閉じる"</label>
                <input id="settings-editor-auto-close-brackets" type="checkbox" prop:disabled=move || !editable.get() || saving.get()
                    prop:checked=move || config.get().map(|v| v.editor.auto_close_brackets).unwrap_or(false)
                    on:change=move |event| {
                        if saving.get() {
                            return;
                        }
                        if let Some(checked) = input_checked(&event) {
                            config.update(|slot| {
                                if let Some(view) = slot { view.editor.auto_close_brackets = checked; }
                            });
                        }
                    }/>
                <label for="settings-editor-indent-with-tabs">"タブでインデント"</label>
                <input id="settings-editor-indent-with-tabs" type="checkbox" prop:disabled=move || !editable.get() || saving.get()
                    prop:checked=move || config.get().map(|v| v.editor.indent_with_tabs).unwrap_or(false)
                    on:change=move |event| {
                        if saving.get() {
                            return;
                        }
                        if let Some(checked) = input_checked(&event) {
                            config.update(|slot| {
                                if let Some(view) = slot { view.editor.indent_with_tabs = checked; }
                            });
                        }
                    }/>
            </section>
            <section>
                <h3>"リーダー設定"</h3>
                <p class="settings-notice">"リーダー設定: config.toml の設定値です。実際の表示はブラウザに保存された表示設定が権威です（ADR-0211）。"</p>
                <label for="settings-reader-font-family">"フォントファミリー"</label>
                <input id="settings-reader-font-family" type="text" readonly=move || !editable.get() || saving.get() prop:disabled=move || saving.get()
                    prop:value=move || config.get().map(|v| v.reader.font_family).unwrap_or_default()
                    on:input=move |event| {
                        if saving.get() {
                            return;
                        }
                        if let Some(value) = input_value(&event) {
                            config.update(|slot| {
                                if let Some(view) = slot { view.reader.font_family = value; }
                            });
                        }
                    }/>
                <label for="settings-reader-font-size">"フォントサイズ"</label>
                <input id="settings-reader-font-size" type="number" min="10" max="48" readonly=move || !editable.get() || saving.get() prop:disabled=move || saving.get()
                    prop:value=move || config.get().map(|v| v.reader.font_size.to_string()).unwrap_or_default()
                    on:input=move |event| {
                        if saving.get() {
                            return;
                        }
                        if let Some(parsed) = input_value(&event).and_then(|value| value.parse::<f32>().ok()) {

                            config.update(|slot| {

                                if let Some(view) = slot { view.reader.font_size = parsed; }

                            });

                        }
                    }/>
                <label for="settings-reader-line-height">"行の高さ"</label>
                <input id="settings-reader-line-height" type="number" step="0.1" readonly=move || !editable.get() || saving.get() prop:disabled=move || saving.get()
                    prop:value=move || config.get().map(|v| v.reader.line_height.to_string()).unwrap_or_default()
                    on:input=move |event| {
                        if saving.get() {
                            return;
                        }
                        if let Some(parsed) = input_value(&event).and_then(|value| value.parse::<f32>().ok()) {

                            config.update(|slot| {

                                if let Some(view) = slot { view.reader.line_height = parsed; }

                            });

                        }
                    }/>
                <label for="settings-reader-theme">"テーマ"</label>
                <input id="settings-reader-theme" type="text" readonly=move || !editable.get() || saving.get() prop:disabled=move || saving.get()
                    prop:value=move || config.get().map(|v| v.reader.theme).unwrap_or_default()
                    on:input=move |event| {
                        if saving.get() {
                            return;
                        }
                        if let Some(value) = input_value(&event) {
                            config.update(|slot| {
                                if let Some(view) = slot { view.reader.theme = value; }
                            });
                        }
                    }/>
                <label for="settings-reader-column-width">"列幅"</label>
                <input id="settings-reader-column-width" type="number" min="1" readonly=move || !editable.get() || saving.get() prop:disabled=move || saving.get()
                    prop:value=move || config.get().and_then(|v| v.reader.column_width.map(|n| n.to_string())).unwrap_or_default()
                    on:input=move |event| {
                        if saving.get() {
                            return;
                        }
                        if let Some(value) = input_value(&event) {
                            let parsed = value.parse::<u32>().ok();
                            config.update(|slot| {
                                if let Some(view) = slot { view.reader.column_width = parsed; }
                            });
                        }
                    }/>
                <label for="settings-reader-scroll-mode">"スクロールモード"</label>
                <input id="settings-reader-scroll-mode" type="text" readonly=move || !editable.get() || saving.get() prop:disabled=move || saving.get()
                    prop:value=move || config.get().map(|v| v.reader.scroll_mode).unwrap_or_default()
                    on:input=move |event| {
                        if saving.get() {
                            return;
                        }
                        if let Some(value) = input_value(&event) {
                            config.update(|slot| {
                                if let Some(view) = slot { view.reader.scroll_mode = value; }
                            });
                        }
                    }/>
                <label for="settings-reader-hyphenation">"ハイフネーション"</label>
                <input id="settings-reader-hyphenation" type="checkbox" prop:disabled=move || !editable.get() || saving.get()
                    prop:checked=move || config.get().map(|v| v.reader.hyphenation).unwrap_or(false)
                    on:change=move |event| {
                        if saving.get() {
                            return;
                        }
                        if let Some(checked) = input_checked(&event) {
                            config.update(|slot| {
                                if let Some(view) = slot { view.reader.hyphenation = checked; }
                            });
                        }
                    }/>
                <label for="settings-reader-vertical-writing">"縦書き"</label>
                <input id="settings-reader-vertical-writing" type="checkbox" prop:disabled=move || !editable.get() || saving.get()
                    prop:checked=move || config.get().map(|v| v.reader.vertical_writing).unwrap_or(false)
                    on:change=move |event| {
                        if saving.get() {
                            return;
                        }
                        if let Some(checked) = input_checked(&event) {
                            config.update(|slot| {
                                if let Some(view) = slot { view.reader.vertical_writing = checked; }
                            });
                        }
                    }/>
                <label for="settings-reader-page-numbers">"ページ番号を表示"</label>
                <input id="settings-reader-page-numbers" type="checkbox" prop:disabled=move || !editable.get() || saving.get()
                    prop:checked=move || config.get().map(|v| v.reader.show_page_numbers).unwrap_or(false)
                    on:change=move |event| {
                        if saving.get() {
                            return;
                        }
                        if let Some(checked) = input_checked(&event) {
                            config.update(|slot| {
                                if let Some(view) = slot { view.reader.show_page_numbers = checked; }
                            });
                        }
                    }/>
                <label for="settings-reader-remember-position">"読書位置を記憶"</label>
                <input id="settings-reader-remember-position" type="checkbox" prop:disabled=move || !editable.get() || saving.get()
                    prop:checked=move || config.get().map(|v| v.reader.remember_position).unwrap_or(false)
                    on:change=move |event| {
                        if saving.get() {
                            return;
                        }
                        if let Some(checked) = input_checked(&event) {
                            config.update(|slot| {
                                if let Some(view) = slot { view.reader.remember_position = checked; }
                            });
                        }
                    }/>
            </section>
            <section>
                <h3>"保存先設定"</h3>
                <p class="settings-notice">"保存先設定は config.toml が権威です。path と秘密情報は画面へ表示しません。"</p>
                // ADR-0304: default_backend / max_recent は未配線の死設定。
                // DTO から除外したため固定文言で表示する。
                <p class="settings-notice">"保存先 backend: 未配線（config.toml が権威）"</p>
                <p class="settings-notice">"最近使用数: 未配線"</p>
            </section>
            <button id="settings-save" type="button" class="btn btn-primary" prop:disabled=move || !editable.get() || saving.get() on:click=on_save>
                {move || if editable.get() { "設定を保存" } else { "保存はデスクトップ版のみ" }}
            </button>
        </form>
    }
}

/// カンマ区切りのルール入力を trim して Vec へ分ける（空要素は除外）。
fn split_rules(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(|rule| rule.trim().to_string())
        .filter(|rule| !rule.is_empty())
        .collect()
}
