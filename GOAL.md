# Goal

未使用の WasmConfig 公開 API を削除する（R6 PR-M08 / TD-M08 の一部、ADR-0205）。

## Scope

### 成果物

- `core/src/wasm.rs`: WasmConfig 定義・impl・Wasm test 3 件を削除
- `docs/decisions/0205-wasm-config-removal.md`: 追加
- `todo.md`: PR-M08 を「残り 3 重」へ更新

### 決定

- 本番参照 0 の公開型を削除し、auto_fix 権威は LintConfig と明記。
- 他 wasm API は維持、歴史 ADR は本 ADR で supersede。

## Acceptance criteria

- wasm 7 tests / E2E 251、全ゲート成功（検証済み）。
