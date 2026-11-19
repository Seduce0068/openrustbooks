# Goal

正規化後同名 ZIP エントリを fail-closed で拒否した（R6 R1-archive#4、ADR-0240）。

## Scope

### 成果物

- `core/src/archive.rs`: index.insert + 重複検出
- `core/src/epub.rs`: mimetype 重複の Validation test
- `core/src/wasm.rs`: WasmArchive の重複拒否 test
- `docs/decisions/0240-duplicate-archive-entries.md`: 追加
- `todo.md` / README / 各索引 / log: 更新

### 決定

- 全生成経路が収束する from_owned の 1 箇所で拒否。
- 正規化等価 5 パターンを test、完全一致 raw 名は未対応として ADR に明記。
- 公開 API / Send + Sync は不変。

## Acceptance criteria

- core 274 / wasm 15 / E2E 254、全ゲート成功（検証済み）。
