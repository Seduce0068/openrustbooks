//! WebAssembly bindings for openrustbooks-core.
//!
//! This module exposes the core functionality to JavaScript/TypeScript
//! via wasm-bindgen for use in the browser version.

#[cfg(target_arch = "wasm32")]
use crate::{
    CoreError, CoreResult,
    ai_markdown::*,
    document::{Document, DocumentFormat, DocumentId, DocumentMetadata},
};
#[cfg(target_arch = "wasm32")]
use serde::{Deserialize, Serialize};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    fn error(s: &str);
}

#[cfg(target_arch = "wasm32")]
macro_rules! console_log {
    ($($t:tt)*) => (log(&format!($($t)*)))
}

#[cfg(target_arch = "wasm32")]
macro_rules! console_error {
    ($($t:tt)*) => (error(&format!($($t)*)))
}

/// Initialize the library for Wasm use
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn init() {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();
    console_log!("openrustbooks-core initialized");
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
    front_matter: Option<JsValue>,
    metadata: JsValue,
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
            metadata: JsValue::NULL,
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
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
#[derive(Serialize, Deserialize)]
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
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
#[derive(Serialize, Deserialize, Clone)]
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
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct WasmAiMarkdownService {
    // In reality, this would hold a reference to the actual service
    // For now, we'll use a simplified approach
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
        context: Option<String>,
    ) -> Result<WasmPipelineResult, JsValue> {
        // This would call the actual pipeline
        // For now, return a placeholder
        console_log!("Generate called with prompt: {}", prompt);
        Ok(WasmPipelineResult {
            content: format!("# Generated\n\nContent for: {}", prompt),
            diff: None,
            lint_passed: true,
            attempts: 1,
            tokens_used: 100,
        })
    }

    /// Refine existing Markdown
    #[wasm_bindgen]
    pub async fn refine(
        &self,
        content: String,
        instruction: String,
        context: Option<String>,
    ) -> Result<WasmPipelineResult, JsValue> {
        console_log!("Refine called with instruction: {}", instruction);
        Ok(WasmPipelineResult {
            content: format!("{}\n\n<!-- Refined: {} -->", content, instruction),
            diff: None,
            lint_passed: true,
            attempts: 1,
            tokens_used: 150,
        })
    }

    /// Convert to table
    #[wasm_bindgen]
    pub async fn to_table(
        &self,
        data: String,
        format: String,
        headers: Option<JsValue>,
    ) -> Result<WasmPipelineResult, JsValue> {
        console_log!("Table conversion for format: {}", format);
        Ok(WasmPipelineResult {
            content: "| Header 1 | Header 2 |\n|----------|----------|\n| Cell 1   | Cell 2   |"
                .to_string(),
            diff: None,
            lint_passed: true,
            attempts: 1,
            tokens_used: 80,
        })
    }

    /// Generate front matter
    #[wasm_bindgen]
    pub async fn front_matter(
        &self,
        content: String,
        schema: Option<JsValue>,
    ) -> Result<WasmPipelineResult, JsValue> {
        console_log!("Front matter generation");
        Ok(WasmPipelineResult {
            content: format!("---\ntitle: Generated\n---\n\n{}", content),
            diff: None,
            lint_passed: true,
            attempts: 1,
            tokens_used: 50,
        })
    }
}

/// Wasm-compatible Markdown linter
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct WasmMarkdownLinter {
    config: JsValue,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl WasmMarkdownLinter {
    #[wasm_bindgen(constructor)]
    pub fn new(config: JsValue) -> Self {
        Self { config }
    }

    #[wasm_bindgen]
    pub fn lint(&self, content: String) -> Result<JsValue, JsValue> {
        console_log!("Linting content ({} chars)", content.len());
        // Return empty issues array for now
        Ok(serde_wasm_bindgen::to_value(&Vec::<WasmLintIssue>::new()).unwrap())
    }

    #[wasm_bindgen]
    pub async fn lint_and_fix(&self, content: String) -> Result<WasmLintFixResult, JsValue> {
        console_log!("Lint and fix content ({} chars)", content.len());
        Ok(WasmLintFixResult {
            passed: true,
            issues: vec![],
            fixed_content: content,
            fixes_applied: 0,
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
    message: String,
    severity: String,
    fixable: bool,
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
    pub fn issues(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.issues).unwrap()
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
#[wasm_bindgen]
impl WasmConfig {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            ai_enabled: true,
            ai_provider: "openai".to_string(),
            ai_model: "gpt-4o-mini".to_string(),
            lint_enabled: true,
            auto_fix: true,
        }
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
#[wasm_bindgen]
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
