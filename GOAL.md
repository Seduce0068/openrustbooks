# Goal

ZIP リーダーを wasm32 API として公開する（R6 TD-M22 段階 1、ADR-0223）。

## Scope

### 成果物

- `core/src/wasm.rs`: WasmArchive（new / entries / read_entry / read_entry_text / is_epub）、固定エラー、unit 7 件
- `docs/decisions/0223-zip-wasm-api.md`: 追加
- `todo.md`: TD-M22 を段階 1 完了・UI は未決定へ

### 決定

- ArchiveReader::from_owned で所有権ごと渡す。
- エラーは固定文言、js_error を流用しない。
- UI / CSP は変更せず段階 2 の別 ADR に委ねる。

## Acceptance criteria

- wasm 14 tests / E2E 253、全ゲート成功（検証済み）。
