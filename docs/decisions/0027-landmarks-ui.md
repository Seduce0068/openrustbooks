---
type: Decision
title: "ADR-0027: landmarks の移動 UI"
description: "EPUB3 nav の landmarks をドロップダウンで表示し、移動できるようにする決定を記録する。"
tags:
  - ADR
  - epub
  - web
  - wasm
generated:
  by: dsh
  at: "2026-11-15T15:48:00+09:00"
status: accepted
---

## 背景

core は `landmarks()` と `page_list()` を公開しているが、Web UI は目次しか
使っておらず、landmarks の位置へ直接移動できなかった（ADR-0009・ADR-0010 の
未決定「landmarks / page-list の活用」）。

## 決定

### landmarks ドロップダウン

- ツールバーに `<select id="reader-landmarks">` を置く。
- `WasmEpubBook::landmarks()` で取得し、各エントリの title をラベルに使う。
- landmarks が空のときは disabled にする。
- 選択を変えると、その landmark の href を spine の章へ解決し、
  `navigate_to` で移動する。フラグメントがあればスクロールする。

### ARIA

ドロップダウンに `aria-label="Landmarks"` を付ける。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| landmarks ドロップダウン（採用） | 追加依存なし、既存の `navigate_to` を再利用できる |
| landmarks を目次に統合する | 目次と landmarks は役割が異なるため、分離する |
| landmarks を使わない | 出版社が用意した構造ナビゲーションを活用しない |

## 影響

- landmarks を持つ EPUB で、表紙・本文開始・索引などへ直接移動できる。
- landmarks が無い EPUB ではドロップダウンが disabled になる。
- 追加依存は無く、既存の `web-sys` 機能の範囲で実装している。

## 検証

- ヘッドレス Chrome 157 と `tools/verify-web-reader.py` の E2E 106 項目が
  3 回連続で成功。landmarks について、ドロップダウンの存在、landmarks が
  読み込まれていること、選択で章が移動することを確認した。
- `cargo test --all-targets --locked` は 119 件成功。
- 6 ゲート（fmt / check / test / clippy / wasm32 × 2）と `trunk build` が成功。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- page-list の活用（ページ番号での移動 UI）。
- landmarks の種別（cover, bodymatter 等）ごとのアイコン表示。
- landmarks が無い EPUB でのフォールバック。
- EPUB 内 CSS、縦書き・ページ送り（ADR-0012 から継続）。
