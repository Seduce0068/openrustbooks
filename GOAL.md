# Goal

Document 側の dead な EPUB 型を削除した（R6 R2-epub#4、ADR-0250）。

## Scope

### 成果物

- `core/src/document.rs`: DocumentContent::Epub / 3 struct を削除
- `docs/decisions/0250-document-epub-type-removal.md`: 追加
- `todo.md`: R2-epub#4 完了、#2 残存
- README / 各索引 / log: 更新

### 決定

- EPUB は EpubBook / WasmEpubBook 経路へ一本化。
- 他 variant / DocumentFormat / epub.rs / desktop / web / wasm は不変。
- 外部 Rust / serde 利用者の有無は未決定。

## Acceptance criteria

- core 279 / E2E 255、全ゲート成功（検証済み）。
