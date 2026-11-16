# Goal

`remote-resources` を宣言する EPUB の扱いをテストで固定し、ADR-0005 の未決定を解消する。

## Scope

### 背景

ADR-0059 の未決定「`remote-resources` プロパティを宣言する EPUB の扱い」。
core の OPF パーサーは manifest item の `properties` のうち `nav` と
`cover-image` のみを解釈し、他のトークンは無視する。実際のリモートアクセスは
サニタイズ（外部 img src の除去、SVG 外部参照の除去、CSS の除去）と
Blob URL 解決で塞がれている。この挙動をテストで確認し ADR に記録する。

### 成果物

- `core/src/epub.rs`: 変更（テスト追加）
- `docs/decisions/0060-remote-resources-policy.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. テスト:
   - `properties="nav remote-resources"` の OPF を持つ EPUB が開け、
     目次と章本文が読めること。
   - `remote-resources` トークンが目次・表紙の判定を壊さないこと。
2. 既存の挙動を変えない:
   - プロダクションコードは変更しない（テスト追加のみ）。
   - 既存の 123 テスト、E2E 176 項目を変えない。

## Acceptance criteria

- 6 ゲートすべて成功する。
- CI は billing issue で実行不可（ローカル検証で代替）。

## Constraints

- 新規依存を追加しない。
- プロダクションコードを変更しない。
- `master` へ push する。force push はしない。
