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

/// Reader component
#[cfg(target_arch = "wasm32")]
#[component]
fn Reader(doc_id: Signal<String>) -> impl IntoView {
    use js_sys::Uint8Array;
    use serde::Deserialize;
    use std::{cell::RefCell, collections::HashMap, rc::Rc};
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::{JsFuture, spawn_local};
    use web_sys::{Event, HtmlInputElement};

    #[derive(Deserialize)]
    struct TocEntry {
        title: String,
        href: String,
        depth: usize,
    }

    let book = Rc::new(RefCell::new(None::<WasmEpubBook>));
    let (chapters, set_chapters) = signal(Vec::<(usize, String)>::new());
    let (current, set_current) = signal(0_usize);
    let (content, set_content) = signal(String::new());
    let (status, set_status) = signal(String::from("Choose an EPUB file to begin."));

    let load_chapter = {
        let book = Rc::clone(&book);
        Rc::new(move |index: usize| {
            let result = book
                .borrow()
                .as_ref()
                .ok_or_else(|| String::from("No EPUB is open."))
                .and_then(|book| {
                    book.chapter_text(index)
                        .map_err(|error| format!("{error:?}"))
                });
            match result {
                Ok(text) => {
                    set_current.set(index);
                    set_content.set(text);
                    set_status.set(String::new());
                }
                Err(error) => set_status.set(error),
            }
        })
    };

    let on_file_change = {
        let book = Rc::clone(&book);
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
            set_status.set(format!("Opening {}…", file.name()));
            set_content.set(String::new());
            set_chapters.set(Vec::new());
            *book.borrow_mut() = None;

            let book = Rc::clone(&book);
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
                let mut toc_titles = HashMap::new();
                for entry in toc {
                    let indent = "　".repeat(entry.depth.min(8));
                    toc_titles
                        .entry(entry.href)
                        .or_insert_with(|| format!("{indent}{}", entry.title));
                }
                let options: Vec<(usize, String)> = spine
                    .into_iter()
                    .map(|(index, href)| {
                        let title = toc_titles.remove(&href).unwrap_or(href);
                        (index, title)
                    })
                    .collect();

                match opened.chapter_text(0) {
                    Ok(text) => {
                        set_current.set(0);
                        set_content.set(text);
                        set_chapters.set(options);
                        set_status.set(String::new());
                        *book.borrow_mut() = Some(opened);
                    }
                    Err(error) => set_status.set(format!("Chapter read failed: {error:?}")),
                }
            });
        }
    };

    let on_select = {
        let load_chapter = Rc::clone(&load_chapter);
        move |event: Event| {
            if let Some(input) = event
                .target()
                .and_then(|target| target.dyn_into::<HtmlInputElement>().ok())
                && let Ok(index) = input.value().parse::<usize>()
            {
                load_chapter(index);
            }
        }
    };
    let on_previous = {
        let load_chapter = Rc::clone(&load_chapter);
        move |_| {
            let index = current.get();
            if index > 0 {
                load_chapter(index - 1);
            }
        }
    };
    let on_next = {
        let load_chapter = Rc::clone(&load_chapter);
        let chapters = chapters;
        move |_| {
            let index = current.get();
            if chapters.get().iter().any(|(chapter, _)| *chapter > index) {
                load_chapter(index + 1);
            }
        }
    };

    view! {
        <div class="reader-container" data-doc-id=move || doc_id.get()>
            <div class="reader-toolbar">
                <label class="btn btn-primary" for="epub-file">"Open EPUB"</label>
                <input id="epub-file" class="reader-file-input" type="file" accept=".epub,application/epub+zip" on:change=on_file_change/>
                <button class="btn" on:click=on_previous disabled=move || current.get() == 0>"◀"</button>
                <select class="reader-chapter-select" on:change=on_select prop:disabled=move || chapters.get().is_empty()>
                    <For
                        each=move || chapters.get()
                        key=|(index, _)| *index
                        children=move |(index, title)| view! {
                            <option value=index.to_string() prop:selected=move || current.get() == index>
                                {title}
                            </option>
                        }
                    />
                </select>
                <button class="btn" on:click=on_next disabled=move || {
                    let index = current.get();
                    !chapters.get().iter().any(|(chapter, _)| *chapter > index)
                }>
                    "▶"
                </button>
            </div>
            <p class="reader-status" role="status">{move || status.get()}</p>
            <div class="reader-content">
                <pre class="document-content reader-text">{move || content.get()}</pre>
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
        <div class="preview" inner_html=move || {
            // Would use comrak or pulldown-cmark to render
            content.get()
        }/>
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
