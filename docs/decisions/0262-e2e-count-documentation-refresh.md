---
type: Decision
title: "ADR-0262: E2E 件数 current 表記のコード出典追従"
description: "E2E 総数の current 表記をコード出典（255）へ追従させ、履歴的数値は保持する決定を記録する。"
tags:
  - ADR
  - documentation
  - e2e
  - traceability
generated:
  by: dsh
  at: "2026-11-20T02:56:36+09:00"
status: accepted
---

## 背景（更新前の現状）

- **コード**: `tools/verify-web-reader.py:887` の `Checker(expected=255)` が期待総数。
  Python AST 抽出で `checker.check` 238 + `checker.equal` 17 = 255、同名重複 0。
- **トレーサビリティ**: `docs/e2e-traceability.md:29,87` は期待総数 255 と
  割り当て 255 件（check 238 + equal 17）で一致していた。
- **current 表記の残存**: `todo.md` の CI-H01 と
  `docs/documentation-operations.md` の E2E 総数出自節に
  「254 件」が current 値として残り、履歴根拠と現状が混在していた。
  完了済み TODO / 過去 ADR / log の 254 は履歴として正しい。

## 決定

- E2E 総数の current 表記は **255 件**（check 238 + equal 17）とする。
- 出典はコードの `Checker(expected=255)` のみとし、文書側で手計算しない
  （ADR-0090 を継承）。
- `todo.md` の CI-H01 と `docs/documentation-operations.md` の出自節を 255 へ追従する。
- 254 件（check 237 + equal 17）と 253 件は ADR-0225 / ADR-0220 の履歴更新として
  保持し、削除・書き換えない。
- E2E check 名・件数・コード・`docs/e2e-traceability.md` は変更しない。

## 根拠

- ADR-0090: 期待総数は常にコード（`Checker(expected=...)`）を出典とする。
- ADR-0182: 同名 check の重複を検知する。欠落検知は ADR-0090 の期待総数ゲートの役割で、
  両者を合わせて 255 件の内訳の妥当性を担保する。
- ADR-0220: 253 件への追従（2 件を VC-004 / VC-009 へ割り当て）。
- ADR-0225: 254 件（check 237 + equal 17）への更新。履歴として保持する。
- ADR-0245: 255 件（check 238 + equal 17）への更新。current 値の根拠。

## 未決定

- 今後 E2E 件数が変更されたとき、文書側の current 表記のズレを自動検出する
  CI ゲートの要否は [ADR-0263](0263-e2e-count-gate.md) で採用した
  （`scripts/check-e2e-count-consistency.sh` を verify.sh / CI へ配線）。
- ADR 内に記載した数値を「決定時点の履歴」として扱う慣例の明文化
  （本 ADR では 254 / 253 を履歴として扱うが、規約化は未実施）。

## 影響

- 文書 / 索引 / log の追従のみ。コード / E2E check / E2E 件数は不変（255 維持）。
- ADR 総数は 262 件（ADR-0261 の次）。

## 検証

- AST 抽出: `checker.check` 238 + `checker.equal` 17 = 255、同名重複 0。
- `npx markdownlint-cli2 'docs/**/*.md' README.md todo.md`: 0 issues。

## 参照

- [ADR-0090: E2E 期待総数のコード出典](0090-e2e-expected-count.md)
- [ADR-0182: E2E check 名の一意性](0182-e2e-unique-check-names.md)
- [ADR-0220: トレーサビリティの 253 件への更新](0220-traceability-refresh.md)
- [ADR-0225: 壊れた画像のセトリング待ち解消](0225-broken-image-settling.md)
- [ADR-0245: 容量超過時の位置削除通知](0245-position-prune-notification.md)
- [E2E トレーサビリティ](../e2e-traceability.md)
