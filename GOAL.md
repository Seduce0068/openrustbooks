# Goal

外部リンクをクリックしたときに新しいタブが開くことを E2E で検証する。

## Scope

### 背景

ADR-0080 では外部リンクの href と `noopener noreferrer` を検証したが、
実際にクリックして `window.open(..., "_blank", "noopener,noreferrer")` が
呼ばれることは未検証。CDP の `Page.windowOpen` イベントで確認する。

### 成果物

- `tools/verify-web-reader.py`: 変更（ポップアップ検証を追加）
- `docs/decisions/0081-external-link-popup-test.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. E2E 検証:
   - `Page.windowOpen` イベントを捕捉し、外部リンクをクリックする。
   - 新しいウィンドウが `example.com` の URL で開くことを確認する（実際の
     ネットワークアクセスは発生させない）。
   - 開いたウィンドウを閉じる。
2. 既存の 125 テスト、E2E 214 項目を変えない。

## Acceptance criteria

- 6 ゲートすべて成功する。
- CI は billing issue で実行不可（ローカル検証で代替）。

## Constraints

- プロダクションコードを変更しない。
- `master` へ push する。force push はしない。
