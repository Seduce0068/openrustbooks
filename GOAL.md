# Goal

ADR-0044 の未決定（XML 処理命令）を ADR-0045 の解決結果へ更新する（todo.md R5-adr#2）。

## Scope

### 背景

ADR-0044 の「未決定」に XML 処理命令（`<?xml-stylesheet?>`）の扱いが残っているが、
ADR-0045 で `PROCESSING_INSTRUCTION_NODE` の削除を決定・実装した。

### 成果物

- `docs/decisions/0044-svg-external-references.md`: 変更
- `GOAL.md`: 本契約

### 必須事項

1. XML 処理命令の項目を「未決定」から「解決済み」へ移し、ADR-0045 を参照する。
2. 他の未決定（CSS `url()`、遅延読み込み、EPUB 内 CSS 等）は維持する。
3. コードは変更しない。

## Acceptance criteria

- markdownlint が通る。

## Constraints

- コードを変更しない。
- `master` へ push する。force push はしない。
