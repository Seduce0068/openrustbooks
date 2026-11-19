# Goal

Reader エラー status の redaction を実施する（R6 TD-M12、ADR-0212）。

## Scope

### 成果物

- `web/src/lib.rs`: 6 経路の固定文言化、report_reader_error、cfg(wasm32)
- `tools/verify-web-reader.py`: E2E 1 件、251 → 252
- `docs/decisions/0212-status-error-redaction.md`: 追加
- `todo.md`: TD-M12 を完了へ移動

### 決定

- status も console も操作別固定文言のみ。
- 生エラー / Debug / 型名 / パス / URL / 本文は出力しない。
- 既存 "This link cannot be opened." は維持。

## Acceptance criteria

- E2E 252 項目、全ゲート成功（検証済み）。
