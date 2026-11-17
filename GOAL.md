# Goal

EPUB 内 CSS の安全な適用方針を調査・判断する（ADR-0012 / 0044 / 0112 等から継続）。

## Scope

### 背景

`core/src/sanitize.rs` は `style` タグと `style` 属性を完全に除去する（fail-closed）。
このため EPUB 独自の組版（縦書き、見出し装飾、独自フォント等）が失われる。
安全に適用するには `url()` 外部参照の除去、スコープ付きスタイル、
CSP `style-src` との整合が必要。

### 成果物

- `docs/decisions/0115-epub-css-policy.md`: 追加（調査と判断）
- `GOAL.md`: 本契約

### 必須事項

1. 現状のサニタイズ・CSP・表示経路を整理する。
2. 安全に CSS を適用するための選択肢を比較する。
3. 実装する／見送る／段階導入するか判断し根拠を記録する。
4. コード変更は行わない（調査と決定のみ）。

## Acceptance criteria

- 判断が ADR に記録される。
- markdownlint が通る。

## Constraints

- コードを変更しない。
- `master` へ push する。force push はしない。
