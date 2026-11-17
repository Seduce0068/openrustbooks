//! Prompt template registry and rendering.

use crate::{CoreError, CoreResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;
use std::sync::OnceLock;

/// Prompt types for different tasks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PromptType {
    Generate,
    Refine,
    Table,
    FrontMatter,
    Summarize,
    Translate,
    ExplainCode,
}

/// Variables for prompt templates
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PromptVariables {
    pub user_prompt: String,
    pub context: Option<String>,
    pub current_content: String,
    pub attempt: usize,
    pub template: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Prompt template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    pub id: String,
    pub prompt_type: PromptType,
    pub version: String,
    pub system_prompt: String,
    pub user_prompt_template: String,
    pub description: String,
    pub tags: Vec<String>,
}

/// Prompt registry for managing templates
pub struct PromptRegistry {
    templates: HashMap<PromptType, PromptTemplate>,
    custom_templates: HashMap<String, PromptTemplate>,
}

impl PromptRegistry {
    /// Create a registry with the built-in templates registered.
    ///
    /// Portable across native and Wasm targets: the built-in templates are
    /// embedded via `include_str!` and need no filesystem access.
    pub fn with_builtins() -> CoreResult<Self> {
        let mut registry = Self {
            templates: HashMap::new(),
            custom_templates: HashMap::new(),
        };

        // Register built-in templates
        registry.register_builtin(
            PromptType::Generate,
            include_str!("../../prompts/generate.hbs"),
        )?;
        registry.register_builtin(PromptType::Refine, include_str!("../../prompts/refine.hbs"))?;
        registry.register_builtin(PromptType::Table, include_str!("../../prompts/table.hbs"))?;
        registry.register_builtin(
            PromptType::FrontMatter,
            include_str!("../../prompts/frontmatter.hbs"),
        )?;
        registry.register_builtin(
            PromptType::Summarize,
            include_str!("../../prompts/summarize.hbs"),
        )?;
        registry.register_builtin(
            PromptType::Translate,
            include_str!("../../prompts/translate.hbs"),
        )?;
        registry.register_builtin(
            PromptType::ExplainCode,
            include_str!("../../prompts/explain_code.hbs"),
        )?;

        Ok(registry)
    }

    /// Load built-in and custom templates from the user's config directory.
    ///
    /// Native only: scanning the `prompts/` directory requires filesystem
    /// access. Wasm callers should use [`PromptRegistry::with_builtins`].
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load() -> CoreResult<Self> {
        let mut registry = Self::with_builtins()?;

        // Try to load custom templates from prompts directory
        if let Ok(custom_dir) = Self::custom_prompts_dir() {
            registry.load_custom_from_dir(&custom_dir)?;
        }

        Ok(registry)
    }

    /// Get the custom prompts directory
    #[cfg(not(target_arch = "wasm32"))]
    fn custom_prompts_dir() -> CoreResult<PathBuf> {
        use directories::ProjectDirs;
        if let Some(proj_dirs) = ProjectDirs::from("com", "onoderakeita", "openrustbooks") {
            Ok(proj_dirs.config_dir().join("prompts"))
        } else {
            Err(CoreError::internal("Could not determine config directory"))
        }
    }

