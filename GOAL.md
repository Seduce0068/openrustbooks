# Goal

localStorage への保存失敗を利用者とログに可視化する（ADR-0151 の残タスク）。

## Scope

### 背景

`web/src/lib.rs` の `save_position()` / `save_settings()` / `save_history()` は
`window.local_storage()` や `set_item()` の失敗で早期 return または `let _` で捨てる。
ADR-0019 は容量超過時の縮退を決めているが、縮退後も失敗した場合の通知経路が無い。

### 成果物

- `web/src/lib.rs`: 変更（失敗時の status 表示と `web_sys::console::error_1`）
- `tools/verify-web-reader.py`: 変更（E2E チェック 1 件追加、232 → 233）
- `docs/decisions/0154-storage-failure-visibility.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. 3 つの save 関数が失敗した場合、`tracing::error!` または
   `web_sys::console::error_1` でログに残す。
2. 利用者向けには `set_status` 経由で「保存に失敗しました」系の文言を表示する
   （既存の status シグナルを再利用）。
3. E2E で localStorage の `setItem` を失敗させる（例: 容量を埋める／オーバーライド）
   シナリオを 1 件追加し、status とログを確認する。
4. 既存 E2E 232 項目と全ゲートを壊さない。

## Acceptance criteria

- E2E 233 項目が成功する。
- 6 ゲート + audit + wasm-test が成功する。

## Constraints

- 新規依存を追加しない。
- 保存スキーマとキー名は変更しない。
- `master` へ push する。force push はしない。
