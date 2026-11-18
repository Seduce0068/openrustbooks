# Goal

EPUB 画像の自然寸法を取得して HTML に埋め込む方法を調査・判断する（ADR-0112 の第一段）。

## Scope

### 背景

ADR-0112 は画像遅延読み込みの前に「自然寸法を `width` / `height` で予約」する方針を決定した。
現在 `EpubCoverImage` はバイト列のみを保持し、寸法は保持しない。
Web 側の `resolve_images()` は Blob URL 化のみで、属性付与は行わない。

### 成果物

- `docs/decisions/0136-image-dimension-reservation.md`: 追加（調査と判断）
- `GOAL.md`: 本契約

### 必須事項

1. 既存依存（image クレート等）で PNG / JPEG / WebP / GIF の寸法を取得できるか確認する。
2. 取得した寸法を `img` の `width` / `height` 属性として埋め込む層（core のサニタイズ後／web の resolve_images）を比較する。
3. SVG の寸法（width/height 属性、viewBox）の扱いを決める。
4. 実装／見送り／段階導入を判断し記録する。
5. コード変更は行わない（調査と決定のみ）。

## Acceptance criteria

- 判断が ADR に記録される。
- markdownlint が通る。

## Constraints

- コードを変更しない。
- 新規依存を追加しない（調査のみ）。
- `master` へ push する。force push はしない。
