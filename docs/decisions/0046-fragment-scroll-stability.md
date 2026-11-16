---
type: Decision
title: "ADR-0046: フラグメントスクロールの安定化"
description: "画像レイアウト未確定によるフラグメント移動のスクロール失敗を解消する決定を記録する。"
tags:
  - ADR
  - epub
  - web
  - testing
generated:
  by: dsh
  at: "2026-11-17T02:40:00+09:00"
status: accepted
---

## 背景

「フラグメント付きリンクで移動先へスクロールする」E2E が約 17%（12 回中 2 回、
以降も断続的に）失敗していた。デバッグの結果、失敗時も
`scroll_to_fragment` は対象要素（`part`）を見つけていたが、
`scrollIntoView()` が効かず `scrollTop` が 0 のままだった。

原因は、画像（特に縦長 PNG）のレイアウトが確定する前に `scrollIntoView()`
が走ること。フラグメント移動には当初 retry が無く、レイアウト確定後の
再適用が行われなかった。

## 決定

### スクロールの適用方法

- `scroll_to_fragment` で `scrollIntoView()` の使用をやめ、
  強制リフロー（`offsetHeight` の読み取り）のうえで
  `.reader-content` の `scrollTop` を対象要素の `offsetTop` に明示設定する。
- `apply_position_frame` にフラグメント移動用の retry を追加する。
  `scrollTop` が 0 のままの場合は、`attempt < 3` の範囲で再適用する。
- これにより、レイアウトが確定した後の再試行で正しい位置が適用される。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 強制リフロー + 明示 scrollTop + retry（採用） | レイアウト未確定に依存せず確実に適用できる |
| `scrollIntoView()` のまま retry だけ追加 | 10 回中 1 回失敗が残った（効果不十分） |
| 強制リフローだけ追加 | 10 回中 1 回失敗が残った（効果不十分） |
| E2E の待ち時間を延ばす | 根本原因を隠すだけ |

## 影響

- フラグメント付きリンクの移動先スクロールが安定する。
- 通常のスクロール位置復元（位置保存からの復元）のロジックは変更しない。
- 追加依存は無く、既存の `web-sys` 機能の範囲で実装している。

## 検証

- ヘッドレス Chrome 157 と `tools/verify-web-reader.py` の E2E 157 項目が
  **10 回連続で成功**（修正前は約 17% の頻度でフラグメントスクロールが失敗）。
- `cargo test --all-targets --locked` は 119 件成功。
- 6 ゲート（fmt / check / test / clippy / wasm32 × 2）と `trunk build` が成功。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- フラグメント移動の retry 回数（現在 3 回）のチューニング。
- 画像の遅延読み込み（`IntersectionObserver`）とフラグメント移動の併用。
- CSS の `url()` による外部参照（EPUB 内 CSS の安全な適用と併せて検討）。
- EPUB 内 CSS、縦書き・ページ送り（ADR-0012 から継続）。
