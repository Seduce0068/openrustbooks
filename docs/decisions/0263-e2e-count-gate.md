---
type: Decision
title: "ADR-0263: E2E 期待件数の整合ゲート"
description: "Checker(expected=N) を唯一の出典として current 文書表記とのズレを fail-closed で検出する静的ゲートを記録する。"
tags:
  - ADR
  - e2e
  - ci
generated:
  by: dsh
  at: "2026-11-20T03:13:00+09:00"
status: accepted
---

## 背景

`tools/verify-web-reader.py` の `Checker(expected=255)` が E2E 期待総数の出典だが、
`todo.md` と `docs/documentation-operations.md` の current 表記は手運用で追従していた。
ADR-0262 は自動検出ゲートの要否を未決定として残していた。

## 決定

- `scripts/check-e2e-count-consistency.sh` を追加し、次を fail-closed で検査する:
  - Python AST で `Checker(expected=N)` が丁度 1 件・正の整数リテラルであること。
  - `checker.check` + `checker.equal` の第 1 引数文字列リテラル数が N と一致すること。
  - check 名に重複が無いこと（ADR-0182）。
  - `todo.md` の未着手 `CI-H01` 行と
    `docs/documentation-operations.md` の current 値節が N と一致すること。
- 履歴（254 / 253 など）は完了済み TODO / log / 過去 ADR に保持し、
  ゲートの対象外とする。
- `verify.sh` の version-consistency 直後と
  `.github/workflows/ci.yml` の `extra_gates` から同じ script を呼ぶ。
- 共通ワークフロー `rust-gates.yml@v1` は変更しない。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 静的ゲートを追加（採用） | 手運用のズレを fail-closed で検出できる |
| ADR 追従の手運用のみ | 件数変更時に current 表記が古くなる（ADR-0262 の課題） |
| 横断テキスト走査 | 履歴 254 / 253 を誤検出するため却下 |

## 影響

- 静的ゲートが 1 つ増える（verify.sh / ci.yml に 1 行ずつ）。
- E2E check 名・件数・`docs/e2e-traceability.md`・Cargo.* は不変（255 維持）。
- 負向 test 15 ケース（expected 変更 / 複数 / 非リテラル / 0 /
  内訳不一致 / 重複名 / current 不一致・欠落・複数 / ファイル欠落 /
  陽性対照）で fail-closed を確認した。

## 検証（Runtime Proof）

- `bash -n` / `shellcheck`: OK、ci.yml YAML パース OK。
- 通常 run: `OK: E2E 期待件数 255 件 / check+equal 255 件`、rc=0。
- 負向 test **15/15 PASS**。
- `./scripts/verify.sh`: 新ゲート rc=0、全ゲート成功、
  E2E **255 passed / 0 failed**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- README / index / ADR 本文 / log の数値まで検査対象に広げるか
  （現在は todo CI-H01 と documentation-operations の 2 アンカー固定）。
- ADR 内数値を「決定時点の履歴」として扱う慣例の規約化（ADR-0262 を継続）。
- 実 CI 上でのゲート発火確認（billing 解消後、CI-H01 を継続）。
