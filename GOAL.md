# Goal

ストリーミング API の未接続を設計受容とする（R6 TD-M18、ADR-0216）。

## Scope

### 成果物

- `web/src/lib.rs`: streaming 表示を「未配線」へ、SettingsAiView.streaming 削除
- `core/src/ai_markdown/pipeline.rs`: 疑似ストリームを明記
- `docs/decisions/0216-streaming-deferred.md`: 追加
- `todo.md`: TD-M18 を完了へ移動

### 決定

- 本番接続は行わず、設定表示を正直化。
- lint retry / IPC / 完了判定 / tokens_used / timeout / 設定配線は ADR 未決定。

## Acceptance criteria

- E2E 253 項目、全ゲート成功（検証済み）。
