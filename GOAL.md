# Goal

`WriteOptions.overwrite` を実効化し、既存ファイルの黙って上書きを防ぐ（R6 最優先 PR-H01/TD-H01）。

## Scope

### 背景

`LocalStorage::write()` は `options.overwrite` を参照せず常に `tokio_fs::write` で上書きする。
`desktop/src/commands/fs.rs::create_file()` は `WriteOptions::default()`（overwrite=false）を
渡すため、「新規作成」操作が既存データを破壊し得る。

### 成果物

- `core/src/io.rs`: 変更（overwrite 判定）
- `desktop/src/commands/fs.rs`: 変更（意図の明示、必要なら）
- `GOAL.md`: 本契約
- `todo.md`: 完了反映

### 必須事項

1. `write()` で `overwrite=false` かつ対象が存在する場合、
   `CoreError::Storage`（例 `"File already exists: ..."`）を返す。
2. `atomic=true` でも同じ判定を `set_item` 前に適用する。
3. 既存の `write_text` / `create_file` の挙動を壊さない
   （新規作成は成功、既存は拒否）。
4. unit test を追加（既存拒否／上書き許可／atomic 分岐）。
5. 全ゲート成功（E2E 246 項目維持）。

## Acceptance criteria

- 全ゲート成功、E2E 246 項目成功。

## Constraints

- 新規依存を追加しない。
- エラー文言にファイル内容を載せない（パスのみ、必要なら相対パス）。
- `master` へ push する。force push はしない。
