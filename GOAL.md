# Goal

status 文言を日本語へ統一する（R6 TD-M11 第一段、ADR-0226）。

## Scope

### 成果物

- `web/src/lib.rs`: Reader status 22 パターン／24 箇所を日本語化
- `tools/verify-web-reader.py`: 文言期待 23 箇所を同期、check 総数 254 維持
- `docs/decisions/0226-status-message-language.md`: 追加
- `todo.md`: TD-M11 第一段を完了へ移動

### 決定

- lang="ja" に合わせ status を日本語統一。
- ADR-0212 の redaction と console 操作名は維持。
- aria-live / role は変更せず、error 分離は第二段の未決定。

## Acceptance criteria

- E2E 254 項目、全ゲート成功（検証済み）。
