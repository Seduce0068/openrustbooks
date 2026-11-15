---
type: Decision
title: "ADR-0034: 本文画像の非同期デコード"
description: "本文画像に decoding=\"async\" を付与し、デコードを非同期化して描画をブロックしない決定を記録する。"
tags:
  - ADR
  - epub
  - web
  - wasm
generated:
  by: dsh
  at: "2026-11-16T00:25:00+09:00"
status: accepted
---

## 背景

ADR-0020 までで画像の読み込み完了を待ってから位置を適用しているが、
画像のデコードはメインスレッドをブロックする。`loading="lazy"` は
`images_settled` が `complete` を待つため位置補正と相性が悪く、これまで
先送りされていた（ADR-0020 の未決定「画像の遅延読み込み
（`loading="lazy"`）と位置補正の併用」）。

## 決定

### `decoding="async"`

- `resolve_images` で `src` を Blob URL に差し替える各画像に
  `decoding="async"` を設定する。
- `loading="lazy"` は付けない（`images_settled` が `complete` を待つため、
  画面外の画像が永久に未完了になり位置補正が壊れる。ADR-0020 の判断を
  維持する）。
- `decoding="async"` は画像のデコードをメインスレッドから外し、
  本文の描画をブロックしない。`complete` の判定には影響しないため
  位置補正との併用に問題はない。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| `decoding="async"` のみ（採用） | 位置補正を壊さず描画のブロックを減らす |
| `loading="lazy"` を併用 | 画面外画像が `complete` にならず位置補正が壊れる |
| どちらも付けない（現状） | 大きな画像のデコードがメインスレッドをブロックする |

## 影響

- 本文の画像デコードが非同期になり、スクロール等の応答性が向上する。
- 位置補正の仕組み（ADR-0020）は変わらない。
- 追加依存は無く、既存の `web-sys` 機能の範囲で実装している。

## 検証

- ヘッドレス Chrome 157 と `tools/verify-web-reader.py` の E2E 131 項目が
  3 回連続で成功。本文画像に `decoding="async"` が付いていること、
  `loading="lazy"` が付いていないことを確認した。
- `cargo test --all-targets --locked` は 119 件成功。
- 6 ゲート（fmt / check / test / clippy / wasm32 × 2）と `trunk build` が成功。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- 画像のデコード・縮小（大きな画像のメモリ使用）。
- `loading="lazy"` と位置補正の併用（`IntersectionObserver` で可視画像の
  み `complete` を待つ方式）。
- 縦書き・ページ送り。
- EPUB 内 CSS の安全な適用（ADR-0012 から継続）。
