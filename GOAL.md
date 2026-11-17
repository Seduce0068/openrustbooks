# Goal

`WasmConfig` の AI 既定値を `AiConfig` と整合させる（ADR-0099 / 0105 の未決定）。

## Scope

### 背景

`core/src/wasm.rs` の `WasmConfig::default()` は `ai_enabled: true` だが、
native の `AiConfig::default()` は ADR-0099 で `enabled: false` に変更済み。
Wasm の AI 実装はスタブで送信はないが、既定値の不整合は将来の
fetch クライアント実装時に「既定で送信する」事故につながる。

### 成果物

- `core/src/wasm.rs`: 変更（`ai_enabled: false`）
- `docs/decisions/0117-wasm-config-alignment.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. `WasmConfig::default().ai_enabled` を false に変更する。
2. 既存の getter / setter は維持する。
3. テストを追加する（既定 false、setter で true に変更可能）。
4. 既存の 195 テスト、E2E 228 項目を壊さない。

## Acceptance criteria

- 6 ゲート・E2E が成功する。

## Constraints

- 新規依存を追加しない。
- Wasm の AI スタブ実装自体は変更しない。
- `master` へ push する。force push はしない。
