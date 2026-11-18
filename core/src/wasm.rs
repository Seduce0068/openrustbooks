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

/// Wasm-compatible document representation
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
#[derive(Serialize, Deserialize)]
pub struct WasmDocument {
    id: String,
    format: String,
    title: Option<String>,
    content: String,
    /// Front matter is arbitrary data, so it is kept as a JSON value rather
    /// than a `JsValue` (which has no `serde` impls).
    front_matter: Option<serde_json::Value>,
    metadata: serde_json::Value,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl WasmDocument {
    #[wasm_bindgen(constructor)]
    pub fn new(id: String, format: String, content: String) -> Self {
        Self {
            id,
            format,
            title: None,
            content,
            front_matter: None,
            metadata: serde_json::Value::Null,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String {
        self.id.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn format(&self) -> String {
        self.format.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn title(&self) -> Option<String> {
        self.title.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_title(&mut self, title: String) {
        self.title = Some(title);
    }

    #[wasm_bindgen(getter)]
    pub fn content(&self) -> String {
        self.content.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_content(&mut self, content: String) {
        self.content = content;
    }
}

/// Wasm-compatible AI pipeline result
///
/// Not `serde`-derivable: [`WasmDiffResult`] holds `JsValue`s by design.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct WasmPipelineResult {
    content: String,
    diff: Option<WasmDiffResult>,
    lint_passed: bool,
    attempts: usize,
    tokens_used: usize,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl WasmPipelineResult {
    #[wasm_bindgen(getter)]
    pub fn content(&self) -> String {
        self.content.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn diff(&self) -> Option<WasmDiffResult> {
        self.diff.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn lint_passed(&self) -> bool {
        self.lint_passed
    }

    #[wasm_bindgen(getter)]
    pub fn attempts(&self) -> usize {
        self.attempts
    }

    #[wasm_bindgen(getter)]
    pub fn tokens_used(&self) -> usize {
        self.tokens_used
    }
}

/// Wasm-compatible diff result
///
/// Fields are `JsValue` so the JS side receives plain objects; that also means
/// the struct cannot derive `Serialize`/`Deserialize` (see above in this module).
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
#[derive(Clone)]
pub struct WasmDiffResult {
    hunks: JsValue,
    stats: JsValue,
    original_lines: usize,
    modified_lines: usize,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl WasmDiffResult {
    #[wasm_bindgen(getter)]
    pub fn hunks(&self) -> JsValue {
        self.hunks.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn stats(&self) -> JsValue {
        self.stats.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn original_lines(&self) -> usize {
        self.original_lines
    }

    #[wasm_bindgen(getter)]
    pub fn modified_lines(&self) -> usize {
        self.modified_lines
    }
}

/// Wasm-compatible AI Markdown service
///
/// fail-closed: ブラウザには AI クライアントが無いため、すべてのメソッドは
/// 架空の成功値を返さずエラーを返す（ADR-0099）。
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct WasmAiMarkdownService {
    /// Reserved for the browser AI client, which is not implemented yet.
    #[allow(dead_code)]
    config: JsValue,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl WasmAiMarkdownService {
    #[wasm_bindgen(constructor)]
    pub fn new(config: JsValue) -> Self {
        Self { config }
    }

    /// Generate Markdown from prompt
    #[wasm_bindgen]
    pub async fn generate(
        &self,
        prompt: String,
        _context: Option<String>,
    ) -> Result<WasmPipelineResult, JsValue> {
        console_log!("AI generate unavailable (prompt: {} chars)", prompt.len());
        Err(JsValue::from_str(
            "AI features are not available in the browser.",
        ))
    }

    /// Refine existing Markdown
    #[wasm_bindgen]
    pub async fn refine(
        &self,
        content: String,
        instruction: String,
        _context: Option<String>,
    ) -> Result<WasmPipelineResult, JsValue> {
        console_log!(
            "AI refine unavailable (content: {} chars, instruction: {} chars)",
            content.len(),
            instruction.len()
        );
        Err(JsValue::from_str(
            "AI features are not available in the browser.",
        ))
    }

    /// Convert to table
    #[wasm_bindgen]
    pub async fn to_table(
        &self,
        _data: String,
        format: String,
        _headers: Option<JsValue>,
    ) -> Result<WasmPipelineResult, JsValue> {
        console_log!("AI table conversion unavailable (format: {})", format);
        Err(JsValue::from_str(
            "AI features are not available in the browser.",
        ))
    }

    /// Generate front matter
    #[wasm_bindgen]
    pub async fn front_matter(
        &self,
        content: String,
        _schema: Option<JsValue>,
    ) -> Result<WasmPipelineResult, JsValue> {
        console_log!(
            "AI front matter unavailable (content: {} chars)",
            content.len()
        );
        Err(JsValue::from_str(
            "AI features are not available in the browser.",
        ))
    }
}

#[cfg(target_arch = "wasm32")]
use crate::{
    ai_markdown::lint::{LintIssue, LintResultWrapper, LintSeverity, MarkdownLinter, lint_passed},
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
    /// ネイティブの `validate_content` と同じ分岐を wasm 側でも再現する
    /// （`MarkdownLinter::lint_and_fix` 自体は auto_fix 引数を持たないため）。
    #[wasm_bindgen]
    pub async fn lint_and_fix(&self, content: String) -> Result<WasmLintFixResult, JsValue> {
        let result: LintResultWrapper = if self.auto_fix {
            self.inner.lint_and_fix(&content).await.map_err(js_error)?
        } else {
            let issues = self.inner.lint(&content).map_err(js_error)?;
            let passed = lint_passed(&issues, self.inner.warnings_as_errors());
            LintResultWrapper {
                passed,
                issues,
                fixed_content: content.clone(),
                fixes_applied: 0,
                // lint のみの経路では修正を行わないため収束扱いとする。
                converged: true,
                conflicting_rules: Vec::new(),
            }
        };

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

/// Wasm-compatible configuration
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
#[derive(Serialize, Deserialize)]
pub struct WasmConfig {
    ai_enabled: bool,
    ai_provider: String,
    ai_model: String,
    lint_enabled: bool,
    auto_fix: bool,
}

#[cfg(target_arch = "wasm32")]
impl Default for WasmConfig {
    fn default() -> Self {
        Self {
            // 送信スタブでもネイティブの AiConfig と揃え、既定はオフ（ADR-0099）。
            ai_enabled: false,
            ai_provider: "openai".to_string(),
            ai_model: "gpt-4o-mini".to_string(),
            lint_enabled: true,
            auto_fix: true,
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl WasmConfig {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
    }

    #[wasm_bindgen(getter)]
    pub fn ai_enabled(&self) -> bool {
        self.ai_enabled
    }
    #[wasm_bindgen(setter)]
    pub fn set_ai_enabled(&mut self, v: bool) {
        self.ai_enabled = v;
    }

    #[wasm_bindgen(getter)]
    pub fn ai_provider(&self) -> String {
        self.ai_provider.clone()
    }
    #[wasm_bindgen(setter)]
    pub fn set_ai_provider(&mut self, v: String) {
        self.ai_provider = v;
    }

    #[wasm_bindgen(getter)]
    pub fn ai_model(&self) -> String {
        self.ai_model.clone()
    }
    #[wasm_bindgen(setter)]
    pub fn set_ai_model(&mut self, v: String) {
        self.ai_model = v;
    }

    #[wasm_bindgen(getter)]
    pub fn lint_enabled(&self) -> bool {
        self.lint_enabled
    }
    #[wasm_bindgen(setter)]
    pub fn set_lint_enabled(&mut self, v: bool) {
        self.lint_enabled = v;
    }

    #[wasm_bindgen(getter)]
    pub fn auto_fix(&self) -> bool {
        self.auto_fix
    }
    #[wasm_bindgen(setter)]
    pub fn set_auto_fix(&mut self, v: bool) {
        self.auto_fix = v;
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

    #[wasm_bindgen]
    pub fn greet(name: &str) -> String {
        format!("Hello, {} from openrustbooks-core!", name)
    }

    #[wasm_bindgen]
    pub fn detect_format(filename: &str) -> String {
        let ext = std::path::Path::new(filename)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        match ext.to_lowercase().as_str() {
            "txt" => "txt",
            "md" | "markdown" | "mkd" => "markdown",
            "epub" => "epub",
            "pdf" => "pdf",
            "zip" => "zip",
            _ => "unknown",
        }
        .to_string()
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
mod tests {
    use super::{WasmAiMarkdownService, WasmConfig, WasmMarkdownLinter};
    use wasm_bindgen::JsValue;
    use wasm_bindgen_test::wasm_bindgen_test;

    /// `Result<T, JsValue>` が期待どおりのエラー文字列を返すことを検証する。
    /// `unwrap_err()` は `T: Debug` を要求するため `match` で回避する。
    fn assert_js_error<T>(result: Result<T, JsValue>, expected: &str) {
        match result {
            Ok(_) => panic!("expected an error: {expected}"),
            Err(error) => assert_eq!(error.as_string().as_deref(), Some(expected)),
        }
    }

    #[wasm_bindgen_test]
    async fn ai_methods_fail_closed() {
        const EXPECTED: &str = "AI features are not available in the browser.";
        let service = WasmAiMarkdownService::new(JsValue::NULL);

        assert_js_error(service.generate("prompt".into(), None).await, EXPECTED);
        assert_js_error(
            service
                .refine("content".into(), "instruction".into(), None)
                .await,
            EXPECTED,
        );
        assert_js_error(
            service
                .to_table("data".into(), "markdown".into(), None)
                .await,
            EXPECTED,
        );
        assert_js_error(service.front_matter("content".into(), None).await, EXPECTED);
    }

    fn linter() -> WasmMarkdownLinter {
        // null は LintConfig としてパースできないため既定設定へフォールバックする。
        WasmMarkdownLinter::new(JsValue::NULL).expect("linter should build")
    }

    /// `lint()` が返した JS 配列を rule ID のリストへ変換する。
    fn rule_names(value: JsValue) -> Vec<String> {
        let issues: Vec<super::WasmLintIssue> =
            serde_wasm_bindgen::from_value(value).expect("issues should deserialize");
        issues.into_iter().map(|issue| issue.rule).collect()
    }

    #[wasm_bindgen_test]
    fn lint_returns_no_issues_for_clean_markdown() {
        let rules = rule_names(
            linter()
                .lint("# Title\n\nClean paragraph.\n".into())
                .expect("lint should succeed"),
        );
        assert!(
            rules.is_empty(),
            "clean markdown should have no issues: {rules:?}"
        );
    }

    #[wasm_bindgen_test]
    fn lint_detects_inline_html() {
        let rules = rule_names(
            linter()
                .lint("# Title\n\n<div>html</div>\n".into())
                .expect("lint should succeed"),
        );
        assert!(
            !rules.is_empty(),
            "inline HTML should produce at least one issue"
        );
        assert!(
            rules.iter().any(|rule| rule == "MD033"),
            "MD033 should be detected: {rules:?}"
        );
    }

    #[wasm_bindgen_test]
    async fn lint_and_fix_returns_passed_and_fixed_content() {
        let dirty = "# Title\n\nTrailing spaces here   \n";
        let result = linter()
            .lint_and_fix(dirty.into())
            .await
            .expect("lint_and_fix should succeed");

        assert!(result.passed(), "fixed markdown should pass lint");
        assert_ne!(result.fixed_content(), dirty, "content should be fixed");
        assert!(
            result.fixes_applied() > 0,
            "at least one fix should be applied"
        );
    }

    #[wasm_bindgen_test]
    async fn auto_fix_false_keeps_content_unchanged() {
        // WasmConfig.auto_fix ではなく、constructor に渡した LintConfig.auto_fix が権威。
        let config = crate::config::LintConfig {
            auto_fix: false,
            ..Default::default()
        };
        let js_config = serde_wasm_bindgen::to_value(&config).expect("config should serialize");
        let linter = WasmMarkdownLinter::new(js_config).expect("linter should build");

        let dirty = "# Title\n\nTrailing spaces here   \n";
        let result = linter
            .lint_and_fix(dirty.into())
            .await
            .expect("lint_and_fix should succeed");

        assert_eq!(
            result.fixed_content(),
            dirty,
            "auto_fix=false なら本文を変更しないこと"
        );
        assert_eq!(result.fixes_applied(), 0, "auto_fix=false なら修正数は 0");
        let rules = rule_names(result.issues().expect("issues should serialize"));
        assert!(
            rules.iter().any(|rule| rule == "MD009"),
            "lint のみの経路でも問題は報告されること: {rules:?}"
        );
    }

    #[wasm_bindgen_test]
    fn incomplete_config_is_rejected() {
        // 部分オブジェクトを黙って既定へ落とすと auto_fix:false が反転するためエラーにする。
        let partial = serde_wasm_bindgen::to_value(&serde_json::json!({ "auto_fix": false }))
            .expect("config should serialize");
        match WasmMarkdownLinter::new(partial) {
            Ok(_) => panic!("incomplete config should be rejected"),
            Err(error) => assert_eq!(
                error.as_string().as_deref().unwrap_or("").split(':').next(),
                Some("Invalid lint config")
            ),
        }
    }

    #[wasm_bindgen_test]
    fn ai_disabled_by_default() {
        // ネイティブの AiConfig と同じく既定はオフ。
        let config = WasmConfig::default();
        assert!(!config.ai_enabled());
    }

    #[wasm_bindgen_test]
    fn set_ai_enabled_switches_state() {
        let mut config = WasmConfig::default();
        config.set_ai_enabled(true);
        assert!(config.ai_enabled());
    }

    #[wasm_bindgen_test]
    fn other_defaults_are_unchanged() {
        // lint / auto_fix の既定値は変更しない。
        let config = WasmConfig::default();
        assert!(config.lint_enabled());
        assert!(config.auto_fix());
    }

    #[wasm_bindgen_test]
    fn image_dimensions_wrapper() {
        // 1x1 PNG の最小ヘッダー。
        let mut png = b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR".to_vec();
        png.extend_from_slice(&1u32.to_be_bytes());
        png.extend_from_slice(&1u32.to_be_bytes());
        assert_eq!(super::image_dimensions_wasm(&png), Some(vec![1, 1]));
        assert_eq!(super::image_dimensions_wasm(b"not an image"), None);
    }
}
