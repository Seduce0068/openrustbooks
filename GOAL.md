# Goal

save_document の root 選択を read_only fail-closed 化する（第2cycle R6 最優先 / R2-RC01）。

## Scope

### 背景

resolve_within_roots は canonical_roots を先頭から走査し、
最初に一致した root を返す。writable 親 + read-only 子の入れ子では
子 root の read_only が無視され、save_document 経由で書き換えられる。
fs.rs の resolve_target_storage は全候補を検査し、一つでも read_only なら拒否する。

### 成果物

- `desktop/src/commands/document.rs`: 変更
- `docs/decisions/0312-document-root-selection.md`: 追加
- `todo.md` / README / 索引 / log: 更新

### 必須事項

1. resolve_within_roots を全候補検査・最深一致方式へ変更する。
2. 候補に read_only root が一つでもあれば固定文言で拒否する。
3. symlink / containment / atomic save / front matter を壊さない。
4. writable 親 + read-only 子（両順）の回帰 test を追加。
5. verify.sh 全ゲート成功、E2E 256 件・WDIO 維持。

## Constraints

- 外部サービス / 秘密情報は扱わない。
- 既存 E2E check 名・件数を変更しない。
- `master` へ push する。force push はしない。
