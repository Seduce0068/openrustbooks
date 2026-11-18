# Goal

EPUB HTML の `img` の作者指定 `width` / `height` をサニタイザで数値検証する（ADR-0149 の未決定）。

## Scope

### 背景

`core/src/sanitize.rs:94` は `img` に `width` / `height` を許可するが、
`attribute_filter` は `src` のみ検証している（:133-139）。
ADR-0149 の実装ではラスター画像は自然寸法で上書きするが、
SVG は既存属性を尊重するため、非数値・極端な値が残り得る。

### 成果物

- `core/src/sanitize.rs`: 変更（`img` の width/height 検証）
- `docs/decisions/0150-img-dimension-validation.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. `img` の `width` / `height` 属性が ASCII 数字のみで、
   1〜16384 の範囲にある場合のみ保持する。
2. それ以外（空 / 単位付き / 負 / 小数 / 0 / 上限超 / 非 ASCII 数字）は
   属性ごと削除する（fail-closed）。
3. 上限は `image_dimensions::MAX_DIMENSION` を再利用する（重複定義しない）。
4. テストを追加（有効値維持、不正値削除、svg の既存属性は別経路なので影響なし）。
5. 既存の 250 テスト + wasm 4、E2E 231 を壊さない。

## Acceptance criteria

- 6 ゲート + audit + wasm-test + E2E 231 が成功する。

## Constraints

- 新規依存を追加しない。
- `src` のローカル参照検証は維持する。
- `master` へ push する。force push はしない。
