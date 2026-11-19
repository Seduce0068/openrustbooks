# Goal

config.toml の OpenAI `extra_headers` / Ollama `options` を実際のクライアントへ渡す（R6 TD-M20）。

## Scope

### 背景

`OpenAiConfig.extra_headers` / `OllamaConfig.options` は config.toml から読まれ、
DTO / Debug ではマスクされる（ADR-0102）が、クライアント生成時に無視されていた。
ADR-0102 は「config.toml の headers は AI リクエストに送信される」と
明記しており、実装と契約が一致していない。

### 成果物

- `core/src/ai_markdown/client.rs`: 変更（header / options の配線と unit test）
- `docs/decisions/0221-ai-client-options.md`: 追加
- `GOAL.md` / `todo.md`: 契約・完了反映

### 必須事項

1. OpenAI: `extra_headers` を `OpenAIConfig::with_header()` 経由で全リクエストへ付与する。
2. Ollama: `options` を API リクエストの `options` オブジェクトへマージする。
3. 不正なヘッダ名／値はエラー文言に値を出さず fail-closed にする。
4. Ollama の `num_predict` は呼び出し側の `max_tokens` を優先し、
   config 側で上書きできないようにする。
5. `model` / `stream` は config options から上書きできないようにする。
6. 既存の allowlist / secret redaction / wasm fail-closed 契約を壊さない。
7. 全ゲート成功（E2E 253 維持）。

## Acceptance criteria

- 全ゲート成功、E2E 253 項目成功。

## Constraints

- 新規依存を追加しない（`async-openai` の `with_header` を利用）。
- `extra_headers` は DTO 非公開・Debug マスクを維持（ADR-0102）。
- `master` へ push する。force push はしない。
