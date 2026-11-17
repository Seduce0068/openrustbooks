# Goal

調査で検出されたデッドコードと古い契約値を整理する。

## Scope

### 背景

`docs/investigation/round-01.md`（別セッションの調査レポート）で次を指摘された。
- `tools/verify-web-reader.py` の `open_jpeg_and_measure()` が未呼び出し
  （JPEG サイズ比較テスト削除時の残骸、約 38 行）。
- `GOAL.md` の E2E 項目数が 216 のままで、現在は 219。

### 成果物

- `tools/verify-web-reader.py`: 変更（未使用関数を削除）
- `GOAL.md`: 本契約（数値は現状 219 を記載）
- `docs/decisions/0083-dead-code-cleanup.md`: 追加

### 必須事項

1. 未使用の `open_jpeg_and_measure()` を削除する。
2. 削除後も E2E が通る（項目数は変わらない）。
3. 既存の 125 テストを変えない。
4. 調査レポート（`docs/investigation/`）は別セッションの成果物のため
   変更・コミットしない。

## Acceptance criteria

- 6 ゲートすべて成功する。
- E2E 219 項目が通る。

## Constraints

- 新規依存を追加しない。
- コード以外の機能は変更しない。
- `master` へ push する。force push はしない。