    /// Load custom templates from a directory
    #[cfg(not(target_arch = "wasm32"))]
    fn load_custom_from_dir(&mut self, dir: &PathBuf) -> CoreResult<()> {
        if !dir.exists() {
            return Ok(());
        }

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("hbs") {
                let content = std::fs::read_to_string(&path)?;
                if let Ok(template) = toml::from_str::<PromptTemplate>(&content) {
                    self.custom_templates.insert(template.id.clone(), template);
                }
            }
        }
        Ok(())
    }

    /// Register a built-in template
    fn register_builtin(&mut self, prompt_type: PromptType, content: &str) -> CoreResult<()> {
        let template = self.parse_template(prompt_type, content)?;
        self.templates.insert(prompt_type, template);
        Ok(())
    }

    /// Parse template from string (supports TOML front matter + Handlebars body)
    fn parse_template(&self, prompt_type: PromptType, content: &str) -> CoreResult<PromptTemplate> {
        // Try to parse as TOML with front matter
        if content.starts_with("---") {
            let lines: Vec<&str> = content.lines().collect();
            for (i, line) in lines.iter().enumerate().skip(1) {
                if *line == "---" {
                    let front_matter = lines[1..i].join("\n");
                    let body = lines[i + 1..].join("\n");
                    let mut template: PromptTemplate = toml::from_str(&front_matter)?;
                    template.user_prompt_template = body;
                    template.prompt_type = prompt_type;
                    return Ok(template);
                }
            }
        }

        // Fallback: simple template with defaults
        Ok(PromptTemplate {
            id: format!("builtin-{:?}", prompt_type).to_lowercase(),
            prompt_type,
            version: "1.0.0".to_string(),
            system_prompt: Self::default_system_prompt(prompt_type).to_string(),
            user_prompt_template: content.to_string(),
            description: format!("Built-in {:?} template", prompt_type),
            tags: vec!["builtin".to_string()],
        })
    }

    /// Default system prompts for each type
    fn default_system_prompt(prompt_type: PromptType) -> &'static str {
        match prompt_type {
            PromptType::Generate => {
                "You are a Markdown expert. Generate clean, well-structured Markdown that passes markdownlint validation. Use proper heading hierarchy, consistent list markers, fenced code blocks with languages, and valid links."
            }
            PromptType::Refine => {
                "You are a Markdown editor. Improve the given Markdown according to the user's instructions while maintaining valid markdownlint compliance. Preserve the original structure unless explicitly asked to change it."
            }
            PromptType::Table => {
                "You are a data-to-Markdown converter. Convert the provided data into a well-formatted GitHub Flavored Markdown table with proper alignment and escaping."
            }
            PromptType::FrontMatter => {
                "You are a document metadata expert. Generate appropriate YAML front matter for the given document content. Include title, description, tags, date, and any other relevant metadata."
            }
            PromptType::Summarize => {
                "You are a technical writer. Create a concise Markdown summary of the provided content with key points as a bullet list."
            }
            PromptType::Translate => {
                "You are a translator. Translate the Markdown content to the target language while preserving all Markdown formatting, code blocks, and structure."
            }
            PromptType::ExplainCode => {
                "You are a code educator. Explain the provided code in clear Markdown with headings, code blocks, and step-by-step breakdowns."
            }
        }
    }

    /// Render a prompt for the given type and variables
    pub fn render(
        &self,
        prompt_type: PromptType,
        variables: PromptVariables,
    ) -> CoreResult<String> {
        let template = self
            .templates
            .get(&prompt_type)
            .or_else(|| {
                self.custom_templates
                    .values()
                    .find(|t| t.prompt_type == prompt_type)
            })
            .ok_or_else(|| CoreError::ai_pipeline(format!("No template for {:?}", prompt_type)))?;

        // Simple template rendering (replace {{variable}} patterns)
        let mut result = template.user_prompt_template.clone();

        // Replace standard variables
        result = result.replace("{{user_prompt}}", &variables.user_prompt);
        result = result.replace("{{current_content}}", &variables.current_content);
        result = result.replace("{{attempt}}", &variables.attempt.to_string());

        if let Some(ctx) = &variables.context {
            result = result.replace("{{context}}", ctx);
        } else {
            result = result.replace("{{context}}", "");
        }

        if let Some(tmpl) = &variables.template {
            result = result.replace("{{template}}", tmpl);
        } else {
            result = result.replace("{{template}}", "");
        }

        // Replace extra variables
        for (key, value) in &variables.extra {
            let json_str = serde_json::to_string(value).unwrap_or_default();
            result = result.replace(&format!("{{{{{}}}}}", key), &json_str);
        }

        // Prepend system prompt
        let full_prompt = format!("{}\n\n---\n\n{}", template.system_prompt, result);

        Ok(full_prompt)
    }

    /// Get a template by type
    pub fn get_template(&self, prompt_type: PromptType) -> Option<&PromptTemplate> {
        self.templates.get(&prompt_type).or_else(|| {
            self.custom_templates
                .values()
                .find(|t| t.prompt_type == prompt_type)
        })
    }

    /// List all available templates
    pub fn list_templates(&self) -> Vec<&PromptTemplate> {
        self.templates
            .values()
            .chain(self.custom_templates.values())
            .collect()
    }

    /// Add a custom template
    pub fn add_custom(&mut self, template: PromptTemplate) {
        self.custom_templates.insert(template.id.clone(), template);
    }
}

/// Global registry instance
static REGISTRY: OnceLock<PromptRegistry> = OnceLock::new();

/// Get the global prompt registry
pub fn global_registry() -> &'static PromptRegistry {
    REGISTRY.get_or_init(|| {
        #[cfg(not(target_arch = "wasm32"))]
        let loaded = PromptRegistry::load();
        #[cfg(target_arch = "wasm32")]
        let loaded = PromptRegistry::with_builtins();
        loaded.expect("Failed to load prompt registry")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL_TYPES: [PromptType; 7] = [
        PromptType::Generate,
        PromptType::Refine,
        PromptType::Table,
        PromptType::FrontMatter,
        PromptType::Summarize,
        PromptType::Translate,
        PromptType::ExplainCode,
    ];

    #[test]
    fn builtins_register_all_prompt_types() {
        let registry = PromptRegistry::with_builtins().expect("組み込みテンプレートは読み込める");
        for prompt_type in ALL_TYPES {
            assert!(
                registry.get_template(prompt_type).is_some(),
                "{prompt_type:?} が登録されていない"
            );
        }
        assert_eq!(registry.list_templates().len(), ALL_TYPES.len());
    }

    #[test]
    fn builtin_system_prompts_are_not_empty() {
        let registry = PromptRegistry::with_builtins().expect("組み込みテンプレートは読み込める");
        for prompt_type in ALL_TYPES {
            let template = registry
                .get_template(prompt_type)
                .expect("テンプレートは登録済み");
            assert!(
                !template.system_prompt.trim().is_empty(),
                "{prompt_type:?} の system_prompt が空"
            );
        }
    }

    #[test]
    fn render_starts_with_system_prompt() {
        let registry = PromptRegistry::with_builtins().expect("組み込みテンプレートは読み込める");
        for prompt_type in ALL_TYPES {
            let rendered = registry
                .render(prompt_type, PromptVariables::default())
                .expect("描画できる");
            let system_prompt = &registry
                .get_template(prompt_type)
                .expect("テンプレートは登録済み")
                .system_prompt;
            assert!(
                rendered.starts_with(system_prompt),
                "{prompt_type:?} の出力先頭に system_prompt がない"
            );
            assert_eq!(
                rendered.matches(system_prompt).count(),
                1,
                "{prompt_type:?} の system_prompt が重複している"
            );
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test]
    async fn service_new_succeeds_with_ai_enabled() {
        use crate::ai_markdown::AiMarkdownService;
        use crate::config::AppConfig;

        let mut config = AppConfig::default();
        config.ai.enabled = true;
        // Ollama は初期化時に HTTP 要求を行わず、ローカル設定だけで生成できる。
        config.ai.default_provider = "ollama".to_string();
        AiMarkdownService::new(&config)
            .await
            .expect("AI 有効設定でサービスを初期化できる");
    }
}
