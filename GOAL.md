# Goal

エラー文言の二重付与を整理し、desktop の `expect()` を Result へ変更する（R6 C05 / R3-error#3,#5）。

## Scope

### 背景

`desktop/src/main.rs` の `run()` は `()` を返し、末尾で
`.expect("error while running tauri application")` により panic していた。
また `CoreError` の Display が prefix を持つのに、
内部で同じ prefix を付ける二重付与が `lint.rs` / `client.rs` に残っていた。
`web/src/lib.rs` の invoke エラーは `JsValue` の Debug を露出していた。

### 成果物

- `desktop/src/main.rs`: 変更（`run() -> tauri::Result<()>`、`expect()` 削除）
- `core/src/ai_markdown/lint.rs`: 変更（二重 prefix 削除）
- `core/src/ai_markdown/client.rs`: 変更（二重 prefix 削除）
- `web/src/lib.rs`: 変更（invoke エラーを固定文言へ）
- `docs/decisions/0229-error-message-cleanup.md`: 追加
- `GOAL.md` / `todo.md`: 契約・完了反映

### 必須事項

1. `run()` は `tauri::Result<()>` を返し、`expect()` を削除する。
2. non-mobile `main()` は `Display` で 1 行エラー表示し、終了コード 101 を維持する。
3. mobile `main()` は戻り値を捨てず `log::error!` で記録する（unused_must_use 回避）。
4. `CoreError::lint` / `ai_provider` の二重 prefix を削除する。
5. `invoke failed: {err:?}` を固定文言へ置換する（JsValue Debug 非露出）。
6. 既存の redaction / status 契約を壊さず、E2E 254 を維持する。
7. 全ゲート成功。

## Acceptance criteria

- 全ゲート成功、E2E 254 項目成功。

## Constraints

- status の error / info 分離（専用 aria-live）は ADR 未決定として残す。
- 新規依存を追加しない。
- `master` へ push する。force push はしない。
