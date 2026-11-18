# Goal

Reader アンマウント時に Blob URL を解放する（R6 TD-M09、ADR-0192）。

## Scope

### 成果物

- `web/src/lib.rs`: on_cleanup で object_urls / cover_url_state を drain/take + revoke
- `docs/decisions/0192-blob-url-cleanup.md`: 追加
- `todo.md`: TD-M09 を完了へ移動

### 決定

- SendWrapper で Rc を包む ADR-0166 パターンを再利用。
- 既存 revoke は維持、E2E 249 件維持。
- to_blob レースは未決定として ADR に記録。

## Acceptance criteria

- E2E 249 項目、全ゲート成功（検証済み）。
