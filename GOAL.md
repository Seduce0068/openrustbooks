# Goal

`IntersectionObserver` による画像遅延読み込みの実装可能性を調査・判断する（ADR-0034 / 0043 等から継続）。

## Scope

### 背景

多数の画像を含む EPUB では、表示前の画像まで全部読み込むため
初期表示が遅くなる。`IntersectionObserver` で可視付近の画像だけ `src` を
設定する遅延読み込みが提案されているが、位置復元・スクロール補正・
フラグメント移動・E2E の `images_settled()` に波及する。

### 成果物

- `docs/decisions/0112-lazy-image-loading.md`: 追加（調査と判断）
- `GOAL.md`: 本契約

### 必須事項

1. 現状の画像読み込み・待機・位置補正の仕組みをコードから整理する。
2. 遅延読み込みを導入した場合の影響箇所を列挙する。
3. 実装する／見送る／段階導入するかを判断し、根拠を記録する。
4. コード変更は行わない（調査と決定のみ）。

## Acceptance criteria

- 判断が ADR に記録される。
- markdownlint が通る。

## Constraints

- コードを変更しない。
- `master` へ push する。force push はしない。
