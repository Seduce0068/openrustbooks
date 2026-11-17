# Goal

AI リトライ回数をトークン上限から誤導出する問題を修正する（todo.md R1-ai#3 / R2-ai#3）。

## Scope

### 背景

`core/src/ai_markdown/pipeline.rs:27` が
`max_retries: config.ai.max_tokens_per_request.min(3)` としている。
トークン上限（8192 等）をリトライ回数（最大 3）に流用しており、
設定を変更するとリトライ回数が意図せず変わる。
要件 REQ-010 は「rumdl 検査合格まで最大 3 回再生成」を求めている。

### 成果物

- `core/src/config.rs`: 変更（`AiConfig.max_retries` を追加、既定 3）
- `core/src/ai_markdown/pipeline.rs`: 変更（専用設定を使用）
- `desktop/src/commands/config.rs`: 変更（DTO に含める）
- `docs/decisions/0105-retry-config.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. `AiConfig` に `max_retries: usize` を追加し、既定値を 3 にする。
2. `PipelineConfig.max_retries` は `config.ai.max_retries` を使う。
3. DTO（`AiConfigView`）に `max_retries` を含め、JS から設定可能にする。
   `merge_secrets` の既存ロジックを壊さない。
4. 既存 TOML に `max_retries` が無い場合も既定 3 で読み込める（後方互換）。
5. テストを追加する。
6. 既存の 169 テスト、E2E 228 項目を壊さない。

## Acceptance criteria

- 6 ゲート・E2E が成功する。

## Constraints

- 新規依存を追加しない。
- 同意ダイアログ・送信先制限は変更しない。
- `master` へ push する。force push はしない。
