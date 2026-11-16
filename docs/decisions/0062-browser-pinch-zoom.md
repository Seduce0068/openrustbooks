---
type: Decision
title: "ADR-0062: ブラウザ標準のピンチズーム"
description: "リーダーがブラウザ標準のピンチズームを阻害していないことを確認する決定を記録する。"
tags:
  - ADR
  - epub
  - web
  - accessibility
generated:
  by: dsh
  at: "2026-11-17T07:25:00+09:00"
status: accepted
---

## 背景

ADR-0025 の未決定「ピンチズーム・ダブルタップなどのジェスチャー」。
`index.html` の viewport は `width=device-width, initial-scale=1.0` で
`user-scalable=no` は無く、`touch-action` の指定も無いため、ブラウザ標準の
ピンチズームは有効なはず。これを確認して未決定の一部を整理する。

## 決定（現状の確認）

- viewport メタタグに `user-scalable=no` / `maximum-scale=1` を付けない。
- `.reader-content` に `touch-action: none` を指定しない。
- これにより、ブラウザ標準のピンチズームがそのまま有効になる。
- 独自のピンチズーム実装は追加しない（ブラウザ標準に任せる）。
- ダブルタップの章移動は今回見送る（標準のダブルタップズームと競合する
  可能性があり、誤操作のリスクがあるため。ADR-0025 に残す）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| ブラウザ標準に任せる（採用） | 実装不要でアクセシビリティも守られる |
| `touch-action: none` で独自実装 | 標準ズームを奪い、操作も不安定になる |
| ダブルタップで章移動 | 標準ズームと競合し誤操作の恐れがある |

## 影響

- 利用者はブラウザ標準のピンチズームで本文を拡大できる。
- 既存のスワイプによる章移動はそのまま動作する（横スワイプの閾値設定も維持）。
- プロダクションコードの変更は無い（確認とテスト追加のみ）。

## 検証

- ヘッドレス Chrome 157 と `tools/verify-web-reader.py` の E2E 181 項目が
  5 回連続で成功。viewport がズームを許可していること、本文領域の
  `touch-action` が `none` でないことを確認した。
- `cargo test --all-targets --locked` は 124 件成功。
- 6 ゲート（fmt / check / test / clippy / wasm32 × 2）と `trunk build` が成功。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- ダブルタップによる章移動（標準ズームとの競合をどう解決するか、ADR-0025）。
- `IntersectionObserver` による画像の遅延読み込み（ADR-0034 から継続）。
- EPUB 内 CSS、縦書き・ページ送り（ADR-0012 から継続）。
