# Goal

AI 死設定（月間上限 / timeout）を設計受容として正直化する（R6 R1-ai#2,#5 / R2-ai#3,#4、ADR-0228）。

## Scope

### 成果物

- `web/src/lib.rs`: 月間上限を「未配線」、timeout を「Ollama のみ」表示
- `docs/decisions/0228-ai-dead-config.md`: 追加
- `todo.md`: R1-ai#2/#5 / R2-ai#3/#4 を完了へ移動
- README / docs index / decisions index / log: 更新

### 決定

- core / config.toml / desktop DTO は不変。
- Web ローカル view のみ「未配線」表示へ変更。
- 月次使用量ストアは別タスクの未決定。

## Acceptance criteria

- E2E 254 項目、全ゲート成功（検証済み）。
