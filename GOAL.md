# Goal

EPUB 非同期ロードに世代管理を追加し、古いファイルが新しい選択を上書きする問題を解消する（R6 PR-H03/TD-H03）。

## Scope

### 背景

`on_file_change()` は選択ごとに `spawn_local` を投げるが、完了前に別ファイルを選んでも古いタスクをキャンセルできない。古いタスクが後で完了すると `*book.borrow_mut() = Some(opened)` と `render_chapter()` が走り、最後に選んだ本ではない内容が表示される。

### 成果物

- `web/src/lib.rs`: 変更（load generation カウンタと完了時チェック）
- `tools/verify-web-reader.py`: 変更（E2E 1 件、247 → 248）
- `docs/decisions/0176-epub-load-generation.md`: 追加
- `GOAL.md` / `todo.md`: 契約・完了反映

### 必須事項

1. Reader 内にロード世代カウンタ（`Rc<Cell<u64>>`）を置き、ファイル選択ごとにインクリメントする。
2. `spawn_local` 完了時に世代が現在値と不一致なら、book / history / UI state を更新せず破棄する。
3. 他の操作（chapter select / scroll）に影響しない。
4. E2E で疑似並列ロード（2 ファイル連続投入）を再現し、最後のファイルが表示されることを検証（可能なら）。
5. 全ゲート成功。

## Acceptance criteria

- E2E 248 項目（または再現不能なら 247 維持＋理由記録）、全ゲート成功。

## Constraints

- 新規依存を追加しない。
- 既存の EPUB read / render 契約を維持。
- `master` へ push する。force push はしない。
