//! Markdown linting using rumdl (Rust markdownlint compatible).

use crate::{CoreResult, CoreError, config::LintConfig};
use rumdl::{Rule, LintOptions};
use std::path::Path;
use std::collections::HashMap;

/// Lint issue found in markdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintIssue {
    pub rule: String,
    pub rule_name: String,
    pub line: usize,
    pub column: usize,
    pub end_line: Option<usize>,
    pub end_column: Option<usize>,
    pub message: String,
    pub severity: LintSeverity,
    pub fixable: bool,
}

/// Lint severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LintSeverity {
    Error,
    Warning,
    Info,
}

/// Lint result
#[derive(Debug, Clone)]
pub struct LintResult {
    pub passed: bool,
    pub issues: Vec<LintIssue>,
    pub fixed_content: String,
    pub fixes_applied: usize,
}

/// Markdown linter wrapper around rumdl
pub struct MarkdownLinter {
    rules: Vec<Box<dyn Rule>>,
    options: LintOptions,
    config: LintConfig,
}

impl MarkdownLinter {
    /// Create a new linter with configuration
    pub fn new(config: &LintConfig) -> CoreResult<Self> {
        let mut rules: Vec<Box<dyn Rule>> = Vec::new();

        // Load all built-in rules
        let all_rules = rumdl::rules::all_rules();

        // Filter rules based on config
        for rule in all_rules {
            let rule_name = rule.name();
            let rule_id = rule_id_from_name(&rule_name);

            // Check if rule is explicitly disabled
            if config.disable_rules.iter().any(|r| r == &rule_id || r == &rule_name) {
                continue;
            }

            // Check if rule is explicitly enabled (if enable_rules is not empty)
            if !config.enable_rules.is_empty() &&
               !config.enable_rules.iter().any(|r| r == &rule_id || r == &rule_name) {
                continue;
            }

            rules.push(rule);
        }

        // Load custom config file if specified
        let options = if let Some(config_file) = &config.config_file {
            LintOptions::from_file(config_file)?
        } else {
            LintOptions::default()
        };

        Ok(Self { rules, options, config: config.clone() })
    }

    /// Lint markdown content and return issues
    pub fn lint(&self, content: &str) -> CoreResult<Vec<LintIssue>> {
        let mut issues = Vec::new();

        for rule in &self.rules {
            let rule_issues = rule.check(content, &self.options)
                .map_err(|e| CoreError::lint(format!("Rule {} failed: {}", rule.name(), e)))?;

            for issue in rule_issues {
                issues.push(LintIssue {
                    rule: rule_id_from_name(&rule.name()),
                    rule_name: rule.name().to_string(),
                    line: issue.line,
                    column: issue.column,
                    end_line: issue.end_line,
                    end_column: issue.end_column,
                    message: issue.message,
                    severity: if self.config.warnings_as_errors { LintSeverity::Error } else { LintSeverity::Warning },
                    fixable: issue.fixable,
                });
            }
        }

        // Sort by line, then column
        issues.sort_by(|a, b| a.line.cmp(&b.line).then(a.column.cmp(&b.column)));

        Ok(issues)
    }

    /// Lint and auto-fix content
    pub async fn lint_and_fix(&self, content: &str) -> CoreResult<LintResult> {
        let mut current_content = content.to_string();
        let mut all_issues = Vec::new();
        let mut fixes_applied = 0;
        let mut iterations = 0;
        const MAX_FIX_ITERATIONS: usize = 5;

        loop {
            let issues = self.lint(&current_content)?;
            let error_count = issues.iter().filter(|i| i.severity == LintSeverity::Error).count();

            all_issues.extend(issues.clone());

            if error_count == 0 || iterations >= MAX_FIX_ITERATIONS || !self.config.auto_fix {
                break;
            }

            // Apply fixes for fixable issues
            let mut fixed_content = current_content.clone();
            let mut applied_this_round = 0;

            for issue in &issues {
                if issue.fixable {
                    if let Some(rule) = self.rules.iter().find(|r| rule_id_from_name(&r.name()) == issue.rule) {
                        if let Ok(fixed) = rule.fix(&fixed_content, &self.options) {
                            fixed_content = fixed;
                            applied_this_round += 1;
                        }
                    }
                }
            }

            if applied_this_round == 0 {
                break;
            }

            fixes_applied += applied_this_round;
            current_content = fixed_content;
            iterations += 1;
        }

        let final_issues = self.lint(&current_content)?;
        let passed = final_issues.iter().all(|i| i.severity != LintSeverity::Error);

        Ok(LintResult {
            passed,
            issues: final_issues,
            fixed_content: current_content,
            fixes_applied,
        })
    }

    /// Get all available rule names
    pub fn available_rules(&self) -> Vec<String> {
        self.rules.iter().map(|r| rule_id_from_name(&r.name())).collect()
    }

    /// Get rule metadata
    pub fn rule_metadata(&self) -> Vec<RuleMetadata> {
        self.rules.iter().map(|r| RuleMetadata {
            id: rule_id_from_name(&r.name()),
            name: r.name().to_string(),
            description: r.description().to_string(),
            fixable: r.is_fixable(),
        }).collect()
    }
}

/// Rule metadata for UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleMetadata {
    pub id: String,
    pub name: String,
    pub description: String,
    pub fixable: bool,
}

/// Convert rule name to rule ID (e.g., "heading-style" -> "MD003")
fn rule_id_from_name(name: &str) -> String {
    // rumdl uses markdownlint rule names, map to MDxxx format
    // This is a simplified mapping - in reality rumdl exposes the rule ID
    match name {
        "heading-style" => "MD003",
        "heading-increment" => "MD001",
        "first-heading-h1" => "MD002",
        "ul-style" => "MD004",
        "ol-prefix" => "MD005",
        "list-indent" => "MD007",
        "no-hard-tabs" => "MD010",
        "no-multiple-blanks" => "MD012",
        "line-length" => "MD013",
        "no-trailing-spaces" => "MD009",
        "no-multiple-space-atx" => "MD019",
        "no-multiple-space-closed-atx" => "MD020",
        "no-missing-space-atx" => "MD018",
        "no-missing-space-closed-atx" => "MD021",
        "fenced-code-language" => "MD040",
        "no-duplicate-heading" => "MD024",
        "no-empty-links" => "MD042",
        "no-bare-urls" => "MD034",
        _ => {
            // Try to extract number from name like "MD013"
            if let Some(num) = name.strip_prefix("MD") {
                if num.parse::<u32>().is_ok() {
                    return name.to_string();
                }
            }
            name.to_uppercase()
        }
    }.to_string()
}

/// Lint configuration file watcher (for hot reload)
pub struct ConfigWatcher {
    #[allow(dead_code)]
    config_path: std::path::PathBuf,
    #[allow(dead_code)]
    sender: tokio::sync::broadcast::Sender<LintConfig>,
}

impl ConfigWatcher {
    pub fn new(config_path: std::path::PathBuf) -> (Self, tokio::sync::broadcast::Receiver<LintConfig>) {
        let (sender, receiver) = tokio::sync::broadcast::channel(16);
        (Self { config_path, sender }, receiver)
    }

    pub async fn watch(&self) -> CoreResult<()> {
        // Would use notify crate to watch config file
        // For now, just return Ok
        Ok(())
    }
}