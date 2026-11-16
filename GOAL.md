# Goal

EPUB 本文の深い入れ子 XHTML がサニタイズ時に安全に処理されることをテストで確認する。

## Scope

### 背景

ADR-0013 の未決定「深い入れ子の XHTML によるスタック消費」。
`core/src/sanitize.rs` のサニタイズに深い入れ子のテストが無く、非常に深い
`<div>` の入れ子でスタックオーバーフローや異常終了しないことを確認する。

### 成果物

- `core/src/sanitize.rs`: 変更（深い入れ子のテストを追加）
- `docs/decisions/0050-deep-nesting-safety.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. テスト:
   - 1000 階層の `<div>` 入れ子を `sanitize_epub_html` に渡しても
     パニックせず、文字列が返ること。
   - サニタイズ結果にスクリプトやイベントハンドラが残らないこと
     （既存のサニタイズ方針を維持）。
2. 既存の挙動を変えない:
   - プロダクションコードは変更しない（テスト追加のみ）。
   - 既存の 119 テスト、E2E 166 項目を変えない。

### 必須テスト

- `cargo test --all-targets --locked` が新しいテストを含めて成功する。

## Acceptance criteria

- 6 ゲートすべて成功する。
- CI は billing issue で実行不可（ローカル検証で代替）。

## Constraints

- 新規依存を追加しない。
- プロダクションコードを変更しない。
- `master` へ push する。force push はしない。
