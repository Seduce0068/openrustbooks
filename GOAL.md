# Goal

Reader の `CoverImage` DTO から未使用フィールドと `allow(dead_code)` を削除する（R6 TD-L04）。

## Scope

### 背景

`web/src/lib.rs` の `CoverImage` は `id` / `media_type` を保持するが参照されず、
型全体に `#[allow(dead_code)]` が付いている（R6 TD-L04）。
デシリアライズしたメタデータを捨てるより、必要なフィールドだけを受け取るべき。

### 成果物

- `web/src/lib.rs`: 変更（`CoverImage` を `href` / `bytes` のみへ縮小、`allow(dead_code)` 削除）
- `docs/decisions/0203-cover-image-dto.md`: 追加
- `GOAL.md` / `todo.md`: 契約・完了反映

### 必須事項

1. `CoverImage` から `id` / `media_type` を削除する。
2. `#[allow(dead_code)]` を削除しても警告が出ないことを確認する。
3. `cover_image()` の JSON から `href` / `bytes` が従来どおり読めることを確認する。
4. 全ゲート成功（E2E 251 維持）。

## Acceptance criteria

- 全ゲート成功、E2E 251 項目成功。

## Constraints

- 表紙表示・Blob URL 生成の挙動は変更しない。
- serde の未知フィールドは無視される既存契約を維持する。
- `master` へ push する。force push はしない。
