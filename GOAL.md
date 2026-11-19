# Goal

ZIP リーダーを wasm32 向け API として公開する（R6 TD-M22 の段階 1）。

## Scope

### 背景

`core/src/archive.rs` の `ArchiveReader` は fail-closed のパス検証、
サイズ制限、不正エントリ拒否を備えて native で完成している。
一方 wasm 側は `detect_format()` が `"zip"` を返すだけで、
エントリ一覧／読み出し API が無い（TD-M22）。
UI は XSS / Blob URL / CSP の未決定が多いため今回は対象外。

### 成果物

- `core/src/wasm.rs`: 変更（`WasmArchive` と wasm-bindgen API、test）
- `docs/decisions/0223-zip-wasm-api.md`: 追加
- `GOAL.md` / `todo.md`: 契約・完了反映

### 必須事項

1. `cfg(target_arch = "wasm32")` で `WasmArchive` を追加する。
2. コンストラクタは `Vec<u8>` を `ArchiveReader::from_owned(bytes, ArchiveLimits::default())` へ渡す。
3. API は `entries()` / `read_entry(path)` / `read_entry_text(path)` / `is_epub()` に限定する。
4. `entries()` は camelCase（`path` / `isDir` / `size` / `compressedSize`）で返す。
5. 各 API は core エラーを fail-closed で JsValue 化する（値・パスはエラー文言に出さない）。
6. wasm-bindgen-test を追加（正常、traversal、サイズ制限、破損）。
7. 全ゲート成功（E2E 253 維持）。

## Acceptance criteria

- wasm test 増加、全ゲート成功、E2E 253 項目成功。

## Constraints

- UI / File input / CSP は変更しない（段階 2 の別 ADR）。
- 新規依存を追加しない。
- `master` へ push する。force push はしない。
