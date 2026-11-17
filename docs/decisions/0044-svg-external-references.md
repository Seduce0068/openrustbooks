---
type: Decision
title: "ADR-0044: SVG の外部リソース参照制御"
description: "SVG 内の外部スキーム参照を除去し、Blob URL 直接オープン時のプライバシー漏洩を防ぐ決定を記録する。"
tags:
  - ADR
  - epub
  - web
  - security
generated:
  by: dsh
  at: "2026-11-17T01:35:00+09:00"
status: accepted
---

## 背景

ADR-0043 で SVG のスクリプト実行経路を塞いだが、
`<use href="http://...">` や `<image href="https://...">` などの外部参照は残っていた
（ADR-0043 の未決定）。Blob URL を直接開くと、これらの参照がネットワーク要求を
発生させ、読者の IP や閲覧時刻が外部サーバに漏れる可能性がある。

## 決定

### 外部スキームの除去

- SVG サニタイズ時に、`href` / `xlink:href` の値が以下のスキームで始まる場合、
  その属性を削除する。
  - `javascript:`（既存判定を維持）
  - `http:` / `https:`
  - `data:` / `blob:` / `file:`
- 判定は空白・制御文字を除去した正規化済み文字列に対して行う
  （ADR-0043 の回避策を踏襲）。
- 相対参照（EPUB 内の画像など）は維持する。
- ADR-0043 のサニタイズ処理（script / foreignObject / SMIL / on* 除去）は
  そのまま維持する。
- 重複していたコメント（SMIL に関する 2 行）を整理した。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 外部スキームだけを除去（採用） | プライバシーを守りつつ相対参照の画像は壊さない |
| すべての href を除去 | EPUB 内の相対画像まで壊れる |
| 何もしない | Blob URL 直接オープンで外部サーバに情報が漏れる |

## 影響

- SVG を含む EPUB で、Blob URL を直接開いても外部サーバへの要求が発生しない。
- EPUB 内の相対画像を参照する SVG は従来どおり表示される。
- 外部の画像・フォントを参照する SVG は、それらが表示されなくなる
  （画像としての自己完結性を優先した受容可能な制約）。
- 追加依存は無く、既存の `web-sys` 機能の範囲で実装している。

## 検証

- ヘッドレス Chrome 157 と `tools/verify-web-reader.py` の E2E 156 項目が
  7 回中 6 回成功。残り 1 回は既存のフラグメントスクロールテストが
  タイミングで失敗したが、今回の SVG 変更とは無関係。
  `https://` / `data:` の外部参照が除去され、相対参照 `images/pic.png` が
  維持されることを確認した。
- `cargo test --all-targets --locked` は 119 件成功。
- 6 ゲート（fmt / check / test / clippy / wasm32 × 2）と `trunk build` が成功。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 解決済み

- XML 処理命令（`<?xml-stylesheet?>`）の扱い: ADR-0045 で
  SVG サニタイズ時に `PROCESSING_INSTRUCTION_NODE` を削除すると決定・実装した。

## 未決定

- CSS の `url()` による外部参照（EPUB 内 CSS の安全な適用と併せて検討）。
- `IntersectionObserver` による画像の遅延読み込み（ADR-0034 から継続）。
- EPUB 内 CSS、縦書き・ページ送り（ADR-0012 から継続）。
