# Goal

未使用 Wasm API の第二段を削除した（R6 R2-crates#3、ADR-0244）。

## Scope

### 成果物

- `core/src/wasm.rs`: WasmDocument を属性ごと削除
- `docs/decisions/0244-wasm-document-removal.md`: 追加
- `todo.md`: 第二段完了、AI cluster は未決定
- README / 各索引 / log: 更新

### 決定

- wasm-bindgen export を実質的に減らすため属性ごと削除。
- serde import は他 API が使用するため維持。
- AI cluster / EPUB / Archive / lint / version は不変。

## Acceptance criteria

- wasm 15 tests / E2E 254、全ゲート成功（検証済み）。
