# Goal

AI pipeline の最終試行で lint 不合格をどう扱うか判断する（ADR-0111 の未決定）。

## Scope

### 背景

`execute_pipeline()` は `lint_result.passed || attempt == max_retries` で
ループを抜け、`passed=false` でも `Ok(PipelineExecutionResult)` を返す。
利用者／UI が `lint_passed` を見なければ不合格文書が確定する。
req-010 は「失敗時は差分提示しユーザー判断へ」を求めている。

### 成果物

- `docs/decisions/0116-final-lint-policy.md`: 追加（調査と判断）
- `GOAL.md`: 本契約

### 必須事項

1. 現状の挙動と呼び出し側（desktop コマンド、Result 型の `lint_passed`）を整理する。
2. `Ok` 維持／`Err` 化／オプトインの選択肢を比較する。
3. 判断と根拠を記録する（コード変更は行わない）。

## Acceptance criteria

- 判断が ADR に記録される。
- markdownlint が通る。

## Constraints

- コードを変更しない（調査と決定のみ）。
- `master` へ push する。force push はしない。
