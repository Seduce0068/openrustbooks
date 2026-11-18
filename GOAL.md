# Goal

Ollama ストリームの行境界とタイムアウトを修正する（R6 PR-M01 / TD-M04、ADR-0189）。

## Scope

### 成果物

- `core/src/ai_markdown/client.rs`: pending / responses キュー、parse_line fail-closed、timeout 配線、unit 14 件
- `docs/decisions/0189-ollama-stream-boundary.md`: 追加
- `todo.md`: PR-M01 / TD-M04 を完了へ移動

### 決定

- バイト累積＋行キューで分断行／複数行／UTF-8 分割を解消。
- error / 不正 JSON / 不正 UTF-8 / 1 MiB 超過は明示エラー。
- timeout は AiConfig.timeout_secs を create_client から渡す。
- is_done はキュー消化後のみ true。

## Acceptance criteria

- core 241 / E2E 249、全ゲート成功（検証済み）。
