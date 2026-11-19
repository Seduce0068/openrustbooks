# Goal

文書削除時の unsaved_changes 残留を解消する（R6 TD-L11、ADR-0208）。

## Scope

### 成果物

- `desktop/src/state/mod.rs`: remove_document で unsaved キーも削除、unit 2 件
- `docs/decisions/0208-unsaved-changes-cleanup.md`: 追加
- `todo.md`: TD-L11 を完了へ移動

### 決定

- documents / unsaved_changes のロックは順番に取得・解放（同時保持なし）。
- 文書が無い場合も孤立フラグを削除。
- force/non-force 契約は不変、非原子窓と non-force 孤立は ADR 未決定。

## Acceptance criteria

- desktop 74 tests / E2E 251、全ゲート成功（検証済み）。
