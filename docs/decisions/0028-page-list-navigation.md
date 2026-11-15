---
type: Decision
title: "ADR-0028: page-list によるページ移動"
description: "EPUB3 nav の page-list を使い、ページ番号入力で章へ移動できるようにする決定を記録する。"
tags:
  - ADR
  - epub
  - web
  - wasm
generated:
  by: dsh
  at: "2026-11-15T17:48:00+09:00"
status: accepted
---

## 背景

core は `page_list()` を公開しているが、Web UI は使っていなかった
（ADR-0009・ADR-0027 の未決定「page-list の活用（ページ番号での移動 UI）」）。
紙書籍のページ番号で参照される本（技術書・教科書）では、ページ番号から
直接該当箇所へ移動できることが読書体験の要になる。

## 決定

### ページ移動 UI

- ツールバーにページ番号入力 `<input id="reader-page-input">` と移動ボタン
  `<button id="reader-page-go">` を置く。
- `WasmEpubBook::page_list()` で取得し、入力値と `entry.title` を完全一致で
  探す。page-list の title は紙のページ番号である。
- 一致した entry の href を spine の章へ解決して `navigate_to` で移動する。
  フラグメントがあればスクロールする。
- 一致する entry が無い場合は状態メッセージ（`Page N not found.`）で報告する。
  入力値は検証して失敗したら fail-closed で何もしない。
- page-list が空のときは入力とボタンを disabled にする。

### 章の解決

landmarks（ADR-0027）と page-list は、href を spine に解決する必要がある。
OPF spine の href は `OEBPS/` などのディレクトリ接頭辞を含む一方、nav の
href は相対パスで接頭辞が異なることがあるため、ファイル名部分の一致で解決する
（`href.rsplit('/').next()` の比較）。解決には spine の href 一覧を保持する
`chapter_hrefs` signal を使う。

### ARIA

入力に `aria-label="Page"` を、ボタンに `aria-label="Go to page"` を付ける。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| ページ番号入力＋移動ボタン（採用） | 追加依存なし、キーボードで完結する |
| landmarks 形式のドロップダウン | 紙のページ番号は数百件になりドロップダウンに向かない |
| page-list を使わない | 出版社が用意したページ対応を活用しない |

## 影響

- page-list を持つ EPUB で、紙のページ番号から直接移動できる。
- page-list が無い EPUB では入力とボタンが disabled になる。
- 追加依存は無く、既存の `web-sys` 機能の範囲で実装している。
- 位置の適用に使う `apply_position_frame` の引数を `PositionApplyCtx`
  構造体にまとめ、画面保存の抑制状態（適用中フラグ・適用完了時刻）を
  一元管理する。

## 検証

- ヘッドレス Chrome 157 と `tools/verify-web-reader.py` の E2E 110 項目が
  連続で成功。page-list について、入力とボタンの存在、ページ番号での
  章移動、存在しないページ番号の状態メッセージを確認した。
- `cargo test --all-targets --locked` は 119 件成功。
- 6 ゲート（fmt / check / test / clippy / wasm32 × 2）と `trunk build` が成功。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- ページ番号のオートコンプリート（候補表示）。
- 現在表示中のページ番号の表示。
- page-list が無い EPUB での代替ナビゲーション。
- EPUB 内 CSS、縦書き・ページ送り（ADR-0012 から継続）。
