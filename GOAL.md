# Goal

save_document を atomic write に変更する（R6 レビュー / R1-ER01）。

## Scope

### 背景

desktop/src/commands/document.rs の save_document は
`tokio::fs::write` で既存ファイルを truncate してから書き込む。
書き込み途中エラーで元ファイルが空／部分内容のまま残る。
core::io には `write_atomic` が既に存在する。

### 成果物

- `desktop/src/commands/document.rs`: 変更
- `docs/decisions/0298-atomic-document-save.md`: 追加
- `todo.md` / README / 索引 / log: 更新

### 必須事項

1. save_document は atomic temp + rename で保存する。
2. 失敗時は元ファイルを保持し、metadata / mark_saved / emit を行わない。
3. 成功時のみ metadata / mark_saved / emit を更新する。
4. 単体 test で書き込み失敗時に元内容が残ることを検証する。
5. front matter 保全、read_only、containment を壊さない。
6. verify.sh 全ゲート成功、E2E 256 件・WDIO 維持。

## Constraints

- 外部サービス / 秘密情報は扱わない。
- E2E check 名・件数を変更しない。
- `master` へ push する。force push はしない。
