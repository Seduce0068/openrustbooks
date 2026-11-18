# Goal

E2E Checker に check 名の一意性検証を追加する（R6 TD-M03）。

## Scope

### 背景

`finish()` は `passed + failures == expected` のみを検証するため、
同名 check を二重に呼んで期待総数を埋めても失敗しない。
要件から E2E 項目が消えてもゲートが通る形骸化リスクがある。

### 成果物

- `tools/verify-web-reader.py`: 変更（Checker に names 集合と重複検知を追加）
- `docs/decisions/0182-e2e-unique-check-names.md`: 追加
- `GOAL.md` / `todo.md`: 契約・完了反映

### 必須事項

1. `Checker.check()` で name の重複を検知し、failures に記録する。
2. 重複時は `passed` を増やさず、`FAIL <name> (duplicate check name)` を出力する。
3. 既存 249 件の check 名がすべて一意であることを静的／動的に確認する。
4. 期待総数は 249 のまま（重複検知は追加 check ではなく不正検出）。
5. 全ゲート成功。

## Acceptance criteria

- E2E 249 項目がすべて一意で、全ゲート成功。

## Constraints

- 新規依存を追加しない。
- 既存 check 名・件数は変更しない（必要な場合のみ明確な理由を記録）。
- `master` へ push する。force push はしない。
