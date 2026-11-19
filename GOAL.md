# Goal

エラー文言の二重付与と expect() を整理する（R6 C05 / R3-error#3,#5、ADR-0229）。

## Scope

### 成果物

- `desktop/src/main.rs`: run() -> tauri::Result<()>、expect 削除、終了コード 101 維持
- `core/src/ai_markdown/lint.rs` / `client.rs`: 二重 prefix 削除
- `web/src/lib.rs`: invoke / parse エラーを固定文言化
- `docs/decisions/0229-error-message-cleanup.md`: 追加
- `todo.md`: C05 / R3-error#3,#5 を完了へ移動（status 分離は未決定）

### 決定

- Display で 1 行エラー表示、mobile は log::error!。
- JsValue Debug / 生エラーを status へ出さない。
- status の error/info 分離は ADR 未決定。

## Acceptance criteria

- E2E 254 項目、全ゲート成功（検証済み）。
