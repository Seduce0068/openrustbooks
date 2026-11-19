# Goal

容量超過時の読書位置削除を通知した（R6 R3-config#5、ADR-0245）。

## Scope

### 成果物

- `web/src/lib.rs`: thread_local ペイロードと専用 status/console 文言
- `tools/verify-web-reader.py`: E2E 1 件、254 → 255
- `docs/e2e-traceability.md`: 255 件追従、VC-006 へ割当
- `docs/decisions/0245-position-prune-notification.md`: 追加
- `todo.md` / README / 各索引 / log: 更新

### 決定

- 削除件数・残件数・成否のみを数値で通知、機微情報は出さない。
- 通常 prune / editor イベント分離は不変、退避は未決定。

## Acceptance criteria

- E2E 255 項目、全ゲート成功（検証済み）。
