# Goal

`close_document()` 後に `unsaved_changes` が残留する問題を修正する（R6 TD-L11）。

## Scope

### 背景

`AppState::remove_document()` は `documents` から文書を削除するが、
`unsaved_changes` のキーを削除しない。
force close 等で未保存フラグ `true` の文書を閉じると
`has_unsaved_changes()` が `true` のままになり、終了時確認が誤作動し得る。

### 成果物

- `desktop/src/state/mod.rs`: 変更（remove_document で unsaved フラグも削除）
- 必要なら `desktop/src/commands/document.rs`: 変更（force close の契約明記のみ）
- `docs/decisions/0208-unsaved-changes-cleanup.md`: 追加
- `GOAL.md` / `todo.md`: 契約・完了反映

### 必須事項

1. `remove_document()` が文書削除時に同じ id の `unsaved_changes` エントリも削除する。
2. 文書が存在しない場合も、孤立した unsaved フラグは削除する。
3. unit test を追加:
   - `mark_unsaved()` 後に `remove_document()` すると `has_unsaved_changes()` が false。
   - 存在しない id の孤立フラグも削除される。
4. 既存の `unsaved_changes_state_transitions` を壊さない。
5. 全ゲート成功（E2E 251 維持）。

## Acceptance criteria

- 全ゲート成功、E2E 251 項目成功。

## Constraints

- 2 つの RwLock を同時に保持しない（デッドロック回避）。
- 新規依存を追加しない。
- `master` へ push する。force push はしない。
