# Goal

close_document の unsaved チェックと削除を atomic 化する（第2cycle R6 / R2-RC01）。

## Scope

### 背景

close_document は `is_unsaved()` と `remove_document()` を別ロックで呼ぶ。
その間に `set_document_content` が mark_unsaved すると、
force=false でも未保存編集が警告なしに削除される。

### 成果物

- `desktop/src/state/mod.rs`: 変更（atomic close API）
- `desktop/src/commands/document.rs`: 変更
- `docs/decisions/0313-atomic-document-close.md`: 追加
- `todo.md` / README / 索引 / log: 更新

### 必須事項

1. unsaved 検査と文書削除を単一 write lock クリティカルセクションで行う。
2. force=false で unsaved なら削除せず固定エラーを返す。
3. force=true は従来どおり削除する。
4. emit / unsaved フラグ / documents の整合を壊さない。
5. 競合再現 test（疑似的に隔挿入）と force 分岐 test を追加。
6. verify.sh 全ゲート成功、E2E 256 件・WDIO 維持。

## Constraints

- 外部サービス / 秘密情報は扱わない。
- E2E check 名・件数を変更しない。
- `master` へ push する。force push はしない。
