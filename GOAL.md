# Goal

圧縮率依存の assertion を決定的な方式／フィールド検証へ置換する（R6 R4-unit#4）。

## Scope

### 背景

`core/src/archive.rs:440` と `core/src/wasm.rs:984` は
`compressed_size < size` という deflate の圧縮率に依存する assertion を持つ。
deflate の出力サイズは仕様で保証されず、
backend / バージョン変更で壊れ得る。
方式そのものは `ZipFile::compression()` で直接検証でき、
`read_entry` の動作は既存の内容一致 test が担保する。

### 成果物

- `core/src/archive.rs`: 変更（方式と `compressed_size` の直接検証）
- `core/src/wasm.rs`: 変更（wasm test の同置換）
- `docs/decisions/0238-deterministic-compression-tests.md`: 追加
- `GOAL.md` / `todo.md`: 契約・完了反映

### 必須事項

1. 比率 assertion を `ZipFile::compression() == CompressionMethod::Deflated` と
   `compressed_size == ZipFile::compressed_size()` へ置換する。
2. `archive.rs` / `wasm.rs` の両方を対象にする。
3. 既存の内容一致 test を維持する（`read_entry` の動作はそちらで担保）。
4. 本番コード・公開 API は変更しない。
5. 全ゲート成功（E2E 254 維持）。

## Acceptance criteria

- 全ゲート成功、E2E 254 項目成功。

## Constraints

- `ArchiveEntry` / wasm DTO にフィールドを追加しない。
- 新規依存を追加しない（既存 `zip` crate の公開 API を使用）。
- `master` へ push する。force push はしない。
