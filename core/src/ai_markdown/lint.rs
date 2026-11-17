//! Markdown linting using rumdl (Rust markdownlint compatible).

use crate::{CoreError, CoreResult, config::LintConfig};
use rumdl_lib::{
    config::{Config, MarkdownFlavor},
    document_run::DocumentRun,
    fix_coordinator::FixResult,
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
    /// 自動修正が収束したか。false なら修正は採用されない
    pub converged: bool,
    /// 未収束時に検出された競合ルール名。
    /// 非空なら振動（ルール間の循環）、空なら反復上限到達など別原因を示す。
    pub conflicting_rules: Vec<String>,
}

/// Markdown linter wrapper around rumdl
pub struct MarkdownLinter {
    rules: Vec<Box<dyn Rule>>,
    /// lint と fix で共有する rumdl 設定
    config: Config,
    /// true なら Warning も不合格扱いにする
    warnings_as_errors: bool,
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
            warnings_as_errors: config.warnings_as_errors,
        })
    }

    /// 判定に使う warnings_as_errors フラグ（他モジュールと共有する）
    pub(crate) fn warnings_as_errors(&self) -> bool {
        self.warnings_as_errors
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

        let (adopted, fixes_applied, converged) =
            apply_fix_result(content, &fixed_content, &fix_result);
        // 採用する本文で再 lint し、本文と lint 結果を整合させる
        let issues = self.lint(adopted)?;
        let lint_passed = lint_passed(&issues, self.warnings_as_errors());
        // 未収束なら lint 結果に関わらず fail-closed で false とする
        let passed = converged && lint_passed;

        Ok(LintResultWrapper {
            passed,
            issues,
            fixed_content: adopted.to_string(),
            fixes_applied,
            converged,
            conflicting_rules: fix_result.conflicting_rules.clone(),
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

/// issues が lint 合格か判定する。
/// Error は常に不合格、Warning は warnings_as_errors=true のときだけ不合格、
/// Info は常に無視。issue の severity 自体は書き換えない。
pub(crate) fn lint_passed(issues: &[LintIssue], warnings_as_errors: bool) -> bool {
    issues.iter().all(|issue| match issue.severity {
        LintSeverity::Error => false,
        LintSeverity::Warning => !warnings_as_errors,
        LintSeverity::Info => true,
    })
}

/// FixResult を反映する。未収束なら修正を破棄し元本文へ戻す（fail-closed）。
/// 戻り値は（採用する本文, 適用した修正数, 収束したか）。
fn apply_fix_result<'a>(
    original: &'a str,
    fixed: &'a str,
    fix_result: &FixResult,
) -> (&'a str, usize, bool) {
    if fix_result.converged {
        (fixed, fix_result.rules_fixed, true)
    } else {
        (original, 0, false)
    }
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
    // MD033（インライン HTML）は修正不能な Warning の代表例
    const WARNING_ONLY: &str = "# Title\n\n<div>html</div>\n";

    fn linter() -> MarkdownLinter {
        MarkdownLinter::new(&LintConfig::default()).expect("linter should build")
    }

    fn linter_with_warnings_as_errors(value: bool) -> MarkdownLinter {
        let config = LintConfig {
            warnings_as_errors: value,
            ..Default::default()
        };
        MarkdownLinter::new(&config).expect("linter should build")
    }

    fn issue(severity: LintSeverity) -> LintIssue {
        LintIssue {
            rule: "MD000".to_string(),
            rule_name: "dummy".to_string(),
            line: 1,
            column: 1,
            end_line: None,
            end_column: None,
            message: "test issue".to_string(),
            severity,
            fixable: false,
        }
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

    #[test]
    fn new_retains_warnings_as_errors_flag() {
        assert!(!linter().warnings_as_errors());
        assert!(linter_with_warnings_as_errors(true).warnings_as_errors());
    }

    #[test]
    fn lint_passed_ignores_warning_by_default() {
        assert!(
            lint_passed(&[issue(LintSeverity::Warning)], false),
            "既定 false では Warning だけでも合格とすること"
        );
    }

    #[test]
    fn lint_passed_rejects_warning_when_enabled() {
        assert!(
            !lint_passed(&[issue(LintSeverity::Warning)], true),
            "true では Warning を不合格とすること"
        );
    }

    #[test]
    fn lint_passed_ignores_info_even_when_enabled() {
        assert!(lint_passed(&[issue(LintSeverity::Info)], true));
        assert!(lint_passed(&[issue(LintSeverity::Info)], false));
    }

    #[test]
    fn lint_passed_rejects_error_regardless_of_flag() {
        for warnings_as_errors in [false, true] {
            assert!(!lint_passed(
                &[issue(LintSeverity::Error)],
                warnings_as_errors
            ));
        }
    }

    #[tokio::test]
    async fn lint_and_fix_warning_fails_only_when_warnings_as_errors() {
        let lenient = linter_with_warnings_as_errors(false);
        let result = lenient
            .lint_and_fix(WARNING_ONLY)
            .await
            .expect("lint_and_fix should succeed");
        assert!(
            result
                .issues
                .iter()
                .any(|i| i.severity == LintSeverity::Warning),
            "比較対象に Warning があること"
        );
        assert!(
            result.passed,
            "既定 false なら Warning だけでは不合格にしないこと"
        );

        let strict = linter_with_warnings_as_errors(true);
        let result = strict
            .lint_and_fix(WARNING_ONLY)
            .await
            .expect("lint_and_fix should succeed");
        assert!(
            result
                .issues
                .iter()
                .any(|i| i.severity == LintSeverity::Warning),
            "severity は Warning のまま書き換えないこと: {:?}",
            result.issues
        );
        assert!(
            result
                .issues
                .iter()
                .all(|i| i.severity != LintSeverity::Error),
            "判定のために severity を Error 化しないこと"
        );
        assert!(!result.passed, "true なら Warning で不合格とすること");
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
        assert!(result.converged, "修正が収束すること");
        assert!(result.conflicting_rules.is_empty(), "競合ルールがないこと");
        assert!(
            result.issues.is_empty(),
            "修正後に問題が残らないこと: {:?}",
            result.issues
        );
    }

    // 疑似 FixResult で fail-closed の分岐を直接検証する
    fn fake_fix_result(converged: bool) -> FixResult {
        FixResult {
            rules_fixed: 3,
            iterations: 2,
            context_creations: 5,
            fixed_rule_names: std::collections::HashSet::new(),
            converged,
            conflicting_rules: if converged {
                Vec::new()
            } else {
                vec!["MD044".to_string(), "MD063".to_string()]
            },
            conflict_cycle: Vec::new(),
        }
    }

    #[test]
    fn apply_fix_result_adopts_fixes_when_converged() {
        let (content, fixes, converged) =
            apply_fix_result("before", "after", &fake_fix_result(true));

        assert_eq!(content, "after", "収束時は修正後の本文を採用すること");
        assert_eq!(fixes, 3, "収束時は修正数を引き継ぐこと");
        assert!(converged);
    }

    #[test]
    fn apply_fix_result_fails_closed_when_not_converged() {
        let (content, fixes, converged) =
            apply_fix_result("before", "after", &fake_fix_result(false));

        assert_eq!(content, "before", "未収束なら元の本文へ戻すこと");
        assert_eq!(fixes, 0, "未収束なら修正数を 0 にすること");
        assert!(!converged, "未収束を呼び出し側へ伝えること");
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
        assert!(result.converged);
        assert!(result.issues.is_empty());
    }
}
