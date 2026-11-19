# Goal

未使用の Wasm AI cluster を削除し、ADR-0156 の該当部分を supersede する（R6 R2-crates#3）。

## Scope

### 背景

`WasmAiMarkdownService` / `WasmPipelineResult` / `WasmDiffResult` は
4 メソッドすべてが固定エラーを返す fail-closed スタブで、
追跡対象の web / E2E / wasm test 以外に参照が無い。
唯一の wasm test `ai_methods_fail_closed` は API 自体の契約を検証する
自己言及 test で、削除範囲に含める。
ADR-0156 の「JS 型互換のため残置」は実態と乖離している。

### 成果物

- `core/src/wasm.rs`: 変更（3 型・impl・test・import を削除）
- `docs/decisions/0256-wasm-ai-cluster-removal.md`: 追加
- ADR-0156 / 0227 / 0232 / 0243 / 0244 / 0246: 変更（supersede と参照更新）
- `todo.md`: R2-crates#3 の AI cluster 完了、#4 / #5 残存
- `GOAL.md` / `README.md` / `docs/index.md` / `docs/decisions/index.md` / `docs/log.md`: 更新

### 必須事項

1. 3 型と `ai_methods_fail_closed` test を属性ごと削除する。
2. lint cluster（`WasmMarkdownLinter` / `WasmLintIssue` / `WasmLintFixResult`）と
   `assert_js_error` helper は維持する。
3. ADR-0156 に「AI 部分のみ ADR-0256 に supersede、lint 部分は維持」と記録する。
4. ADR-0232 のブラウザ AI 契約を「API を公開しない」へ更新する。
5. 全ゲート成功（E2E 255 維持、wasm test は 15 → 14、core 284 維持）。

## Acceptance criteria

- wasm test 14 件、core 284 件維持、E2E 255、全ゲート成功。

## Constraints

- web / E2E / 公開 JS 契約の追跡ソースは変更しない（stale web/dist は対象外）。
- 外部 JS 利用者の有無は未決定として ADR に残す。
- `master` へ push する。force push はしない。
