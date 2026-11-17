# Goal

古い ADR に残る解消済みの未決定項目を整理する。

## Scope

### 背景

ADR-0005 〜 0013 の未決定には、その後の ADR で既に解消された項目が多数残っている。
例えば目次解析（ADR-0006）、非整形式 XHTML（ADR-0059）、リモート資源
（ADR-0060/0066）、SVG の Blob URL（ADR-0043〜0045）、深い入れ子（ADR-0050/0065）、
メモリコピー（ADR-0051）、相対リンク（ADR-0014）など。

### 成果物

- `docs/decisions/0005-epub-body-reading.md`
- `docs/decisions/0006-epub-table-of-contents.md`
- `docs/decisions/0007-epub-toc-fragments.md`
- `docs/decisions/0009-epub-landmarks-and-page-list.md`
- `docs/decisions/0010-epub-wasm-bindings.md`
- `docs/decisions/0012-epub-html-display.md`
- `docs/decisions/0013-epub-display-hardening.md`
- `GOAL.md`: 本契約

### 必須事項

1. 各 ADR の「未決定」から解消済み項目を「解決済み」セクションへ移し、
   参照先 ADR を明記する。
2. まだ未実装の項目（EPUB 内 CSS、縦書き、メディア、MathML 等）は
   「未決定」に残す。
3. コードは変更しない。

## Acceptance criteria

- markdownlint が通る。
- 6 ゲートすべて成功する。

## Constraints

- コードを変更しない。
- `master` へ push する。force push はしない。
