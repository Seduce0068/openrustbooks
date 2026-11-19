# Goal

OpenAI extra_headers / Ollama options をクライアントへ配線する（R6 TD-M20、ADR-0221）。

## Scope

### 成果物

- `core/src/ai_markdown/client.rs`: with_header / 予約ヘッダ拒否 / Ollama options マージ、unit 7 件
- `docs/decisions/0221-ai-client-options.md`: 追加
- `todo.md`: TD-M20 を完了へ移動

### 決定

- extra_headers は固定文言で fail-closed、予約ヘッダは拒否。
- Ollama options は num_predict / model / stream を上書き不可でマージ。

## Acceptance criteria

- core 267 tests / E2E 253、全ゲート成功（検証済み）。
