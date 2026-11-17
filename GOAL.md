# Goal

設定 IPC の DTO からローカルパス情報を除外する（ipc-auditor の指摘、ADR-0102 の未決定）。

## Scope

### 背景

`ConfigView` は `GeneralConfig`（`data_dir` / `cache_dir`）、`LintConfig`
（`config_file` / `rule_config`）、`StorageConfigView`（`local_roots[].path` /
`recent_paths`）、`OllamaConfig`（`options`）、`AiConfigView`（`model_overrides`）
を丸ごと JS へ返す。ユーザー名や最近開いた文書名が露出する。

### 成果物

- `desktop/src/commands/config.rs`: 変更（DTO の VIEW 型追加とマージ保持）
- `docs/decisions/0103-path-field-redaction.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. 次の DTO 用 VIEW 型を追加し、パス／汎用マップを返さないようにする。
   - `GeneralConfigView`: `data_dir` / `cache_dir` を除外（もしくは file_name のみ）。
   - `LintConfigView`: `config_file` / `rule_config` を除外。
   - `StorageConfigView`: `local_roots[].path` を除外（`id` / `name` / `read_only` / `watch` は残す）、`recent_paths` を除外。
   - `OllamaConfigView`: `options` を除外。
   - `AiConfigView`: `model_overrides` を除外。
2. `merge_secrets` を拡張し、除外したフィールドは現在値を保持する
   （GUI 往復で消えないように）。
3. 既存の 161 テスト、E2E 228 項目を壊さない。
4. 新規テストを追加する（DTO JSON にパス情報が無いこと、マージ保持）。

## Acceptance criteria

- 6 ゲート・E2E が成功する。

## Constraints

- 新規依存を追加しない。
- UI の配線は変更しない（未接続のまま）。
- `master` へ push する。force push はしない。
