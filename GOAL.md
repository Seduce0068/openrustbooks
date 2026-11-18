# Goal

`max_retries = 0` のとき初期 1 回のみ実行され `attempts = 1` になることを回帰テストで固定する（ADR-0118 / 0120 の未決定）。

## Scope

### 背景

`execute_pipeline()` は `for attempt in 0..=max_retries` で、
`max_retries = 0` なら `0..=0` の 1 回だけ実行され `attempts = 1` になる。
これまで専用テストが無く、ADR-0118 / 0120 で未決定として残っている。

### 成果物

- `core/src/ai_markdown/mod.rs`: 変更（テスト追加）
- `docs/decisions/0133-zero-retry-regression.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. `max_retries = 0` の PipelineConfig で `execute_pipeline()` を呼び、
   `attempts == 1` になることを検証する。
2. モック AiClient / 既存のテストヘルパーを再利用し、新規依存を追加しない。
3. 既存の 232 テスト + wasm 3、E2E 229 項目を壊さない。

## Acceptance criteria

- 6 ゲート + audit + wasm-test + E2E が成功する。

## Constraints

- 本番コードは変更しない（テストのみ）。
- `master` へ push する。force push はしない。
