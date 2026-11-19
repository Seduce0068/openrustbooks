# Goal

E2E 件数の current 表記をコード出典（255）へ追従させる（R6 ドキュメント整合）。

## Scope

### 背景

`docs/e2e-traceability.md` と `tools/verify-web-reader.py` は
`Checker(expected=255)`（check 238 + equal 17）で一致している。
一方、未着手の `todo.md` の CI-H01 と
`docs/documentation-operations.md` の E2E 件数出自節には
「254 件」が current 値として残り、履歴根拠と現状が混在している。

### 成果物

- `todo.md`: CI-H01 の期待件数を 255 へ更新
- `docs/documentation-operations.md`: 255 件の根拠（ADR-0245）を追記し、254 は履歴として明示
- `docs/decisions/0262-e2e-count-documentation-refresh.md`: 追加
- `GOAL.md` / `README.md` / `docs/index.md` / `docs/decisions/index.md` / `docs/log.md`: 更新

### 必須事項

1. current 表記のみを 255 へ更新し、完了ログ / 過去 ADR の履歴的数値は書き換えない。
2. `Checker(expected=255)` と check 238 + equal 17 の内訳を根拠として記録する。
3. E2E check 名・件数・コードは変更しない。
4. markdownlint 0 issues、verify.sh（E2E 255）成功。

## Acceptance criteria

- 文書・索引が整合し、全ゲート成功。

## Constraints

- `tools/verify-web-reader.py` / `docs/e2e-traceability.md` / E2E check 名は変更しない。
- 過去の ADR / log / 完了済み TODO の履歴記述は変更しない。
- `master` へ push する。force push はしない。

## 完了記録

- todo CI-H01 と documentation-operations の current 値を 255（check 238 + equal 17）へ追従。
- 過去 ADR / log / 完了済み TODO の履歴 254 は保持。
- ADR-0262 で監査指摘（現状表記、ADR-0182 の役割）を修正。
- `./scripts/verify.sh`: 全ゲート成功、E2E 255 passed / 0 failed。
