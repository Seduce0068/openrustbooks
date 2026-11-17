# Goal

内蔵プロンプトテンプレートに `system_prompt` を追加し、AI サービスの初期化失敗を修正する（ADR-0107 の未決定）。

## Scope

### 背景

`core/src/ai_markdown/prompt.rs:41` の `PromptTemplate` は `system_prompt: String` を必須とする。
一方 `core/prompts/*.hbs` 全 7 ファイルの TOML front matter には `system_prompt` が無く、
`parse_template()` の `toml::from_str` が `missing field system_prompt` で失敗する。
結果、native の `AiMarkdownService::new()` は設定に関わらず常に失敗する。

### 成果物

- `core/prompts/*.hbs`: 変更（7 ファイルに `system_prompt` を追加）
- `docs/decisions/0108-prompt-system-prompt.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. 全 7 テンプレートの front matter に `system_prompt` を追加する。
   - 内容は既存の `client.rs` の固定 system message
     （"You are a Markdown expert. ..."）と同等の意味にする。
2. `PromptRegistry::with_builtins()` が成功することをテストで確認する。
3. `AiMarkdownService::new()` が AI 有効時に成功することをテストで確認する。
4. 既存の 179 テスト、E2E 228 項目を壊さない。

## Acceptance criteria

- 6 ゲート・E2E が成功する。

## Constraints

- 新規依存を追加しない。
- テンプレート本文（ユーザープロンプト部分）は変更しない。
- `master` へ push する。force push はしない。
