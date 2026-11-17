# Goal

AI リトライ回数を安全な範囲にクランプする（ADR-0105 / 0118 の未決定）。

## Scope

### 背景

`AiMarkdownService::new()` は `config.ai.max_retries` をそのまま
`PipelineConfig` に渡す。大きすぎる値は API 呼び出しを最大 N+1 回まで
増やし（コスト制御 REQ-010 と緊張）、`usize::MAX` では
`max_retries + 1`（ADR-0118）がオーバーフローする。

### 成果物

- `core/src/ai_markdown/pipeline.rs`: 変更（クランプ関数と適用）
- `docs/decisions/0120-retry-limit.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. `max_retries` を 0..=10 にクランプする（既定 3 は維持）。
2. クランプは `AiMarkdownService::new()` で行い、config.toml / IPC の
   両方の経路に効かせる。
3. 0 は許可し（初期1回のみ）、10 を超える値は 10 に丸める。
4. テストを追加する（0 / 3 / 5 / 11 / usize::MAX）。
5. 既存の 195 テスト + wasm 3 件、E2E 228 項目を壊さない。

## Acceptance criteria

- 6 ゲート・E2E が成功する。

## Constraints

- 新規依存を追加しない。
- リトライ回数の意味（`0..=max_retries` = 初期1回＋リトライN回）は変更しない。
- `master` へ push する。force push はしない。
