# Goal

未検出バックエンド / provider の unit test を追加する（R6 R4-unit#3 残、ADR-0231）。

## Scope

### 成果物

- `core/src/io.rs`: 未検出 backend と成功経路の test
- `core/src/ai_markdown/client.rs`: 未知 provider と API キー非漏洩の test
- `docs/decisions/0231-unknown-backend-tests.md`: 追加
- `todo.md`: R4-unit#3 を完了へ移動

### 決定

- 本番コードは変更せず、エラー分岐と文言を test で固定。
- core 267→270（+3）、E2E 254 維持。

## Acceptance criteria

- core 270 tests、E2E 254、全ゲート成功（検証済み）。
