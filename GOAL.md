# Goal

`image_dimensions()` の JPEG / WebP ヘッダーと境界値テストを追加する（R6 TD-M01）。

## Scope

### 背景

`image_dimensions()` は PNG / JPEG / GIF / WebP をサポートするが、
unit test は PNG / GIF のみ。JPEG / WebP のヘッダー解析や
境界値の回帰を検出できない（R6 TD-M01）。

### 成果物

- `core/src/image_dimensions.rs`: 変更（テスト追加）
- `docs/decisions/0184-image-dimension-tests.md`: 追加
- `GOAL.md` / `todo.md`: 契約・完了反映

### 必須事項

1. JPEG 最小ヘッダー（SOI + SOF0）から寸法が取得できるテストを追加。
2. WebP（VP8X または VP8）ヘッダーから寸法が取得できるテストを追加。
3. 0 / 1 / MAX_DIMENSION / MAX_DIMENSION+1 を表駆動で検証（PNG 既存を拡張可）。
4. 全ゲート成功（E2E 249 維持）。

## Acceptance criteria

- 全ゲート成功、E2E 249 項目成功。

## Constraints

- 新規依存を追加しない。
- production code の変更は不要（テストのみ）。
- `master` へ push する。force push はしない。
