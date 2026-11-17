# Goal

Markdown lint の auto-fix を実装するか、死設定として扱うか判断する（todo.md R1-ai / R2-ai#4）。

## Scope

### 背景

`core/src/ai_markdown/lint.rs` の `lint_and_fix()` は
`fixed_content = 入力` / `fixes_applied = 0` を返す no-op。
一方 `LintConfig.auto_fix` と AI pipeline の `PipelineConfig.auto_fix` は
設定として存在し、要件 REQ-010 は rumdl による自動整形を求めている。

### 成果物

- `core/src/ai_markdown/lint.rs`: 変更（auto-fix 実装または明確な拒否／コメント）
- `docs/decisions/0110-lint-auto-fix.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. rumdl のローカルクレートを調査し、auto-fix API の有無を確認する。
2. API があれば安全に適用して `fixes_applied` を返す。
3. API が無ければ no-op の理由を明記し、設定（`auto_fix`）との関係を整理する。
4. テストを追加する。
5. 既存の 186 テスト、E2E 228 項目を壊さない。

## Acceptance criteria

- 6 ゲート・E2E が成功する。

## Constraints

- 新規依存を追加しない（rumdl は既存依存）。
- 同意ダイアログ・送信先制限・AI service refresh は変更しない。
- `master` へ push する。force push はしない。
