# Goal

使用されていない trait メソッド `TokenStream::is_done` / `StorageBackend::available_space` を削除する（R6 TD-L02 / TD-L03）。

## Scope

### 背景

- `TokenStream::is_done()` は OpenAI / Ollama 実装が `false` を返すだけだったが、
  Ollama ストリーム修正（ADR-0189）で状態を持つようになり unit test のみが参照する。
- `StorageBackend::available_space()` は `Ok(None)` を返すだけで呼び出し側が存在しない。
  未実装 API を残すと利用者が誤って依存し得る。

### 成果物

- `core/src/ai_markdown/client.rs`: 変更（`is_done` 削除と関連テスト整理）
- `core/src/io.rs`: 変更（`available_space` 削除）
- `docs/decisions/0197-dead-trait-methods.md`: 追加
- `GOAL.md` / `todo.md`: 契約・完了反映

### 必須事項

1. `TokenStream` trait から `is_done()` を削除し、両実装とテスト参照を整理する。
2. `StorageBackend` trait から `available_space()` を削除し、実装を削除する。
3. 残存参照が 0 件になる（`grep -rn 'is_done\|available_space'`）。
4. 全ゲート成功（E2E 250 維持）。

## Acceptance criteria

- 全ゲート成功、E2E 250 項目成功。

## Constraints

- 既存の `next_token()` / ストリーム完了契約は維持する。
- 新規依存を追加しない。
- `master` へ push する。force push はしない。
