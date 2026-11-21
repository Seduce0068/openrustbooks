---
type: Decision
title: "ADR-0291: E2E 件数ゲートの単一情報源化"
description: "E2E 期待件数を Checker(expected=N) のみに依存させ、todo.md の CI-H01 行をゲート条件から外す決定。"
tags:
  - ADR
  - CI
  - E2E
generated:
  by: dsh
  at: "2026-11-22T04:15:00+09:00"
status: accepted
---

## 背景

check-e2e-count-consistency.sh は todo.md の未着手 CI-H01 行を要求していたため、
CI-H01 を完了へ移すだけで CI が壊れた。
期待件数の正しい出典は tools/verify-web-reader.py の Checker(expected=N) である。

## 決定

- todo.md の CI-H01 行の有無・内容をゲート条件から削除する。
- 期待件数は Checker(expected=N) の AST 抽出のみを単一情報源とする。
- checker.check + checker.equal の実数一致は fail-closed で維持する。
- documentation-operations.md の current 表記は drift 検知用の
  fail-closed 補助検査として残し、drift 時はコード出典が正とする。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| Checker 単一情報源（採用） | TODO 状態と CI を分離できる |
| current 表記も廃止 | 文書 drift の早期検知を失うため却下 |
| 完了セクションも走査 | TODO 配置変更に結合し続けるため却下 |

## 検証（Runtime Proof）

- 負向 test: CI-H01 行を削除した一時 repo は **rc=0**。
- 負向 test: `Checker(expected=256)` を 257 へ改変した一時 repo は **rc=1**。
- 負向 test: checker.check を 1 減らすと **rc=1**（実装者報告）。
- `bash -n` / `shellcheck` / `shfmt -d`: rc=0。
- `./scripts/verify.sh`: 全ゲート成功、E2E **256 passed / 0 failed**。

## 未決定

- documentation-operations.md の補助検査を warning に格下げするか。
