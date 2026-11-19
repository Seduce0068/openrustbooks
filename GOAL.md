# Goal

未使用 Wasm API の第一段を削除した（R6 R2-crates#3、ADR-0243）。

## Scope

### 成果物

- `core/src/wasm.rs`: greet / detect_format を属性ごと削除
- `docs/decisions/0243-unused-wasm-api-removal.md`: 追加
- `todo.md`: R2-crates#3 を第一段完了・第二段以降未決定へ
- README / 各索引 / log: 更新

### 決定

- wasm-bindgen の export は pub と独立のため属性ごと削除。
- version / WasmDocument / AI cluster は維持。

## Acceptance criteria

- wasm 15 tests / E2E 254、全ゲート成功（検証済み）。
