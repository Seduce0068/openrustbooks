---
type: Decision
title: "ADR-0050: 深い入れ子 XHTML の安全性確認"
description: "深い入れ子の XHTML がサニタイズ時に異常終了しないことをテストで確認する決定を記録する。"
tags:
  - ADR
  - rust
  - epub
  - security
generated:
  by: dsh
  at: "2026-11-17T03:22:00+09:00"
status: accepted
---

## 背景

ADR-0013 の未決定「深い入れ子の XHTML によるスタック消費」。
`core/src/sanitize.rs` のサニタイズに深い入れ子のテストが無く、非常に深い
`<div>` の入れ子でスタックオーバーフローや異常終了しないことを確認する
必要があった。

## 決定

- `core/src/sanitize.rs` のテストに、1000 階層の `<div>` 入れ子を含む
  XHTML を `sanitize_epub_html` に渡すテストを追加する。
- パニックせずに文字列が返ること、本文中の `onclick` と `script` が
  除去されることを確認する（既存のサニタイズ方針を維持）。
- プロダクションコードは変更しない（テスト追加のみ）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 1000 階層のテストを追加（採用） | 現状の安全性を確認し、将来の回帰を防ぐ |
| サニタイザに深さ制限を追加 | 現状問題が無く、正当な深い構造を壊すリスクがある |
| 未決定のまま残す | スタック消費のリスクを把握できない |

## 影響

- 深い入れ子によるスタック消費のリスクがないことを継続的に確認できる。
- プロダクションコードの挙動は変わらない。
- 追加依存は無い。

## 検証

- `cargo test -p openrustbooks-core --lib sanitize --locked` は 19 件成功
  （新規 1 件を含む）。1000 階層の入れ子でも正常に処理された。
- `cargo test --all-targets --locked` は 120 件成功。
- ヘッドレス Chrome 157 と `tools/verify-web-reader.py` の E2E 166 項目が成功。
- 6 ゲート（fmt / check / test / clippy / wasm32 × 2）と `trunk build` が成功。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- より深い入れ子（例: 10000 階層）での挙動。
- EPUB 内 CSS の安全な適用（`url()` の除去、スコープ付きスタイル）。
- 読み込み時のメモリコピー（`ArrayBuffer`・`Vec`・`ArchiveReader` の複製）。
- EPUB 内 CSS、縦書き・ページ送り（ADR-0012 から継続）。
