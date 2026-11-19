# Goal

未使用の WasmConfig 公開 API を削除し、設定モデルの重複を減らす（R6 PR-M08 / TD-M08 の一部）。

## Scope

### 背景

`core/src/wasm.rs` の `WasmConfig` は wasm32 向け公開型だが、
`web/src/lib.rs` / desktop からは未参照で、wasm unit test 3 件のみが依存する。
`auto_fix` の権威は `LintConfig`、AI 既定値は native `AiConfig` にあり、
設定モデル 4 重化の原因（ADR-0160 / 0194）。

### 成果物

- `core/src/wasm.rs`: 変更（WasmConfig 定義・実装・関連 test 3 件を削除）
- `docs/decisions/0205-wasm-config-removal.md`: 追加
- `GOAL.md` / `todo.md`: 契約・完了反映

### 必須事項

1. `WasmConfig` struct / Default / wasm_bindgen impl / import / 3 tests を削除する。
2. 本番コード・test・docs に WasmConfig 参照が残らないこと（歴史 ADR の言及は維持し、新規 ADR で supersede を明記）。
3. wasm-pack test が 10 → 7 件になること（削除 3 件）、他テストを壊さない。
4. 全ゲート成功（E2E 251 維持）。

## Acceptance criteria

- 全ゲート成功、E2E 251 項目成功。

## Constraints

- 新規依存を追加しない。
- 既存の `WasmMarkdownLinter` / `render_markdown_wasm` などの公開 API は維持。
- `master` へ push する。force push はしない。
