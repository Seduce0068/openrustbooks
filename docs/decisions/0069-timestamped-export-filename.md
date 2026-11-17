---
type: Decision
title: "ADR-0069: エクスポート名のタイムスタンプ"
description: "エクスポート JSON のダウンロード名にローカル時刻のタイムスタンプを付ける決定を記録する。"
tags:
  - ADR
  - epub
  - web
generated:
  by: dsh
  at: "2026-11-17T09:35:00+09:00"
status: accepted
---

## 背景

エクスポートのダウンロード名が `openrustbooks-positions.json` 固定のため、
同じブラウザで再度エクスポートすると `openrustbooks-positions (1).json`
のようにブラウザが連番を付け、どれが最新か分かりにくかった。

## 決定

- ダウンロード名を `openrustbooks-YYYYMMDD-HHMMSS.json` にする。
- タイムスタンプはローカルタイムゾーン（`js_sys::Date` のローカル getter）。
- 秒まで含めるため、通常は連番が付かない。同じ秒に 2 回エクスポートした
  場合のみブラウザの連番に任せる。
- JSON の中身（version / positions / settings / history）は変更しない。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| タイムスタンプ付き（採用） | ファイル名だけで新旧が判断できる |
| 固定名のまま | 連番が増え、最新が分かりにくい |
| UUID | 時系列が分からない |

## 影響

- エクスポートしたファイル名に日時が入る。
- 既存のバックアップファイルの形は変わらない（読み込みはIME入力の
  ファイル選択なので影響なし）。
- 追加依存は無い。

## 検証

- ヘッドレス Chrome 157 と `tools/verify-web-reader.py` の E2E 198 項目が
  5 回連続で成功。
- ダウンロード名が `openrustbooks-\d{8}-\d{6}(?: \(\d+\))?\.json` の
  形式であることを確認した。
- `cargo test --all-targets --locked` は 125 件成功。
- 6 ゲート（fmt / check / test / clippy / wasm32 × 2）と `trunk build` が成功。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- ファイル名へのタイムゾーン明示（例: `+0900`）。
- `IntersectionObserver` による画像の遅延読み込み（ADR-0034 から継続）。
- EPUB 内 CSS、縦書き・ページ送り（ADR-0012 から継続）。
