# Goal

圧縮率依存の assertion を決定的な方式／フィールド検証へ置換した（R6 R4-unit#4、ADR-0238）。

## Scope

### 成果物

- `core/src/archive.rs` / `core/src/wasm.rs`: 方式と `compressed_size` の直接検証へ置換
- `docs/decisions/0238-deterministic-compression-tests.md`: 追加
- `todo.md`: R4-unit#4 を完了へ移動

### 決定

- 比率 assertion を削除し、`ZipFile::compression()` と `compressed_size()` で直接検証。
- 既存の内容一致 test は維持、本番コード / 公開 API は不変。

## Acceptance criteria

- core 270 / wasm 14 / E2E 254、全ゲート成功（検証済み）。
