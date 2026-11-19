//! WebAssembly bindings for openrustbooks-core.
//!
//! This module exposes the core functionality to JavaScript/TypeScript
//! via wasm-bindgen for use in the browser version.

#[cfg(target_arch = "wasm32")]
use crate::epub::{ChapterLink, EpubBook, EpubCoverImage, EpubTocEntry};
#[cfg(target_arch = "wasm32")]
use serde::{Deserialize, Serialize};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[cfg(target_arch = "wasm32")]
macro_rules! console_log {
    ($($t:tt)*) => (log(&format!($($t)*)))
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

/// EPUB 読み取り API を JavaScript に公開するラッパー。
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct WasmEpubBook {
    inner: EpubBook,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl WasmEpubBook {
    /// EPUB ZIP のバイト列から本を開く。
    #[wasm_bindgen(constructor)]
    pub fn new(bytes: Vec<u8>) -> Result<WasmEpubBook, JsValue> {
        // JS から渡されたバイト列の所有権をそのまま使い、二重コピーを避ける。
        let inner =
            EpubBook::open_owned(bytes).map_err(|error| JsValue::from_str(&error.to_string()))?;
        Ok(Self { inner })
    }

    /// spine に含まれる章の数を返す。
    pub fn chapter_count(&self) -> usize {
        self.inner.chapters().len()
    }

    /// 指定章のアーカイブ内パスを返す。
    pub fn chapter_href(&self, index: usize) -> Result<String, JsValue> {
        self.inner
            .chapters()
            .get(index)
            .map(|chapter| chapter.href.clone())
            .ok_or_else(|| {
                js_error(crate::error::CoreError::DocumentNotFound(format!(
                    "章の index が範囲外です: {index}"
                )))
            })
    }

    /// 指定章の XHTML を返す。
    pub fn chapter_html(&self, index: usize) -> Result<String, JsValue> {
        self.inner
            .chapter_html(index)
            .map_err(|error| JsValue::from_str(&error.to_string()))
    }

    /// 指定章のプレーンテキストを返す。
    pub fn chapter_text(&self, index: usize) -> Result<String, JsValue> {
        self.inner
            .chapter_text(index)
            .map_err(|error| JsValue::from_str(&error.to_string()))
    }

    /// 指定章の表示名を返す。情報が無ければ `undefined`。
    pub fn chapter_title(&self, index: usize) -> Result<Option<String>, JsValue> {
        self.inner
            .chapter_title(index)
            .map_err(|error| JsValue::from_str(&error.to_string()))
    }

    /// 指定章から見た相対リソースをバイト列で返す。
    pub fn chapter_resource(&self, index: usize, href: String) -> Result<Vec<u8>, JsValue> {
        self.inner
            .chapter_resource(index, &href)
            .map_err(|error| JsValue::from_str(&error.to_string()))
    }

    /// 章内の相対リンクを同じ本の章へ解決する。
    ///
    /// 解決できた場合は `{ chapterIndex, fragment }`、移動しない場合は
    /// `null` を返す。
    pub fn resolve_chapter_link(&self, index: usize, href: String) -> Result<JsValue, JsValue> {
        let link = self
            .inner
            .resolve_chapter_link(index, &href)
            .map_err(js_error)?;
        serialize_chapter_link(link)
    }

    /// EPUB3 nav の目次を JavaScript 値に変換して返す。
    pub fn toc(&self) -> Result<JsValue, JsValue> {
        let entries = self.inner.toc().map_err(js_error)?;
        serialize_navigation(entries)
    }

    /// EPUB3 nav の landmarks を JavaScript 値に変換して返す。
    pub fn landmarks(&self) -> Result<JsValue, JsValue> {
        let entries = self.inner.landmarks().map_err(js_error)?;
        serialize_navigation(entries)
    }

    /// EPUB3 nav の page-list を JavaScript 値に変換して返す。
    pub fn page_list(&self) -> Result<JsValue, JsValue> {
        let entries = self.inner.page_list().map_err(js_error)?;
        serialize_navigation(entries)
    }

    /// manifest が `remote-resources` を宣言しているか。
    pub fn has_remote_resources(&self) -> bool {
        self.inner.has_remote_resources()
    }

    /// EPUB の言語（例: `ja`, `en`）。不明なら `undefined`。
    pub fn language(&self) -> Option<String> {
        self.inner.metadata().language.clone()
    }

    /// 表紙画像の情報を JavaScript 値に変換して返す。無ければ空値。
    pub fn cover_image(&self) -> Result<JsValue, JsValue> {
        let cover = self.inner.cover_image().map_err(js_error)?;
        serialize_cover(cover)
    }
}

#[cfg(target_arch = "wasm32")]
fn js_error(error: crate::error::CoreError) -> JsValue {
    JsValue::from_str(&error.to_string())
}

#[cfg(target_arch = "wasm32")]
fn serialize_navigation(entries: Vec<EpubTocEntry>) -> Result<JsValue, JsValue> {
    let entries: Vec<WasmEpubTocEntry> = entries.into_iter().map(Into::into).collect();
    serde_wasm_bindgen::to_value(&entries).map_err(|error| JsValue::from_str(&error.to_string()))
}

#[cfg(target_arch = "wasm32")]
fn serialize_cover(cover: Option<EpubCoverImage>) -> Result<JsValue, JsValue> {
    let cover: Option<WasmEpubCoverImage> = cover.map(Into::into);
    serde_wasm_bindgen::to_value(&cover).map_err(|error| JsValue::from_str(&error.to_string()))
}

#[cfg(target_arch = "wasm32")]
fn serialize_chapter_link(link: Option<ChapterLink>) -> Result<JsValue, JsValue> {
    let Some(link) = link else {
        return Ok(JsValue::NULL);
    };
    serde_wasm_bindgen::to_value(&WasmChapterLink {
        chapter_index: link.chapter_index,
        fragment: link.fragment,
    })
    .map_err(|error| JsValue::from_str(&error.to_string()))
}

#[cfg(target_arch = "wasm32")]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct WasmChapterLink {
    chapter_index: usize,
    fragment: Option<String>,
}

#[cfg(target_arch = "wasm32")]
#[derive(Serialize)]
struct WasmEpubTocEntry {
    title: String,
    href: String,
    fragment: Option<String>,
    depth: usize,
}

#[cfg(target_arch = "wasm32")]
impl From<EpubTocEntry> for WasmEpubTocEntry {
    fn from(entry: EpubTocEntry) -> Self {
        Self {
            title: entry.title,
            href: entry.href,
            fragment: entry.fragment,
            depth: entry.depth,
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[derive(Serialize)]
struct WasmEpubCoverImage {
    id: String,
    href: String,
    media_type: String,
    bytes: Vec<u8>,
}

#[cfg(target_arch = "wasm32")]
impl From<EpubCoverImage> for WasmEpubCoverImage {
    fn from(cover: EpubCoverImage) -> Self {
        Self {
            id: cover.id,
            href: cover.href,
            media_type: cover.media_type,
            bytes: cover.bytes,
        }
    }
}

/// ZIP アーカイブの読み取り専用 Wasm API（TD-M22 段階 1 / ADR-0223）。
///
/// パス検証・サイズ制限・不正エントリ拒否の権威は
/// [`crate::archive::ArchiveReader`] が持ち、ここでは JS 境界向けの変換と
/// fail-closed な固定エラー文言だけを提供する。
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct WasmArchive {
    inner: crate::archive::ArchiveReader,
}

// JS 境界に出すエラーは固定文言のみ。CoreError の `to_string()` はパスや
// ZipError 本文を含むため絶対に流用しない（既存の `js_error()` も使わない）。
#[cfg(target_arch = "wasm32")]
const ERR_INVALID_ARCHIVE: &str = "Invalid ZIP archive";
#[cfg(target_arch = "wasm32")]
const ERR_ENTRY_READ: &str = "ZIP entry read failed";
#[cfg(target_arch = "wasm32")]
const ERR_INVALID_UTF8: &str = "ZIP entry is not valid UTF-8";
#[cfg(target_arch = "wasm32")]
const ERR_ENTRIES: &str = "ZIP entries unavailable";

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl WasmArchive {
    /// ZIP バイト列から読み取り専用ビューを開く。
    ///
    /// 所有権をそのまま `ArchiveReader::from_owned` へ渡し二重コピーを避ける。
    /// いかなる失敗も固定文言へ変換し、内部の理由は露出しない。
    #[wasm_bindgen(constructor)]
    pub fn new(bytes: Vec<u8>) -> Result<WasmArchive, JsValue> {
        let inner = crate::archive::ArchiveReader::from_owned(
            bytes,
            crate::archive::ArchiveLimits::default(),
        )
        .map_err(|_| JsValue::from_str(ERR_INVALID_ARCHIVE))?;
        Ok(Self { inner })
    }

    /// エントリ一覧を camelCase の JS 配列で返す。
    pub fn entries(&self) -> Result<JsValue, JsValue> {
        let entries: Vec<WasmArchiveEntry> = self
            .inner
            .entries()
            .iter()
            .map(|entry| WasmArchiveEntry {
                path: entry.path.clone(),
                is_dir: entry.is_dir,
                size: entry.size,
                compressed_size: entry.compressed_size,
            })
            .collect();
        serde_wasm_bindgen::to_value(&entries).map_err(|_| JsValue::from_str(ERR_ENTRIES))
    }

    /// 指定パスのエントリをバイト列で返す。
    ///
    /// パス逸脱・未検出・壊れたデータなど、全失敗を固定文言にする。
    pub fn read_entry(&self, path: &str) -> Result<Vec<u8>, JsValue> {
        self.inner
            .read_entry(path)
            .map_err(|_| JsValue::from_str(ERR_ENTRY_READ))
    }

    /// 指定パスのエントリを UTF-8 テキストで返す（先頭 BOM は除去）。
    ///
    /// 読み出し失敗と UTF-8 不正を別の固定文言にするため、バイト列取得は
    /// `ArchiveReader::read_entry` に委譲し、デコードのみここで行う。
    pub fn read_entry_text(&self, path: &str) -> Result<String, JsValue> {
        let bytes = self.read_entry(path)?;
        let text = String::from_utf8(bytes).map_err(|_| JsValue::from_str(ERR_INVALID_UTF8))?;
        Ok(text.strip_prefix('\u{FEFF}').unwrap_or(&text).to_string())
    }

    /// `mimetype` が先頭で内容が `application/epub+zip` なら true。
    pub fn is_epub(&self) -> bool {
        self.inner.is_epub()
    }
}

/// `entries()` の JS 公開用 DTO（camelCase: `path` / `isDir` / `size` / `compressedSize`）。
#[cfg(target_arch = "wasm32")]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WasmArchiveEntry {
    path: String,
    is_dir: bool,
    size: u64,
    compressed_size: u64,
}

#[cfg(target_arch = "wasm32")]
use crate::{
    ai_markdown::lint::{LintIssue, LintSeverity, MarkdownLinter},
    config::LintConfig,
};

/// Wasm-compatible Markdown linter
///
/// featureless の rumdl を直接配線した実 lint。ADR-0156 の fail-closed スタブを
/// 解消し、ブラウザでも `MarkdownLinter` の判定結果をそのまま返す。
/// `converged` / `conflicting_rules` は `WasmLintFixResult` にまだ公開しない
/// （ADR-0156 の未決定を維持し、構造体変更を最小限に留める）。
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct WasmMarkdownLinter {
    inner: MarkdownLinter,
    /// `LintConfig.auto_fix` を保持し、修正経路の分岐に使う。
    auto_fix: bool,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl WasmMarkdownLinter {
    /// JS 側の設定を解釈して lint 本体を構築する。
    ///
    /// `null` / `undefined`（引数省略）は `LintConfig::default()` を使う。
    /// それ以外は `serde_wasm_bindgen::from_value` で完全な `LintConfig` を要求し、
    /// パース失敗（型不一致・必須フィールド不足）は既定へ反転させずエラーにする
    /// （fail-closed）。未知のキーは serde が無視する。パース成功後も
    /// `MarkdownLinter::new` がルール設定を検証し、不正な rule_config はエラーにする。
    #[wasm_bindgen(constructor)]
    pub fn new(config: JsValue) -> Result<Self, JsValue> {
        // null / undefined（引数省略）は既定設定。それ以外はパース失敗を
        // 黙って既定に落とさずエラーにする（部分オブジェクトの
        // `auto_fix:false` が既定 true へ反転する fail-open を避ける）。
        let lint_config = if config.is_null() || config.is_undefined() {
            LintConfig::default()
        } else {
            match serde_wasm_bindgen::from_value::<LintConfig>(config) {
                Ok(value) => value,
                Err(error) => {
                    return Err(JsValue::from_str(&format!("Invalid lint config: {error}")));
                }
            }
        };
        let auto_fix = lint_config.auto_fix;
        let inner = MarkdownLinter::new(&lint_config).map_err(js_error)?;
        Ok(Self { inner, auto_fix })
    }

    /// Markdown を lint し、issues を JS 配列として返す。
    #[wasm_bindgen]
    pub fn lint(&self, content: String) -> Result<JsValue, JsValue> {
        console_log!("Linting markdown (content: {} chars)", content.len());
        let issues = self.inner.lint(&content).map_err(js_error)?;
        let wasm_issues: Vec<WasmLintIssue> = issues.iter().map(to_wasm_issue).collect();
        serde_wasm_bindgen::to_value(&wasm_issues)
            .map_err(|error| JsValue::from_str(&error.to_string()))
    }

    /// Lint し、`auto_fix` が有効なら自動修正して結果を返す。
    ///
    /// 分岐そのものは `validate_with_auto_fix` に集約し、ここでは
    /// `LintResultWrapper` を JS 公開用の `WasmLintFixResult` へ変換するだけ。
    #[wasm_bindgen]
    pub async fn lint_and_fix(&self, content: String) -> Result<WasmLintFixResult, JsValue> {
        let result =
            crate::ai_markdown::lint::validate_with_auto_fix(&self.inner, &content, self.auto_fix)
                .await
                .map_err(js_error)?;

        Ok(WasmLintFixResult {
            passed: result.passed,
            issues: result.issues.iter().map(to_wasm_issue).collect(),
            fixed_content: result.fixed_content,
            fixes_applied: result.fixes_applied,
        })
    }
}

/// Wasm-compatible lint issue
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
#[derive(Serialize, Deserialize)]
pub struct WasmLintIssue {
    rule: String,
    rule_name: String,
    line: usize,
    column: usize,
    // GOAL.md の必須事項に従い、native の LintIssue と同じ範囲情報を公開する。
    end_line: Option<usize>,
    end_column: Option<usize>,
    message: String,
    severity: String,
    fixable: bool,
}

/// `LintIssue` を JS 公開用の `WasmLintIssue` へ変換する。
///
/// severity は `LintSeverity` の serde 表現（lowercase）と揃えた文字列にする。
#[cfg(target_arch = "wasm32")]
fn to_wasm_issue(issue: &LintIssue) -> WasmLintIssue {
    let severity = match issue.severity {
        LintSeverity::Error => "error",
        LintSeverity::Warning => "warning",
        LintSeverity::Info => "info",
    };
    WasmLintIssue {
        rule: issue.rule.clone(),
        rule_name: issue.rule_name.clone(),
        line: issue.line,
        column: issue.column,
        end_line: issue.end_line,
        end_column: issue.end_column,
        message: issue.message.clone(),
        severity: severity.to_string(),
        fixable: issue.fixable,
    }
}

/// Wasm-compatible lint fix result
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
#[derive(Serialize, Deserialize)]
pub struct WasmLintFixResult {
    passed: bool,
    issues: Vec<WasmLintIssue>,
    fixed_content: String,
    fixes_applied: usize,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl WasmLintFixResult {
    #[wasm_bindgen(getter)]
    pub fn passed(&self) -> bool {
        self.passed
    }

    #[wasm_bindgen(getter)]
    pub fn issues(&self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(&self.issues)
            .map_err(|error| JsValue::from_str(&error.to_string()))
    }

    #[wasm_bindgen(getter)]
    pub fn fixed_content(&self) -> String {
        self.fixed_content.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn fixes_applied(&self) -> usize {
        self.fixes_applied
    }
}

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
