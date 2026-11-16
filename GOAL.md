# Goal

ブラウザ標準のピンチズームがリーダーで阻害されていないことを確認し、ADR に記録する。

## Scope

### 背景

ADR-0025 の未決定「ピンチズーム・ダブルタップなどのジェスチャー」。
`index.html` の viewport は `width=device-width, initial-scale=1.0` で
`user-scalable=no` は無く、`touch-action` の指定も無いため、ブラウザ標準の
ピンチズームは有効なはず。これを確認して未決定を整理する。

### 成果物

- `docs/decisions/0062-browser-pinch-zoom.md`: 追加
- `tools/verify-web-reader.py`: 変更（設定確認の E2E 検証を追加）
- `GOAL.md`: 本契約

### 必須事項

1. 確認:
   - viewport メタタグに `user-scalable=no` / `maximum-scale=1` が無い。
   - `.reader-content` に `touch-action: none` が指定されていない。
2. 既存の挙動を変えない:
   - プロダクションコードは変更しない（確認のみ）。
   - 既存の 124 テスト、E2E 179 項目を変えない。

### 必須テスト

- `tools/verify-web-reader.py` に追加する。
  - viewport がズームを許可している。
  - 本文領域の `touch-action` が `none` でない。

## Acceptance criteria

- 6 ゲートすべて成功する。
- CI は billing issue で実行不可（ローカル検証で代替）。

## Constraints

- 新規依存を追加しない。
- プロダクションコードを変更しない。
- `master` へ push する。force push はしない。
