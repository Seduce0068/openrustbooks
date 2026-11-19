# Goal

未使用の Wasm API `WasmDocument` を削除する（R6 R2-crates#3 第二段）。

## Scope

### 背景

`core/src/wasm.rs` の `WasmDocument` は JavaScript へ export されているが、
`web/src` / wasm unit test / ADR 決定から参照されていない。
ADR-0243 では第一段（greet / detect_format）完了後に
第二段候補として明示されていた。
`#[wasm_bindgen]` は `pub` と独立に JS export を生成するため、
属性ごと削除する必要がある。

### 成果物

- `core/src/wasm.rs`: 変更（`WasmDocument` struct / impl を削除）
- `docs/decisions/0244-wasm-document-removal.md`: 追加
- `todo.md`: R2-crates#3 第二段完了、AI cluster は未決定へ
- `GOAL.md` / `README.md` / `docs/index.md` / `docs/decisions/index.md` / `docs/log.md`: 更新

### 必須事項

1. `WasmDocument` とその `#[wasm_bindgen] impl` を属性ごと削除する。
2. AI cluster / `WasmEpubBook` / `WasmArchive` / lint API / `version()` は変更しない。
3. 削除後も wasm32 core / web check、wasm test、E2E 254 が成功する。
4. ADR に外部 JS 互換と AI cluster の未決定を記録する。

## Acceptance criteria

- wasm check / wasm test / verify.sh 成功、E2E 254 維持。

## Constraints

- ADR-0156 / 0160 / 0223 / 0243 の公開契約は上書きしない。
- 新規依存を追加しない。
- `master` へ push する。force push はしない。
