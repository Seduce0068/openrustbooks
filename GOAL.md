# Goal

バックアップ import JSON の実用上限と構造制限を追加する（第2cycle R6 / R3-SE01）。

## Scope

### 背景

`IMPORT_JSON_MAX_BYTES` は core の 256 MiB から導出されており、
`serde_json::from_str::<Value>` の展開で元テキストに加えて
数百 MB のツリーを確保し得る。ブラウザタブのヒープ枯渇が懸念される。

### 成果物

- `web/src/lib.rs`: 変更（実用上限 + エントリ数 / 深さ制限）
- `docs/decisions/0317-import-json-limits.md`: 追加
- `todo.md` / README / 索引 / log: 更新

### 必須事項

1. バイト上限を実用値（例 8 MiB）へ下げる。
2. `serde_json::Value` のキー数 / 配列長 / ネスト深さを制限する。
3. 上限超過は固定文言 `IMPORT_JSON_TOO_LARGE_MESSAGE` で拒否する。
4. パス / ファイル名 / キー / 値をエラーへ出さない。
5. 既存 E2E 256 件・check 名・export/import 正常系を維持。
6. verify.sh 全ゲート成功。

## Constraints

- 新規依存は使わない。
- 256 MiB の read 上限（core::io）は変更しない。
- `master` へ push する。force push はしない。

## 完了記録

- import JSON を 8 MiB 実用上限 + parse 前構造走査へ変更。
- 構文エラーと構造制限を分離し、既存 E2E 文言を維持。
- web 17 tests、verify.sh 全ゲート成功（E2E 256、WDIO 1）。
- 上限設定化 / incremental import は ADR-0317 の未決定。
