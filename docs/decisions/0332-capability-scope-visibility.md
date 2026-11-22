---
type: Decision
title: "ADR-0332: capability ゲートの検査範囲を明示"
description: "check-capability-scope.sh は web 側ソース検査であり、実 WebView の権限制限ではないことを明示する決定。"
tags:
  - ADR
  - security
  - IPC
generated:
  by: dsh
  at: "2026-11-23T00:57:00+09:00"
status: accepted
---

## 背景

check-capability-scope.sh は web 側ソースの invoke 呼び出しを
4 command に限定していたが、withGlobalTauri=true の実 WebView では
invoke_handler に登録された全 command へ到達可能だった。
ゲートの成功文言が実効権限位を誤解させる恐れがあった。

## 決定

- ゲート冒頭コメントに次を明記する:
  - これは web 側ソースの静的検査であり runtime capability / ACL ではない。
  - withGlobalTauri=true では invoke_handler 登録済みの全 command
    （fs / document / ai / window を含む）へ実 WebView から到達可能。
- 成功文言を「web 側ソースの設定 command 呼び出しは 4 種に限定」
  （source inspection のみ）へ変更する。
- 実効権限を制限する ACL 導入は今回の範囲外で、未決定として残す。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 成功文言を実態へ修正（採用） | 誤解を防ぎ、最小変更で済む |
| capability / ACL を導入 | 設定 UI の IPC 経路を変える必要があり範囲外 |
| 現状維持 | 実効権限が 4 command と誤解される |

## 検証（Runtime Proof）

- bash -n / shellcheck: rc=0。
- check-capability-scope.sh: rc=0（新しい成功文言を確認）。
- `./scripts/verify.sh`: 全ゲート成功、E2E **263 passed / 0 failed**、WDIO gate rc=0。

## 未決定

- withGlobalTauri を false に戻すか、capability / ACL で command 単位に制限するか。
