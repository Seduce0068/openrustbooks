---
type: Decision
title: "ADR-0141: 画像縮小 E2E のフレーク修正"
description: "画像縮小の完了待ちを条件確認＋再待機にし、E2E の取りこぼしを防ぐ決定を記録する。"
tags:
  - ADR
  - testing
  - epub
generated:
  by: dsh
  at: "2026-11-18T14:00:00+09:00"
status: accepted
---

## 背景

ADR-0140 の 10 回連続実行で
`有効時は巨大画像が 2000px 以下に縮小される`（naturalWidth=4500）が 1 回失敗した。
E2E は `wait_until(..., timeout=10)` の戻り値を無視し、
縮小完了前に `naturalWidth` を読んでいた（**推測**）。

## 決定

- `tools/verify-web-reader.py:2663-2681` を変更し、
  `wait_until` の戻り値を確認する。
- タイムアウト時は `naturalWidth` / `src` / `status` の診断ログを出してから
  10 秒再待機する（合計最大 20 秒）。
- チェック条件、縮小ポリシー（4000px 超・2000px 以下・MIME 保持）、
  E2E 期待総数 229 は変更しない。
- `web/src/lib.rs` は変更しない（E2E 側の待機のみで解決）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 完了待ちの戻り値確認＋再待機（採用） | 最小変更でフレークを解消 |
| 固定 sleep 延長 | 完了シグナルが無いため根本解決にならない |
| status 待ち | 警告用 status と衝突（ADR-0139 の前例） |
| アプリに data 属性追加 | 決定论的だが変更範囲が広がる（未決定として残す） |

## 影響

- 縮小が遅い環境でも 20 秒まで待つため、誤失敗が減る。
- 完全に縮小が完了しない場合は引き続き失敗（fail-closed）し、
  診断ログで原因を特定できる。
- 新規依存なし、unsafe なし。

## 検証

- 修正後 E2E **5 回連続 229/0**（所要 125〜140 秒／回、
  タイムアウト warn は 5 回とも未出力）。
- `./scripts/verify.sh` rc=0（全ゲート・audit・trunk build・E2E 229/0）。
- 全単体テスト core 188 + desktop 56 + web 2、wasm-test 3、
  audit 脆弱性 0 件。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定（downscale-flake-auditor の指摘）

- アプリ側の完了シグナル（`data-downscaled` 等）と
  `downscale_large_images()` 内の generation guard。
- `applying=false` を縮小完了まで遅らせるか。
- `content_width > 2000` の場合の「2000px 以下」という文言と
  実装（`max(2000, content_width)`）のずれ（ADR-0067 から継続）。
- ADR-0140 第 10 回の起動待ち早期 abort の切り分け。
- 5 回連続成功の統計的限界（ADR-0140 から継続）、CLS 計測、
  5 ブラウザ実機検証。
