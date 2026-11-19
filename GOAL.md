# Goal

並行設定更新の lost update を revision CAS で検出する（R6 TD-L10、ADR-0210）。

## Scope

### 成果物

- `core/src/error.rs`: Conflict
- `desktop/src/state/mod.rs`: ConfigState / revision / pending / CAS commit
- `desktop/src/commands/config.rs`: 6 コマンドの expected 伝播、ConfigView revision
- `docs/decisions/0210-config-revision-cas.md`: 追加
- `todo.md`: TD-L10 を完了へ移動

### 決定

- expected は await 前に取得、commit の short write guard で比較。
- pending 予約で二重 CAS を防止。
- 保存成功時のみ revision 前進、 rollback/no-op/Conflict は不変。

## Acceptance criteria

- core 261 / desktop 79 / E2E 251、全ゲート成功（検証済み）。
