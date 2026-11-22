# Goal

Wasm JS 境界の CoreError を redaction する（第2cycle R6 / R1-DS01）。

## Scope

### 背景

`core/src/wasm.rs::js_error()` と `core/src/wasm/*.rs` の一部が
`CoreError::to_string()` を JS へ返す。href / パス / 内部型名を含む生文字列が
公開 Wasm API から読める。desktop IPC は code 由来の固定文言へ統一済み。

### 成果物

- `core/src/error.rs`: 共有 code→固定文言 mapper
- `core/src/wasm.rs`, `core/src/wasm/*.rs`: 変更
- `desktop/src/commands/mod.rs`: 共有 mapper を利用
- `docs/decisions/0323-wasm-error-redaction.md`: 追加
- `todo.md` / README / 索引 / log: 更新

### 必須事項

1. CoreError::code() から固定文言を返す共有 mapper を core に置く。
2. Wasm の全 CoreError 変換を共有 mapper 経由にする。
3. serde_wasm_bindgen エラーなど非 CoreError も固定文言にする。
4. desktop の ipc_message は共有 mapper へ委譲し、既存文言を維持する。
5. wasm / core / desktop test と E2E 256 件を維持。
6. verify.sh 全ゲート成功。

## Constraints

- 既存の公開 API 名・シグネチャは変更しない。
- 外部サービス / 秘密情報は扱わない。
- `master` へ push する。force push はしない。
