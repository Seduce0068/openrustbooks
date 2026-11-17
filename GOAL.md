# Goal

Markdown lint の auto-fix を実装するか、死設定として扱うか判断する（todo.md R1-ai / R2-ai#4）。

## Scope

### 背景

`core/src/ai_markdown/lint.rs` の `lint_and_fix()` は
`fixed_content = 入力` / `fixes_applied = 0` を返す no-op。
一方 `LintConfig.auto_fix` と AI pipeline の `PipelineConfig.auto_fix` は
設定として存在し、要件 REQ-010 は rumdl による自動整形を求めている。

### 成果物

- `core/src/ai_markdown/lint.rs`: 変更（rumdl fix API による auto-fix 実装）
- `core/src/ai_markdown/mod.rs`: 変更（`auto_fix` フラグの尊重、任意）
- `docs/decisions/0110-lint-auto-fix.md`: 追加
- `GOAL.md`: 本契約

調査結果: rumdl 0.3.2 に `document_run::DocumentRun::fix(max_iterations)` が
公開されており、新規依存なしで実装可能。REQ-010 が自動整形を求めるため
実装を選択する。

### 必須事項

1. `DocumentRun::fix(10)` で auto-fix を実行し、`fixed_content` と
   `fixes_applied = FixResult.rules_fixed` を返す。
2. 修正後に再 lint し、`passed` / `issues` を修正後の内容で作り直す。
3. `auto_fix` が false のときは修正せず lint のみ行う（フラグを尊重）。
4. 反復上限 10 を守り、エラーは `CoreError` に変換する。
5. テストを追加する。
6. 既存の 186 テスト、E2E 228 項目を壊さない。

## Acceptance criteria

- 6 ゲート・E2E が成功する。

## Constraints

- 新規依存を追加しない（rumdl は既存依存）。
- 同意ダイアログ・送信先制限・AI service refresh は変更しない。
- `master` へ push する。force push はしない。
