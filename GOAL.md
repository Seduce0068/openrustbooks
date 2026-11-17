# Goal

`LintConfig.warnings_as_errors` を lint 判定に配線する（ADR-0110 / 0116 の未決定）。

## Scope

### 背景

`LintConfig.warnings_as_errors`（既定 false）は設定・DTO に存在するが、
`MarkdownLinter` の判定（`passed = Error severity のみ`）で使われていない。
利用者が true に設定しても Warning がエラー扱いにならない。

### 成果物

- `core/src/ai_markdown/lint.rs`: 変更（フラグ保持と判定への反映）
- `docs/decisions/0123-warnings-as-errors.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. `MarkdownLinter` が `warnings_as_errors` を保持する。
2. true のときは Warning severity も不合格扱いにする（Info は従来どおり無視）。
3. `lint()` と `lint_and_fix()` の両方に効かせる。
4. 既定 false の挙動は不変（既存テストが壊れない）。
5. テストを追加する。
6. 既存の 203 テスト + wasm 3 件、E2E 228 項目を壊さない。

## Acceptance criteria

- 6 ゲート・E2E が成功する。

## Constraints

- 新規依存を追加しない。
- `config_file` / `rule_config` の配線は今回行わない（別タスク）。
- `master` へ push する。force push はしない。
