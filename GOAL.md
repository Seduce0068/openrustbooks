# Goal

AI 死設定（月間上限 / timeout）を設計受容として正直化する（R6 R1-ai#2,#5 / R2-ai#3,#4）。

## Scope

### 背景

監査の結果、`max_retries` は配線済み、`streaming` / `check_updates` は
ADR-0216 / 0222 で設計受容・「未配線」表示済みだった。
残る実質課題は `max_tokens_per_month` で、
設定値が IPC DTO と「月間上限: 値」表示に現れるだけで、
使用量の集計・保存・超過判定は存在しない。
`timeout_secs` は Ollama の reqwest timeout にのみ配線され、
OpenAI 側には渡っていない。

### 成果物

- `web/src/lib.rs`: 変更（月間上限を「未配線」、timeout を「Ollama のみ」と表示）
- `docs/decisions/0228-ai-dead-config.md`: 追加
- `GOAL.md` / `todo.md`: 契約・完了反映
- `README.md` / `docs/index.md` / `docs/decisions/index.md` / `docs/log.md`: 更新

### 必須事項

1. 設定画面の「月間上限: 値」を「未配線」固定へ変更する。
2. 「タイムアウト: Ns」を「タイムアウト: Ns（Ollama のみ）」へ変更する。
3. config.toml / DTO / E2E 総数（254）は変更しない。
4. ADR に配線状況の表と未決定（月の定義、計測対象、保存先、wasm、超過時挙動、OpenAI timeout）を記載する。
5. 全ゲート成功。

## Acceptance criteria

- E2E 254 項目、全ゲート成功。

## Constraints

- コードの設定フィールドは削除しない（互換維持）。
- 新規依存を追加しない。
- `master` へ push する。force push はしない。
