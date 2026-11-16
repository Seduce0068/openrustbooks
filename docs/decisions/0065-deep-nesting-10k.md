---
type: Decision
title: "ADR-0065: 10000 階層の入れ子テスト"
description: "より深い入れ子 XHTML の挙動を回帰テストとして固定する決定を記録する。"
tags:
  - ADR
  - rust
  - epub
  - testing
generated:
  by: dsh
  at: "2026-11-17T08:20:00+09:00"
status: accepted
---

## 背景

ADR-0050 の未決定「より深い入れ子（例: 10000 階層）での挙動」。
調査時に手動で 10000 階層を試し、約 5.4 秒で正常に処理できることを
確認した。これを回帰テストとして固定する。

## 決定

- `core/src/sanitize.rs` に 10000 階層の `<div>` 入れ子を
  `sanitize_epub_html` に渡すテストを追加する。
- パニックせず文字列が返り、本文が残ることを確認する。
- 既存の 1000 階層テストは維持する。
- プロダクションコードは変更しない（テスト追加のみ）。

## 影響

- サニタイザの再帰処理が壊れた場合に 10000 階層で検出できる。
- テスト時間が約 5.5 秒増えるが、受容可能な範囲。
- 追加依存は無い。

## 検証

- `cargo test -p openrustbooks-core --lib --locked nesting` が成功
  （10000 階層は 5.54 秒）。
- `cargo test --all-targets --locked` は 125 件成功（124 → 125）。
- ヘッドレス Chrome 157 と `tools/verify-web-reader.py` の E2E 187 項目が成功。
- 6 ゲート（fmt / check / test / clippy / wasm32 × 2）と `trunk build` が成功。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- `IntersectionObserver` による画像の遅延読み込み（ADR-0034 から継続）。
- EPUB 内 CSS、縦書き・ページ送り（ADR-0012 から継続）。
