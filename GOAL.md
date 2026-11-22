# Goal

save_document の並行編集 lost update を修正する（第2cycle R6 / R2-RC01）。

## Scope

### 背景

save_document_impl は get_document で clone し、atomic_write().await 中に
set_document_content が新しい本文を格納しても、完了後に古い snapshot を
state.add_document へ書き戻し mark_saved するため最新本文が失われる。

### 成果物

- `desktop/src/state/mod.rs`: 変更（文書世代 / 保存 CAS API）
- `desktop/src/commands/document.rs`: 変更
- `docs/decisions/0318-document-save-cas.md`: 追加
- `todo.md` / README / 索引 / log: 更新

### 必須事項

1. 保存開始時の文書世代を記録し、書き込み後に再確認する。
2. 並行編集があれば古い snapshot を state へ書き戻さず、Conflict を返す。
3. 成功時のみ metadata / mark_saved / emit を行う。
4. content=None の保存と content=Some の保存の双方で安全に動作する。
5. 競合再現 test（Barrier 等）と通常保存 test を追加。
6. verify.sh 全ゲート成功、E2E 256 件・WDIO 維持。

## Constraints

- 外部サービス / 秘密情報は扱わない。
- 既存 E2E check 名・件数を変更しない。
- `master` へ push する。force push はしない。

## 完了記録

- save_document に世代 CAS を導入し、並行編集/force close の競合を Detection。
- 競合時に古い snapshot / mark_saved / emit を防止。
- desktop 157 tests、verify.sh 全ゲート成功（E2E 256、WDIO 1）。
- ディスク復元 / Conflict UI 文言は ADR-0318 の未決定。
