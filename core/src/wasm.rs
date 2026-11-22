//! WebAssembly bindings for openrustbooks-core.
//!
//! This module exposes the core functionality to JavaScript/TypeScript
//! via wasm-bindgen for use in the browser version.
//!
//! 責務別の実装は [`epub`] / [`archive`] / [`lint`] に分離し、ここには共有基盤
//! （初期化・共通エラー変換・JS 境界の薄いラッパー）と再エクスポートだけを残す。

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// テキストスコープのマクロは、定義より後に宣言された子モジュールからも使える。
// `$crate::wasm::log` を明示して呼び出し側のスコープに依存しないようにする。
#[cfg(target_arch = "wasm32")]
macro_rules! console_log {
    ($($t:tt)*) => ($crate::wasm::log(&format!($($t)*)))
}

/// Wasm 向けの初期化処理（Rust / JS の両方から共有する）。
#[cfg(target_arch = "wasm32")]
pub(crate) fn init_core() {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();
}

/// Initialize the library for Wasm use
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn init() {
    init_core();
    console_log!("openrustbooks-core initialized");
}

/// 画像バイト列の自然寸法を `[width, height]` で返す（取得不可なら `undefined`）。
///
/// pure 関数 [`crate::image_dimensions`] の wasm-bindgen ラッパー。web の
/// `resolve_images()` が `chapter_resource()` のバイト列を渡して使う。
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn image_dimensions_wasm(bytes: &[u8]) -> Option<Vec<u32>> {
    crate::image_dimensions(bytes).map(|(width, height)| vec![width, height])
}

/// Markdown をサニタイズ済み HTML へ変換する Wasm 公開 API。
///
/// 純粋関数 [`crate::render_markdown`] の薄いラッパー。ネイティブの
/// `cargo check --all-targets` でも web クレートから参照できるよう、
/// `wasm_bindgen` 属性は wasm32 のときだけ付与する。
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn render_markdown_wasm(md: &str) -> String {
    crate::render_markdown(md)
}

/// SVG バイト列をサニタイズする Wasm 公開 API。
///
/// 純粋関数 [`crate::sanitize::sanitize_svg_xml`] の薄いラッパー。UTF-8 変換
/// とサニタイズに失敗した場合は `None`（fail-closed）。`render_markdown_wasm`
/// と同じく `wasm_bindgen` 属性は wasm32 のときだけ付与し、ネイティブの
/// `cargo check --all-targets` からも web クレートが参照できるようにする。
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn sanitize_svg_wasm(bytes: &[u8]) -> Option<Vec<u8>> {
    let svg = std::str::from_utf8(bytes).ok()?;
    crate::sanitize::sanitize_svg_xml(svg).map(String::into_bytes)
}

/// `serde_wasm_bindgen` の変換失敗など、CoreError 以外を JS 境界へ出すときの固定文言。
#[cfg(target_arch = "wasm32")]
pub(crate) const SERIALIZATION_FAILED_MESSAGE: &str = "Serialization failed";

/// JS から渡された lint 設定を解釈できないときの固定文言。
///
/// `wasm/tests.rs` の wasm unit test が `Invalid lint config` という接頭辞に
/// 依存するため維持する（ADR-0160 の fail-closed 契約も同じ文言）。
#[cfg(target_arch = "wasm32")]
pub(crate) const INVALID_LINT_CONFIG_MESSAGE: &str = "Invalid lint config";

/// `CoreError` を JS 境界向けの固定文言へ変換する共有ヘルパー。
///
/// 子モジュール（`epub` / `lint`）から `super::js_error` として利用する。
/// `Display` はパス / host / 本文を含み得るため使わず、`code()` を共有 mapper
/// [`crate::error::fixed_message`] へ通す。専用の固定文言を使う `archive` は
/// この関数を使わない。
#[cfg(target_arch = "wasm32")]
fn js_error(error: crate::error::CoreError) -> JsValue {
    JsValue::from_str(crate::error::fixed_message(error.code()))
}

// 責務別モジュール。native では子モジュール自体をコンパイルしない。
#[cfg(target_arch = "wasm32")]
mod archive;
#[cfg(target_arch = "wasm32")]
mod epub;
#[cfg(target_arch = "wasm32")]
mod lint;

// `lib.rs` の `pub use wasm::*` で従来どおりの公開パスを維持する。
#[cfg(target_arch = "wasm32")]
pub use archive::WasmArchive;
#[cfg(target_arch = "wasm32")]
pub use epub::WasmEpubBook;
#[cfg(target_arch = "wasm32")]
pub use lint::{WasmLintFixResult, WasmLintIssue, WasmMarkdownLinter};

// `wasm/tests.rs` が参照する DTO と ADR-0223 の 4 固定文言は、元の実効可視性を
// 保つためテストビルド時だけ親へ取り込む（公開 API には含めない）。
// `ERR_ENTRIES` は tests が直接参照しないため unused を局限的に許容する。
#[cfg(all(test, target_arch = "wasm32"))]
use archive::WasmArchiveEntry;
#[cfg(all(test, target_arch = "wasm32"))]
#[allow(unused_imports)]
use archive::{ERR_ENTRIES, ERR_ENTRY_READ, ERR_INVALID_ARCHIVE, ERR_INVALID_UTF8};

/// Utility functions for Wasm
#[cfg(target_arch = "wasm32")]
pub mod utils {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub fn version() -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }
}

// Non-Wasm stubs for native compilation
#[cfg(not(target_arch = "wasm32"))]
pub fn init() {
    crate::init();
}

#[cfg(not(target_arch = "wasm32"))]
pub mod utils {
    pub fn version() -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
#[rustfmt::skip]
#[path = "wasm/tests.rs"]
mod tests;
