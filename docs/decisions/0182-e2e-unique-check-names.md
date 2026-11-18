---
type: Decision
title: "ADR-0182: E2E check 名の一意性検証"
description: "Checker に check 名の重複検知を追加し、総数ゲートの形骸化を防ぐ決定を記録する。"
tags:
  - ADR
  - testing
  - e2e
generated:
  by: dsh
  at: "2026-11-19T04:43:00+09:00"
status: accepted
---

## 背景

ADR-0090 の期待総数ゲートは `passed + failures == expected` のみを検証するため、
同名 check を二重に呼んで期待総数を埋めても失敗しなかった。
要件から E2E 項目が消えてもゲートが通る形骸化リスクがあった（R6: TD-M03）。

## 決定

- `Checker` に `names: set[str]` を追加する。
- `check()` の先頭で名前の重複を完全一致で検知する:
  - 重複時: `failures` に `"<name> (duplicate check name)"` を記録し、
    `FAIL <name> (duplicate check name)` を出力して `passed` を増やさない。
  - 新規時: `names.add(name)` して従来どおり成功／失敗を処理する。
- `equal()` は `check()` を経由するため、同じ名前空間で検知される。
- 期待総数は **249 のまま**。重複検知は追加 check ではなく不正検出である。
- 既存 249 件の check 名はすべて一意で、名前変更は不要。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 完全一致の重複検知（採用） | 最小変更で形骸化を防げる |
| 正規化（大小文字・空白） | 表記ゆれの定義が必要で本次範囲外 |
| 期待リストとの照合 | 保守コストが高い |

## 影響

- 同名 check が 1 回余分に呼ばれると `executed` が期待総数を超え得るが、
  `finish()` は `rc=1` を返すためゲートは落ちる。
- 1 インスタンス前提（`run_checks()` は Checker を 1 つだけ生成）。
- 新規依存なし。

## 検証（Runtime Proof）

- AST 静的解析: `checker.check` 232 + `checker.equal` 17 = 249、
  第一引数はすべて文字列リテラル、ユニーク 249、重複 0。
- 一時スクリプトで重複名を渡すと
  `FAIL A (duplicate check name)` が出力され、
  `passed` が増えず `failures` に記録されることを確認。
- `python3 -m py_compile`、`./scripts/verify.sh`: 成功。
- E2E **249 項目すべて成功**、`duplicate check name` の出現 0 件。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- 表記ゆれ（大小写・前後空白・全角半角）の正規化要否。
- `failures` 文字列形式の変更要否。
- Checker を複数インスタンスに分割する場合の名前空間設計。
