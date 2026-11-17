---
type: Decision
title: "ADR-0082: mailto リンクの検証"
description: "mailto: リンクが外部リンクと同じ安全な分岐で開かれることを E2E で検証する決定を記録する。"
tags:
  - ADR
  - epub
  - web
  - security
generated:
  by: dsh
  at: "2026-11-17T14:10:00+09:00"
status: accepted
---

## 背景

ADR-0081 の未決定。外部リンク（http/https）のポップアップ呼び出しは
検証済みだが、同じ分岐に含まれる `mailto:` は未検証だった。

## 決定

- サンプル CH1 に `mailto:test@example.com` へのリンクを追加する。
- E2E で次を確認する。
  - サニタイズ後も href が残る。
  - クリックで `window.open` が 1 回呼ばれる。
  - URL が `mailto:test@example.com`、target が `_blank`、features に
    `noopener` と `noreferrer` が含まれる（ADR-0081 のスタブパターン）。
- 実際にはメールアプリを開かないため、外部副作用は無い。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| スタブで引数を検証（採用） | 副作用なしで http/https と同じ安全性を確認できる |
| 実際にメールアプリを開く | E2E 環境に依存し不安定 |
| 検証なし | 分岐の一部が回帰から守られない |

## 影響

- 外部リンクと mailto の両方が同じ安全方針で開かれることを確認できる。
- プロダクションコードの変更は無い。
- 追加依存は無い。

## 検証

- ヘッドレス Chrome 157 と `tools/verify-web-reader.py` の E2E 219 項目が
  8 回中 7 回成功（1 回はディスク容量不足による一時的な環境問題で、
  回復後は 3 回連続成功）。
- mailto リンクの href 保持と、`window.open` の引数（`_blank` /
  `noopener,noreferrer`）を確認した。
- `cargo test --all-targets --locked` は 125 件成功。
- 6 ゲート（fmt / check / test / clippy / wasm32 × 2）と `trunk build` が成功。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- `IntersectionObserver` による画像の遅延読み込み（ADR-0034 から継続）。
- EPUB 内 CSS、縦書き・ページ送り（ADR-0012 から継続）。
