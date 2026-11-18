# Goal

未使用 trait メソッド TokenStream::is_done / StorageBackend::available_space を削除する（R6 TD-L02 / L03、ADR-0197）。

## Scope

### 成果物

- `core/src/ai_markdown/client.rs`: is_done 削除、next_token シーケンスで回帰担保
- `core/src/io.rs`: available_space 削除
- `docs/decisions/0197-dead-trait-methods.md`: 追加
- `todo.md`: TD-L02 / L03 を完了へ移動

### 決定

- 両方の trait メソッドと実装を削除。
- ADR-0189 のキュー優先配信は next_token シーケンスで検証。
- 残存参照 0 件。

## Acceptance criteria

- E2E 250 項目、全ゲート成功（検証済み）。
