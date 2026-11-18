# Goal

Wasm AI / lint スタブの偽成功を排除し、未実装を明示する（ADR-0151 の高優先度残タスク）。

## Scope

### 背景

`core/src/wasm.rs` の `WasmAiMarkdownService` は固定文字列と
`lint_passed: true`、架空の `tokens_used` を返し、
`WasmMarkdownLinter` は常に空 issues / `passed: true` を返す。
Web UI からは未参照だが、将来接続された際に誤った成功結果を返す危険がある。

### 成果物

- `core/src/wasm.rs`: 変更（stub を fail-closed な `Err(JsValue)` に変更、または real lint 接続）
- `docs/decisions/0156-wasm-stub-failclosed.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. AI メソッド（`generate` / `refine` / `to_table` / `front_matter`）が
   「ブラウザでは AI 機能を利用できません」系の明示的エラーを返す。
2. `WasmMarkdownLinter::lint()` / `lint_and_fix()` も同様に fail-closed にするか、
   rumdl を利用して実 lint を返す（実装可能ならこちらを優先）。
3. 架空の `tokens_used` / `lint_passed: true` を返さない。
4. 既存の 254 単体テスト + wasm 4、E2E 233 項目を壊さない。

## Acceptance criteria

- 全ゲート成功、E2E 233 項目成功。
- ADR に判断と根拠が記録される。

## Constraints

- 新規依存を追加しない。
- ブラウザ側 AI 送信（同意・API key）は本次で実装しない。
- `master` へ push する。force push はしない。
