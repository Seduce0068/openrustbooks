# Goal

auto_fix 分岐の native / wasm 重複を共通関数へ集約する（R6 PR-M09）。

## Scope

### 背景

`core/src/ai_markdown/mod.rs::validate_content()` と
`core/src/wasm.rs::WasmMarkdownLinter::lint_and_fix()` が
「auto_fix=true なら lint_and_fix、false なら lint + LintResultWrapper」という
同じ分岐を各自実装している。片方だけ修正されると native と browser で
lint 判定が食い違う。

### 成果物

- `core/src/ai_markdown/lint.rs`: 変更（pure な共通関数を追加）
- `core/src/ai_markdown/mod.rs`: 変更（共通関数を利用）
- `core/src/wasm.rs`: 変更（共通関数を利用）
- `docs/decisions/0194-lint-validation-unification.md`: 追加
- `GOAL.md` / `todo.md`: 契約・完了反映

### 必須事項

1. `lint.rs` に `pub async fn validate_with_auto_fix(linter: &MarkdownLinter, content: &str, auto_fix: bool) -> CoreResult<LintResultWrapper>` を追加する。
2. native `validate_content()` は薄いラッパーへ変更する。
3. wasm `lint_and_fix()` は共通関数へ委譲し、重複コードを削除する。
4. 既存の unit / wasm テストを壊さない（LintResultWrapper の converged / conflicting_rules 契約を維持）。
5. 全ゲート成功（E2E 250 維持）。

## Acceptance criteria

- 全ゲート成功、E2E 250 項目成功。

## Constraints

- 新規依存を追加しない。
- `LintResultWrapper` の形・意味は変更しない。
- `unsafe` 禁止。
- `master` へ push する。force push はしない。
