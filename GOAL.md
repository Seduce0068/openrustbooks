# Goal

AI プロンプトの system 指示をテンプレート側に一元化する（ADR-0108 の未決定）。

## Scope

### 背景

`client.rs` の `complete()` / `stream_complete()` は OpenAI に固定 system message
"You are a Markdown expert..." を送る一方、`render()` はテンプレートの
`system_prompt` をユーザーメッセージ先頭に挿入するため、意味的に二重になる。
Ollama には `client.rs` に system 経路が無く、テンプレート側が唯一の指示経路。

### 成果物

- `core/src/ai_markdown/client.rs`: 変更（固定 system message の削除）
- `docs/decisions/0109-system-prompt-unification.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. `OpenAiClient::complete()` / `stream_complete()` から固定 system message を削除し、
   `prompt` のみを user message として送る。
2. Ollama 側のペイロードは変更しない（テンプレートの system_prompt が維持される）。
3. `AiClient` トレイト・`complete()` シグネチャは変更しない。
4. 境界テストを追加する（モック AiClient に渡る prompt に system_prompt が含まれ、
   OpenAI の messages に system role が無いこと）。
5. 既存の 183 テスト、E2E 228 項目を壊さない。

## Acceptance criteria

- 6 ゲート・E2E が成功する。

## Constraints

- 新規依存を追加しない。
-テンプレート本文・`render()` は変更しない。
- `master` へ push する。force push はしない。
