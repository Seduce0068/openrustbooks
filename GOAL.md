# Goal

AI の送信先 `base_url` を許可リストで制限する（ADR-0099/0100 の未決定、baseurl-auditor 推奨）。

## Scope

### 背景

`OpenAiConfig.base_url` / `OllamaConfig.base_url` は単なる String で、
Tauri IPC の `update_config` / `update_ai_config` から JS が任意 URL
（例: `http://attacker.example`）に変更して有効化できる。Rust 側 HTTP は
WebView の CSP 対象外のため、同意なく本文が外部へ送られるリスクがある。

### 成果物

- `core/src/ai_markdown/client.rs`: 変更（送信先検証関数と適用）
- `desktop/src/commands/config.rs`: 変更（設定更新時の検証）
- `docs/decisions/0101-base-url-allowlist.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. 既定ホストのみを許可する。
   - OpenAI: `https://api.openai.com/v1`
   - Ollama: `http://localhost:11434`（loopback http は許可）
2. scheme は https または loopback の http のみ許可する。
3. 今回は既定ホスト＋loopback のみをハードコードで許可する。
   カスタムエンドポイントの opt-in は、IPC が `AppConfig` 全体を受け取るため
   JS から書き換えられない設計が必要で、将来課題とする。
4. IPC 設定更新時とクライアント生成時の二重で検証する（defense in depth）。
5. 新規依存を追加しない（`url` クレートは既存依存）。
6. 既存の 146 テスト、E2E 228 項目を壊さない。

## Acceptance criteria

- 6 ゲート・E2E が成功する。

## Constraints

- Web 版の wasm スタブには影響しない。
- 同意ダイアログの UI は今回行わない。
- `master` へ push する。force push はしない。
