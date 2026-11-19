# Goal

未使用の Wasm API `utils::greet` / `utils::detect_format` を削除する（R6 R2-crates#3 の第一段）。

## Scope

### 背景

`core/src/wasm.rs` の `utils::greet()` / `utils::detect_format()` は
JavaScript へ export されているが、web / wasm test / ADR のいずれからも参照されていない。
`#[wasm_bindgen]` は Rust の `pub` と独立して JS export を生成するため、
属性ごと削除する必要がある。
AI cluster（WasmAiMarkdownService 等）は ADR-0156 の公開決定があるため本次対象外。

### 成果物

- `core/src/wasm.rs`: 変更（2 関数と wasm_bindgen 属性を削除）
- `docs/decisions/0243-unused-wasm-api-removal.md`: 追加
- `todo.md`: R2-crates#3 を「第一段完了、第二段以降は未決定」へ更新
- `GOAL.md` / `README.md` / `docs/index.md` / `docs/decisions/index.md` / `docs/log.md`: 更新

### 必須事項

1. `greet()` / `detect_format()` を属性ごと削除し、JS export を減らす。
2. `utils::version()` と native 側の対称性は維持する。
3. `WasmDocument` / AI cluster は本次で変更しない（ADR 契約と段階案を維持）。
4. 全ゲート成功（E2E 254 維持）。

## Acceptance criteria

- wasm check / wasm test / verify.sh 成功、E2E 254 維持。

## Constraints

- ADR-0156 / 0160 / 0223 の公開決定は変更しない。
- 新規依存を追加しない。
- `master` へ push する。force push はしない。
