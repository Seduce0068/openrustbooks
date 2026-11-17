# Goal

エクスポート JSON のダウンロード名にタイムスタンプを付与する。

## Scope

### 背景

エクスポートのダウンロード名が `openrustbooks-positions.json` 固定のため、
同じブラウザで再度エクスポートすると `openrustbooks-positions (1).json` のように
ブラウザが連番を付け、どれが最新か分かりにくい。タイムスタンプを付けて
バックアップの識別を容易にする。

### 成果物

- `web/src/lib.rs`: 変更（ダウンロード名の生成）
- `tools/verify-web-reader.py`: 変更（E2E 検証を追加）
- `docs/decisions/0069-timestamped-export-filename.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. ファイル名:
   - ダウンロード名を `openrustbooks-YYYYMMDD-HHMMSS.json` にする
     （ローカルタイムゾーン）。
   - 秒まで含め、同じ秒に 2 回エクスポートした場合のみブラウザの連番に任せる。
2. 既存の挙動を変えない:
   - JSON の中身（version / positions / settings / history）は変えない。
   - 既存の 125 テスト、E2E 197 項目を変えない。

### 必須テスト

- `tools/verify-web-reader.py` に追加する。
  - ダウンロードされたファイル名が `openrustbooks-` で始まり
    `8桁の日付-6桁の時刻.json` の形式である。

## Acceptance criteria

- 6 ゲートすべて成功する。
- CI は billing issue で実行不可（ローカル検証で代替）。

## Constraints

- 新規依存を追加しない。
- `desktop/` を変更しない。
- コメントと doc コメントは日本語で簡潔に書く。
- `master` へ push する。force push はしない。
