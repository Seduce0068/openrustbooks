---
type: Decision
title: "ADR-0144: CDP recv タイムアウト"
description: "E2E の CDP 応答待ちにタイムアウトを設け、ハング時に fail-closed で診断を残す決定を記録する。"
tags:
  - ADR
  - testing
  - e2e
generated:
  by: dsh
  at: "2026-11-18T15:40:00+09:00"
status: accepted
---

## 背景

`tools/verify-web-reader.py` の `Cdp.send()` は `await self.ws.recv()` に
タイムアウトが無く、CDP が応答しない場合 `wait_until` の 20 秒 deadline も
効かず E2E 全体がハングし得た（ADR-0142 / 0143 の未決定）。

## 決定

- `CDP_RECV_TIMEOUT = 10.0` 秒を採用（`wait_until` の 20 秒より短く、
  再ポーリングの機会を残す値。browser WS 捕捉の 10 秒と同値だが意味は別）。
- `Cdp.send()` は `asyncio.wait_for(self.ws.recv(), timeout=...)` を使い、
  タイムアウト時は `RuntimeError` を送出する（fail-closed）。
  - メッセージに `method` / `request_id` / `timeout` /
    `buffered_diagnostic_events` / `dropped_diagnostic_events` /
    `diagnostic_event_preview`（最大 3 件・各 160 文字）を添える。
  - `__cause__` は `TimeoutError` / `asyncio.wait_for`。
  - `take_diagnostics()` は呼ばず、バッファを破壊しない。
- `wait_until` は `except RuntimeError: raise` を先に置き、
  リトライでタイムアウトを飲み込まない。
- `run_checks()` のラッパーが `format_cdp_diagnostics(cdp)` の内容を出力し、
  `finish(checker)` で `expected 229 checks, but executed N` を出してから再 raise。
- `main()` が RuntimeError を捕捉し、traceback なしで rc=1 を返す
  （finally のクリーンアップは維持）。
- `select_page_target()` の `Target.getTargets` には
  `timeout=min(2.0, remaining)` を渡し、
  `TARGET_SELECT_TIMEOUT = 8.0` が 10 秒 recv に飲み込まれないようにした。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 10 秒 recv ＋ fail-closed（採用） | ハングを検知しつつ再ポーリング機会を残す |
| 20 秒（wait_until と同値） | ハング検出自体が deadline と競合する |
| RuntimeError を `except Exception` で吸う | タイムアウトが不可視になり fail-open になる |

## 影響

- CDP が沈黙した場合、10 秒で失敗し、診断と 229 サマリが残る。
- 通常系の挙動・チェック内容・期待総数 229 は不変。
- 新規依存なし（Python 標準 `asyncio` のみ）。

## 検証（Runtime Proof）

- 沈黙フェイク CDP によるネガティブ検証（checker 件数には不含）:
  1. `send()` が全トークン一致の RuntimeError を送出、診断バッファ非消費。
  2. `wait_until` が 0.10 秒で RuntimeError を再送出（False 化しない）。
  3. `run_checks()` が `pageerror: boom` と
     `expected 229 checks, but executed 0` を出力して再送出。
- `python3 -m py_compile` rc=0。
- 修正後 E2E 5 回連続すべて 229/0（125〜153 秒／回）。
- `./scripts/verify.sh` rc=0（全 11 ゲート、E2E 229/0）。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- recv キャンセル後の WebSocket 再利用の安全性は未検証
  （現在は RuntimeError で抜けるため再接続不要だが、**推測**）。
- `wait_until` の 20 秒 deadline と recv 10 秒 × 再ポーリングで
  実質 20 秒をわずかに超え得る（monotonic 残り時間の適用は未実施）。
- タイムアウト値の CLI / 環境変数化。
- リトライ前の `localStorage.clear()` タイミング（ADR-0142 から継続）。
- 20 回連続実行、CLS 計測、5 ブラウザ実機検証（ADR-0140 から継続）。
