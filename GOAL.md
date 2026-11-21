# Goal

lint フィードバックが no-op な要求の無駄な AI リトライを停止する（R6 レビュー / R2-ER01）。

## Scope

### 背景

TableRequest / FrontMatterRequest の add_lint_feedback は空実装で、
pipeline は lint 不合格のたびに同一 prompt / current_content で AI を再呼び出しする。
最大 10+1 回の外部呼び出しが無駄に発生し、課金・レート制限を消費する。

### 成果物

- `core/src/ai_markdown/mod.rs`: 変更（lint feedback 対応の有無を公開）
- `core/src/ai_markdown/pipeline.rs`: 変更（no-op 要求は即時 lint_passed=false）
- `docs/decisions/0293-lint-retry-feedback.md`: 追加
- `todo.md` / README / 索引 / log: 更新

### 必須事項

1. lint feedback を適用できない要求種別は、初回 lint 失敗時にリトライしない。
2. 既存の lint feedback を持つ要求のリトライ動作は壊さない。
3. 単体 test で AI 呼び出し回数が 1 回に留まることを検証する。
4. verify.sh 全ゲート成功、E2E 256 件・WDIO 維持。

## Constraints

- 外部 AI API への送信は行わない。
- prompt / 既存成功契約を壊さない。
- `master` へ push する。force push はしない。

## 完了記録

- Table/FrontMatter の no-op lint feedback で初回失敗時にリトライ停止。
- execute_pipeline 内で max_retries を 10 以下へ clamp、usize::MAX も安全。
- core 318 tests、verify.sh 全ゲート成功（E2E 256、WDIO 1）。
