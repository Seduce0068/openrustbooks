# Goal

image_dimensions の JPEG / WebP と境界値テストを追加する（R6 TD-M01、ADR-0184）。

## Scope

### 成果物

- `core/src/image_dimensions.rs`: テストのみ追加（production 不変）
- `docs/decisions/0184-image-dimension-tests.md`: 追加
- `todo.md`: TD-M01 を完了へ移動

### 決定

- JPEG SOF0 / WebP VP8X / VP8 lossy のヘッダー形式を imagesize 0.15 源码と突合。
- PNG の 0 / 1 / MAX / MAX+1 を表駆動で検証。
- E2E 249 件維持。

## Acceptance criteria

- core 220 件、E2E 249 件、全ゲート成功（検証済み）。
