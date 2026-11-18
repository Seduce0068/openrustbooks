# Goal

未実装の watch を利用可能と報告しないようにする（R6 TD-H04、ADR-0178）。

## Scope

### 成果物

- `core/src/io.rs`: can_watch=false、unit 2 件
- `desktop/src/commands/fs.rs`: watch_directory 関数削除
- `desktop/src/main.rs`: invoke_handler から削除
- `docs/decisions/0178-watch-capability-honesty.md`: 追加
- `todo.md`: TD-H04 を完了へ移動

### 決定

- capability と IPC 表面を実装事実へ一致させる。
- 型・トレイトは将来実装のため残す。
- 新規依存を追加しない。

## Acceptance criteria

- E2E 247 項目、全ゲート成功（検証済み）。
