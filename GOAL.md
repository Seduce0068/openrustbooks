# Goal

画像の読み込み完了判定を `complete()` から `naturalWidth > 0` も含むように強化する（ADR-0112 の準備）。

## Scope

### 背景

MDN によれば `HTMLImageElement.complete` は
`src` 未設定／空／broken／画像読み込み無効でも `true` を返す。
現状の `images_settled()` は `complete()` のみで判定しており、
自然サイズが確定していない画像を「完了」とみなす可能性がある。
位置復元（ADR-0020/0037/0046）は画像の自然サイズに依存するため、
判定を強化する。

### 成果物

- `web/src/lib.rs`: 変更（`images_settled()` に `naturalWidth > 0` を追加）
- `tools/verify-web-reader.py`: 変更（E2E 検証を追加、任意）
- `docs/decisions/0113-image-settle-hardening.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. `images_settled()` は `complete()` に加えて `natural_width() > 0` を要求する。
2. 既存の 193 テスト、E2E 228 項目を壊さない。
3. 新規依存を追加しない。

## Acceptance criteria

- 6 ゲート・E2E が成功する。

## Constraints

- 遅延読み込み（`IntersectionObserver`）は導入しない。
- 位置復元・スクロール補正のロジックは変更しない。
- `master` へ push する。force push はしない。
