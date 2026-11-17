---
type: Decision
title: "ADR-0073: JPEG 品質の設定化"
description: "縮小 JPEG の品質を表示設定から変更できるようにする決定を記録する。"
tags:
  - ADR
  - epub
  - web
  - performance
generated:
  by: dsh
  at: "2026-11-17T10:45:00+09:00"
status: accepted
---

## 背景

ADR-0072 の未決定「JPEG 品質を利用者が設定で変更できるようにするか」。
品質 0.85 固定では、画質を優先したい利用者やファイルサイズを優先したい
利用者が調整できなかった。

## 決定

- `ReaderSettings` に `jpegQuality` を追加。既定 85、範囲 50〜100
  （0.50〜1.00 を整数パーセントで保持）。
- 既存設定にキーが無い場合は既定 85（後方互換）。保存・エクスポート・
  インポート対象。
- ツールバーに数値入力（`<input id="reader-jpeg-quality">`）を置く。
- `downscale_large_images` の JPEG encoder options に
  `quality: jpeg_quality / 100.0` を使う。
- リセット時は既定 85 に戻す。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| パーセント整数で設定化（採用） | 入力が分かりやすく、既存の閾値設定と同じパターン |
| 0〜1 の小数で設定化 | 入力が難しく、保存形式も煩雑 |
| 固定 0.85 | 利用者の好みに対応できない |

## 影響

- 縮小機能をオンにした利用者が JPEG の画質を調整できる。
- 縮小機能自体は既定オフのまま（ADR-0067）。
- 追加依存は無い。

## 検証

- ヘッドレス Chrome 157 と `tools/verify-web-reader.py` の E2E 202 項目が
  5 回連続で成功。
- JPEG 品質入力の存在、変更値（70）の保存、範囲外（999）の 100 への
  クランプを確認した。
- `cargo test --all-targets --locked` は 125 件成功。
- 6 ゲート（fmt / check / test / clippy / wasm32 × 2）と `trunk build` が成功。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- JPEG 品質変更による実際のファイルサイズ・画質の定量比較。
- `IntersectionObserver` による画像の遅延読み込み（ADR-0034 から継続）。
- EPUB 内 CSS、縦書き・ページ送り（ADR-0012 から継続）。
