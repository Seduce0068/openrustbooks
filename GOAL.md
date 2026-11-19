# Goal

ストリーミング API の未接続を設計受容とし、設定表示を正直化する（R6 TD-M18）。

## Scope

### 背景

`AiClient::stream_complete()` / `TokenStream::next_token()` は実装済みだが
本番呼び出しは 0 件。`complete()` は `_streaming` を無視して常に
`stream: false` でリクエストし、`stream_generate()` は
`generate()` の全文を 1 回だけ `on_token` へ渡す疑似ストリームである。
一方 UI は `config.ai.streaming = true` を「有効」と表示しており、
実態と一致しない。wasm の AI service は fail-closed で配信先も無い。

### 成果物

- `web/src/lib.rs`: 変更（`streaming` 表示を「未配線」と明記）
- `core/src/ai_markdown/pipeline.rs`: 変更（`stream_generate` を疑似ストリームと明記）
- `docs/decisions/0216-streaming-deferred.md`: 追加
- `GOAL.md` / `todo.md`: 契約・完了反映

### 必須事項

1. UI で `streaming` が true / false に関わらず「未配線」と表示する
   （値は denoising のため内部で保持してよいが、有効と誤認させない）。
2. `stream_generate()` のコメントを「疑似ストリーム（全文を 1 回通知）」と明記する。
3. ADR に設計受容の根拠と未決定（retry 表示、IPC、完了判定、tokens_used、timeout、設定配線）を記載する。
4. 既存 E2E 253 を壊さない。
5. 全ゲート成功。

## Acceptance criteria

- 全ゲート成功、E2E 253 項目成功。

## Constraints

- ストリーミング本番接続は行わない（UI / IPC 設計が未定）。
- 既存の AI 生成・lint retry 契約を変更しない。
- `master` へ push する。force push はしない。
