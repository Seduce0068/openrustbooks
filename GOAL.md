# Goal

EditorConfig / ReaderConfig の死設定を正直化する（R6 TD-M19、ADR-0217）。

## Scope

### 成果物

- `web/src/lib.rs`: Editor / Reader セクションへ未接続注記
- `docs/decisions/0217-editor-reader-dead-config.md`: 追加
- `todo.md`: TD-M19 を完了へ移動

### 決定

- DTO / config.toml 互換を維持し、フィールドは削除しない。
- 実動作の権威（localStorage / 下書き）を注記で明示。

## Acceptance criteria

- E2E 253 項目、全ゲート成功（検証済み）。
