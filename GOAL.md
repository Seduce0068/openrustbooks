# Goal

`LintConfig.config_file` / `rule_config` の rumdl 側配線を調査・判断する（ADR-0123 の未決定）。

## Scope

### 背景

`LintConfig` は `config_file: Option<PathBuf>` と
`rule_config: HashMap<String, serde_json::Value>` を持つが、
`MarkdownLinter::new()` は常に `Config::default()` を使い、
これらを読んでいない。利用者の `.markdownlint.yaml` やルール固有設定が
無視されている。

### 成果物

- `core/src/ai_markdown/lint.rs`: 変更（配線可能なら実装）
- `docs/decisions/0124-rumdl-config-wiring.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. rumdl 0.3.2 の `Config` 型とファイル読み込み API を調査する。
2. `rule_config` は安全に配線できるため実装する（MDxxx キーと JSON object のみ、已知的ルールのみ）。
3. `config_file` は rumdl の `extends` が任意パス／環境変数へ脱出できるため未対応とし、ADR に明文化する。
4. 既存の 213 テスト（native）+ wasm 3、E2E 228 項目を壊さない。

## Acceptance criteria

- 6 ゲート・E2E が成功する。

## Constraints

- 新規依存を追加しない。
- 任意パスの読み込みは fail-closed にする（ADR-0097 の方針）。
- `master` へ push する。force push はしない。
