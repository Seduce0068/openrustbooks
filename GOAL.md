# Goal

`WriteOptions.overwrite` を実効化し、既存ファイルの黙って上書きを防ぐ（R6 最優先 PR-H01/TD-H01、ADR-0173）。

## Scope

### 背景

`LocalStorage::write()` は `options.overwrite` を無視し、`create_file()` の新規作成操作が既存データを破壊し得た。

### 成果物

- `core/src/io.rs`: 既存拒否ガードと unit test 6 件
- `docs/decisions/0173-overwrite-guard.md`: 追加
- `todo.md`: PR-H01/TD-H01 を完了へ移動
- `GOAL.md`: 最終契約へ更新

### 決定

- `resolve_path()` 直後、`create_parents` / `atomic` 分岐より前に overwrite 判定。
- エラー文言は root 相対パスのみ（正規化不能なら固定文言）。
- TOCTOU と固定 tmp 名の衝突は未決定として継続。

## Acceptance criteria

- E2E 246 項目、全ゲート成功（検証済み）。
