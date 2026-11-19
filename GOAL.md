# Goal

表紙 media-type 契約を回 regress test で固定した（R6 R1-archive#5、ADR-0249）。

## Scope

### 成果物

- `core/src/epub.rs`: is_image_media_type の unit test 追加
- `docs/decisions/0249-cover-media-type.md`: 追加
- `todo.md`: R1-archive#5 完了、#2 / #3 残存
- README / 各索引 / log: 更新

### 決定

- 空許容、image/ 受入、明示された非画像拒否を unit test で固定。
- 本番コードと E2E 255 は不変。

## Acceptance criteria

- core 279 / E2E 255、全ゲート成功（検証済み）。
