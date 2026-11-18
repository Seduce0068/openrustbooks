# Goal

未実装の `watch` を「利用可能」と報告しないようにする（R6 TD-H04）。

## Scope

### 背景

`LocalStorage::new()` は `can_watch: true` を報告するが、
`watch()` 本体は `"Watch not implemented yet"` を返す。
`watch_directory` コマンドも登録済みで、呼ぶと常に失敗する。

### 成果物

- `core/src/io.rs`: 変更（`can_watch: false`、unit test）
- `desktop/src/commands/fs.rs`: 変更（コマンドを明示的 unsupported に）
- `desktop/src/main.rs`: 変更（invoke_handler から削除）
- `docs/decisions/0178-watch-capability-honesty.md`: 追加
- `GOAL.md` / `todo.md`: 契約・完了反映

### 必須事項

1. `LocalStorage::new()` の `can_watch` を `false` にする。
2. `watch_directory` コマンドを `invoke_handler` から外すか、
   「not implemented」を返す明示スタブにする（方針を ADR に記録）。
3. `StorageWatchStream` / `StorageEvent` 型や `watch()` トレイトは
   将来実装のため残してよい。
4. unit test を追加（capabilities.can_watch == false）。
5. 全ゲート成功（E2E 247 維持）。

## Acceptance criteria

- 全ゲート成功、E2E 247 項目成功。

## Constraints

- 新規依存（notify 等）を追加しない（今回は宣言整合のみ）。
- 既存の storage 契约を壊さない。
- `master` へ push する。force push はしない。
