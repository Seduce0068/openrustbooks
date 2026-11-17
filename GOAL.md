# Goal

README の残存 markdownlint エラー（MD040）を解消する。

## Scope

### 背景

README 7 行目のコードブロックに言語指定が無く、markdownlint が
MD040/fenced-code-language を報告している。内容はディレクトリツリーなので
`text` を指定する。

### 成果物

- `README.md`: 変更（コードブロックに `text` を指定）
- `GOAL.md`: 本契約

### 必須事項

1. プロジェクト構成のコードブロックを ` ``` ` から ` ```text ` に変更する。
2. markdownlint が README 全体で 0 issues になる。
3. コードは変更しない。

## Acceptance criteria

- `markdownlint-cli2 README.md` が 0 issues。
- 6 ゲートすべて成功する。

## Constraints

- コードを変更しない。
- `master` へ push する。force push はしない。
