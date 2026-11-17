# Goal

`expected_images` を解決済み画像の実数に合わせ、src 削除画像による無駄な 2 秒待ちを解消する（ADR-0113 の未決定）。

## Scope

### 背景

`expected_images` は章 HTML の `"<img"` 出現数で計算されるが、
`resolve_images()` で解決失敗して `src` を削除された画像もカウントされる。
一方 `images_settled()` は src なし画像をスキップするため
`with_source >= expected` が成立せず、2 秒 deadline まで待ち続ける。

### 成果物

- `web/src/lib.rs`: 変更（`expected_images` の算出方法を修正）
- `docs/decisions/0114-expected-image-count.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. `expected_images` を「src を持つ画像の数」に変更する。
2. `images_settled()` の `with_source >= expected` ロジックは維持する。
3. 既存の 193 テスト、E2E 228 項目を壊さない。
4. 新規依存を追加しない。

## Acceptance criteria

- 6 ゲート・E2E が成功する。

## Constraints

- 遅延読み込みは導入しない。
- `images_settled()` 本体の判定（`complete() && naturalWidth > 0`）は変更しない。
- `master` へ push する。force push はしない。
