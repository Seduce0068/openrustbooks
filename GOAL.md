# Goal

エクスポート名のタイムスタンプにタイムゾーン偏移を付与する。

## Scope

### 背景

ADR-0069 の未決定「ファイル名へのタイムゾーン明示」。
現在の `openrustbooks-YYYYMMDD-HHMMSS.json` はローカル時刻だが、どの
タイムゾーンの時刻か分からない。偏移を付けると、異なるタイムゾーンの
端末間でバックアップを扱うときに順序を正しく判断できる。

### 成果物

- `web/src/lib.rs`: 変更（ファイル名への偏移付与）
- `tools/verify-web-reader.py`: 変更（E2E 検証を更新）
- `docs/decisions/0070-export-filename-timezone.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. ファイル名:
   - `openrustbooks-YYYYMMDD-HHMMSS+HHMM.json` 形式にする
     （例: `openrustbooks-20261117-093500+0900.json`）。
   - 偏移は `Date.getTimezoneOffset()` の符号を反転させた値（分）を
     `HHMM` に変換する。UTC は `+0000`。
2. 既存の挙動を変えない:
   - JSON の中身は変えない。
   - 既存の 125 テスト、E2E 198 項目を壊さない（ファイル名形式の検証は更新する）。

### 必須テスト

- `tools/verify-web-reader.py` に追加する。
  - エクスポート名が `openrustbooks-\d{8}-\d{6}[+-]\d{4}\.json` 形式である。

## Acceptance criteria

- 6 ゲートすべて成功する。
- CI は billing issue で実行不可（ローカル検証で代替）。

## Constraints

- 新規依存を追加しない。
- `desktop/` を変更しない。
- コメントと doc コメントは日本語で簡潔に書く。
- `master` へ push する。force push はしない。
