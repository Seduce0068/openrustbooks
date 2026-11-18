---
type: Decision
title: "ADR-0183: wait_until の例外伝播"
description: "E2E 待機ヘルパーで握り潰していた Python 例外を再送出する決定を記録する。"
tags:
  - ADR
  - testing
  - e2e
generated:
  by: dsh
  at: "2026-11-19T04:55:00+09:00"
status: accepted
---

## 背景

`wait_until()` は `RuntimeError`（CDP 無応答）のみ再送出し、
その他の Python 例外を `pass` で無視していた（R6: TD-M02）。
Python 例外（WebSocket `ConnectionClosed` / `JSONDecodeError` 等）が
タイムアウトまで隠れ、検出力が静かに低下する。

## 決定

- `wait_until()` の `except Exception: pass` を
  `except Exception: raise` へ変更する。
- ADR-0144 準拠の明示 `except RuntimeError: raise` は維持する
  （機能的に冗長だが契約を明示するため残す）。
- ただし **`Cdp.evaluate()` 自体は CDP `error` と JS `exceptionDetails` を
  検査していない**。そのため JS 式の typo／未定義定数は Python 例外にならず、
  `wait_until()` は `value=None` としてポーリングを続ける（根本解決は未実施）。
- 既存 249 項目の待機式には `?.textContent.includes(...)` のように
  要素未出現時に JS TypeError になるものが 60 箇所以上ある。
  evaluate を変更して JS 例外を即 Python 例外化すると、
  これら正常な一時状態まで失敗するため本次では変更しない。
- 期待総数は 249 のまま。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| Python 例外のみ再送出（採用） | 既存 249 項目を壊さず基盤例外を可視化 |
| evaluate で JS 例外も検査 | 一時 TypeError で大量失敗する（将来課題） |
| 全待機式を安全呼びへ変更 | 範囲が広く本次対象外 |

## 検証（Runtime Proof）

- 一時フェイク Cdp で 4 ケースを確認:
  - evaluate 例外が 0.000s / 1 call で即送出。
  - RuntimeError も即送出。
  - truthy は即 `True`。
  - falsy は deadline までポーリングして `False`（既存挙動維持）。
- `python3 -m py_compile`、`./scripts/verify.sh`: 成功。
- E2E **249 項目すべて成功**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定（本次で解消していない根本課題）

- `Cdp.evaluate()` で CDP `error` / JS `exceptionDetails` を検査し、
  Python 例外へ変換するか。
- JS 例外を即 fail にする場合の既知一時例外の分類
  （ExecutionContext 破壊、`?.textContent.includes` 系）と retry 方針。
- retry の所在（wait_until 内／呼び出し側）、回数・バックオフ。
- 例外クラス新設 vs RuntimeError 統一（ADR-0144 の catch 契約）。
- CDP recv 10 秒と wait_until deadline 20 秒の競合（ADR-0144 から継続）。
