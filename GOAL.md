# Goal

`execute_pipeline()` の到達不能なデッドコードを削除する（ADR-0116 の未決定）。

## Scope

### 背景

`execute_pipeline()` は `lint_result.passed || attempt == max_retries` で
必ず `Ok` を返すため、ループ外の `last_error` / `Err(...)` は到達不能。
`last_error` の代入も無意味。ADR-0116 でデッドコードの削除が未決定だった。

### 成果物

- `core/src/ai_markdown/mod.rs`: 変更（`last_error` と到達不能な Err を削除）
- `docs/decisions/0118-pipeline-dead-code.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. `let mut last_error = None;` とループ内の代入、ループ後の `Err(...)` を削除する。
2. 最終試行で `lint_passed=false` のとき `Ok` を返す挙動（ADR-0116）は変更しない。
3. `call_ai()` のエラーは `?` で従来どおり伝播する。
4. 既存の 195 テスト、E2E 228 項目を壊さない。

## Acceptance criteria

- 6 ゲート・E2E が成功する。

## Constraints

- 新規依存を追加しない。
- リトライ回数・差分・lint フラグの挙動は変更しない。
- `master` へ push する。force push はしない。
