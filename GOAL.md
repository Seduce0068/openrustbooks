# Goal

縦書き表示とページ送りの現状を調査し、段階導入方針を判断する（todo.md の残タスク）。

## Scope

### 背景

Web リーダーは現在横書きの連続スクロール（`readers-writing-mode` 等は未実装）。
ユーザー要望には縦書き／ページ送りが残っているが、要件と実装経路が未整理。

### 成果物

- `docs/decisions/0145-vertical-writing-pagination.md`: 追加（調査と判断）
- `GOAL.md`: 本契約

### 必須事項

1. Web / core の現在の表示・スクロール・位置復元の仕組みを列挙する。
2. 縦書き（CSS `writing-mode: vertical-rl`）を入れた場合に壊れる既存機能（位置復元、画像、フラグメント、E2E）を特定する。
3. ページ送り（連続スクロールとの併用／排他）の選択肢を比較する。
4. 実装／見送り／段階導入を判断し記録する。
5. コード変更は行わない（調査と決定のみ）。

## Acceptance criteria

- 判断が ADR に記録される。
- markdownlint が通る。

## Constraints

- コードを変更しない。
- 新規依存を追加しない（調査のみ）。
- `master` へ push する。force push はしない。
