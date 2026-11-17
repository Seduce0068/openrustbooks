---
type: Decision
title: "ADR-0070: エクスポート名へのタイムゾーン明示"
description: "エクスポート JSON のファイル名にタイムゾーン偏移を付ける決定を記録する。"
tags:
  - ADR
  - epub
  - web
generated:
  by: dsh
  at: "2026-11-17T09:50:00+09:00"
status: accepted
---

## 背景

ADR-0069 の未決定「ファイル名へのタイムゾーン明示」。
`openrustbooks-YYYYMMDD-HHMMSS.json` はローカル時刻だが、どのタイムゾーンの
時刻か分からない。異なるタイムゾーンの端末間でバックアップを扱うときに
順序を正しく判断できない。

## 決定

- ダウンロード名を `openrustbooks-YYYYMMDD-HHMMSS+HHMM.json` にする。
  例: `openrustbooks-20261117-093500+0900.json`。
- 偏移は `Date.getTimezoneOffset()` の符号を反転させた分を `HHMM` に変換する。
  - `getTimezoneOffset()` は「UTC - 現地」の分（JST なら -540）を返すため、
    符号を反転して `+0900` とする。
  - UTC は `+0000`、負の偏移（例: 米国東部）は `-0500` となる。
- JSON の中身は変更しない。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 数値偏移を付与（採用） | タイムゾーン名に依存せず順序判断ができる |
| タイムゾーン名（JST 等） | 省略形が曖昧で、地域によって異なる |
| 時刻を UTC に統一 | ローカル時刻との対応が分かりにくい |

## 影響

- エクスポートしたファイル名に日時と偏移が入る。
- 既存のバックアップの読み込みはファイル選択式のため影響なし。
- 追加依存は無い。

## 検証

- ヘッドレス Chrome 157 と `tools/verify-web-reader.py` の E2E 198 項目が
  5 回連続で成功。
- ダウンロード名が `openrustbooks-\d{8}-\d{6}[+-]\d{4}(?: \(\d+\))?\.json`
  の形式であることを確認した。
- `cargo test --all-targets --locked` は 125 件成功。
- 6 ゲート（fmt / check / test / clippy / wasm32 × 2）と `trunk build` が成功。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- 夏時間の境界における偏移の扱い（Date API が自動で判定するため追加処理は不要）。
- `IntersectionObserver` による画像の遅延読み込み（ADR-0034 から継続）。
- EPUB 内 CSS、縦書き・ページ送り（ADR-0012 から継続）。
