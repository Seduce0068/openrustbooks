//! Markdown linting using rumdl (Rust markdownlint compatible).

use crate::{CoreError, CoreResult, config::LintConfig};
use rumdl_lib::{
    config::{Config, MarkdownFlavor},
    document_run::DocumentRun,
    lint,
    rule::Rule,
    rule::{LintError, LintWarning, Severity},
};
use serde::{Deserialize, Serialize};

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
#[derive(Debug, Clone, Serialize)]
pub struct LintResultWrapper {
    pub passed: bool,
    pub issues: Vec<LintIssue>,
    pub fixed_content: String,
    pub fixes_applied: usize,
}

/// Markdown linter wrapper around rumdl
pub struct MarkdownLinter {
    rules: Vec<Box<dyn Rule>>,
    /// lint と fix で共有する rumdl 設定
    config: Config,
}

impl MarkdownLinter {
    /// Create a new linter with configuration
    pub fn new(config: &LintConfig) -> CoreResult<Self> {
        let mut rules: Vec<Box<dyn Rule>> = Vec::new();

        // Load all built-in rules - need a default config
        let rumdl_config = Config::default();
        let all_rules = rumdl_lib::rules::all_rules(&rumdl_config);

        // Filter rules based on config
        for rule in all_rules {
            let rule_name = rule.name();
            let rule_id = rule_id_from_name(rule_name);

            // Check if rule is explicitly disabled
            if config
                .disable_rules
                .iter()
                .any(|r| r == &rule_id || r == rule_name)
            {
                continue;
            }

            // Check if rule is explicitly enabled (if enable_rules is not empty)
            if !config.enable_rules.is_empty()
                && !config
                    .enable_rules
                    .iter()
                    .any(|r| r == &rule_id || r == rule_name)
            {
                continue;
            }

            rules.push(rule);
        }

        Ok(Self {
            rules,
            config: rumdl_config,
        })
    }

    /// Lint markdown content and return issues
    pub fn lint(&self, content: &str) -> CoreResult<Vec<LintIssue>> {
        let flavor = MarkdownFlavor::Standard;
        let result: Result<Vec<LintWarning>, LintError> = lint(
            content,
            &self.rules,
            false,
            flavor,
            None,
            Some(&self.config),
        );

        let warnings = result.map_err(|e| CoreError::lint(format!("Lint error: {:?}", e)))?;

        let mut issues = Vec::new();
        for warning in warnings {
            issues.push(LintIssue {
                rule: rule_id_from_name(warning.rule_name.as_deref().unwrap_or("")),
                rule_name: warning.rule_name.unwrap_or_default(),
                line: warning.line,
                column: warning.column,
                end_line: Some(warning.end_line),
                end_column: Some(warning.end_column),
                message: warning.message,
                severity: match warning.severity {
                    Severity::Error => LintSeverity::Error,
                    Severity::Warning => LintSeverity::Warning,
                    Severity::Info => LintSeverity::Info,
                },
                fixable: warning.fix.is_some(),
            });
        }

        // Sort by line, then column
        issues.sort_by(|a, b| a.line.cmp(&b.line).then(a.column.cmp(&b.column)));

        Ok(issues)
    }

    /// Lint して自動修正し、修正後の本文で再 lint する
    pub async fn lint_and_fix(&self, content: &str) -> CoreResult<LintResultWrapper> {
        // 反復上限は rumdl の wasm 実装に倣う
        let (fixed_content, fix_result) = DocumentRun::new(content, &self.rules, &self.config)
            .fix(10)
            .map_err(|e| CoreError::validation(format!("Auto-fix error: {}", e)))?;

        // 修正前の issues のままだと retry 判定が誤るため再 lint する
        let issues = self.lint(&fixed_content)?;
        let passed = issues.iter().all(|i| i.severity != LintSeverity::Error);

        Ok(LintResultWrapper {
            passed,
            issues,
            fixed_content,
            fixes_applied: fix_result.rules_fixed,
        })
    }

    /// Get all available rule names
    pub fn available_rules(&self) -> Vec<String> {
        self.rules
            .iter()
            .map(|r| rule_id_from_name(r.name()))
            .collect()
    }

    /// Get rule metadata
    pub fn rule_metadata(&self) -> Vec<RuleMetadata> {
        self.rules
            .iter()
            .map(|r| RuleMetadata {
                id: rule_id_from_name(r.name()),
                name: r.name().to_string(),
                description: r.description().to_string(),
                fixable: false, // rumdl Rule trait doesn't expose is_fixable directly
            })
            .collect()
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
    match name {
        "heading-style" => "MD003".to_string(),
        "heading-increment" => "MD001".to_string(),
        "first-heading-h1" => "MD002".to_string(),
        "ul-style" => "MD004".to_string(),
        "ol-prefix" => "MD005".to_string(),
        "list-indent" => "MD007".to_string(),
        "no-hard-tabs" => "MD010".to_string(),
        "no-multiple-blanks" => "MD012".to_string(),
        "line-length" => "MD013".to_string(),
        "no-trailing-spaces" => "MD009".to_string(),
        "no-multiple-space-atx" => "MD019".to_string(),
        "no-multiple-space-closed-atx" => "MD020".to_string(),
        "no-missing-space-atx" => "MD018".to_string(),
        "no-missing-space-closed-atx" => "MD021".to_string(),
        "fenced-code-language" => "MD040".to_string(),
        "no-duplicate-heading" => "MD024".to_string(),
        "no-empty-links" => "MD042".to_string(),
        "no-bare-urls" => "MD034".to_string(),
        _ => {
            if let Some(num) = name.strip_prefix("MD")
                && num.parse::<u32>().is_ok()
            {
                return name.to_string();
            }
            name.to_uppercase()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 末尾空白（MD009）は修正可能な代表例
    const DIRTY: &str = "# Title\n\nTrailing spaces here   \n";

    fn linter() -> MarkdownLinter {
        MarkdownLinter::new(&LintConfig::default()).expect("linter should build")
    }

    #[test]
    fn lint_detects_fixable_issue_without_changing_content() {
        let linter = linter();
        let issues = linter.lint(DIRTY).expect("lint should succeed");

        assert!(
            issues.iter().any(|i| i.rule == "MD009"),
            "末尾空白が MD009 として検出されること"
        );
    }

    #[tokio::test]
    async fn lint_and_fix_applies_fixes_and_relints() {
        let linter = linter();
        let result = linter
            .lint_and_fix(DIRTY)
            .await
            .expect("lint_and_fix should succeed");

        assert!(result.fixes_applied > 0, "修正ルールが 1 以上あること");
        assert_ne!(result.fixed_content, DIRTY, "本文が変更されること");
        assert!(result.passed, "修正後に lint が通ること");
        assert!(
            result.issues.is_empty(),
            "修正後に問題が残らないこと: {:?}",
            result.issues
        );
    }

    #[tokio::test]
    async fn lint_and_fix_is_noop_for_clean_content() {
        let linter = linter();
        let clean = "# Title\n\nTrailing spaces here\n";
        let result = linter
            .lint_and_fix(clean)
            .await
            .expect("lint_and_fix should succeed");

        assert_eq!(result.fixes_applied, 0);
        assert_eq!(result.fixed_content, clean);
        assert!(result.passed);
        assert!(result.issues.is_empty());
    }
}
