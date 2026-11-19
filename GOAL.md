# Goal

壊れた画像による章ごとの 2 秒セトリング待ちを解消する（R6 TD-M10）。

## Scope

### 背景

`images_settled()` は `src` を持つ全画像に
`complete() && natural_width() > 0` を要求する。
読み込み失敗画像は `complete()` が true でも自然幅 0 のままなので、
`IMAGE_SETTLE_MS = 2000` の deadline まで位置復元が待たされる。

### 成果物

- `web/src/lib.rs`: 変更（画像の load / error 結果を記録し、失敗画像は待ち対象外に）
- `tools/verify-web-reader.py`: 変更（E2E 1 件、253 → 254）
- `docs/decisions/0225-broken-image-settling.md`: 追加
- `GOAL.md` / `todo.md`: 契約・完了反映

### 必須事項

1. `resolve_images()` が生成した `<img>` の `load` / `error` を追跡し、
   失敗画像を `images_settled()` の待ち対象から除外する。
2. 正常画像は従来どおり自然幅確定まで待つ（CLS / 位置復元契約を維持）。
3. 章切り替え・generation 変更時に古い画像のリスナーが誤って効かないようにする。
4. E2E 1 件追加（壊れた画像を含む章で 2 秒未満に位置復元が完了し、
   正常画像の待ちも壊さない）。期待総数 253 → 254。
5. 全ゲート成功。

## Acceptance criteria

- E2E 254 項目、全ゲート成功。

## Constraints

- 既存の EPUB 画像サニタイズ / Blob URL 寿命管理を変更しない。
- 新規依存を追加しない。
- `master` へ push する。force push はしない。
