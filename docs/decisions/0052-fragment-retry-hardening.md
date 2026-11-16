---
type: Decision
title: "ADR-0052: フラグメントスクロール retry の強化と残る課題"
description: "フラグメントスクロールの retry を強化し、残るフレークの調査結果を記録する。"
tags:
  - ADR
  - epub
  - web
  - testing
generated:
  by: dsh
  at: "2026-11-17T04:35:00+09:00"
status: accepted
---

## 背景

ADR-0046 で `scrollIntoView()` から明示的な `scrollTop` 設定に変更したが、
その後も約 1/10 の頻度で「フラグメント付きリンクで移動先へスクロールする」が
`scrollTop=0` で失敗した。retry は 3 回・`setTimeout(0)` のみで、画像の
レイアウトが確定する前に使い切られていた。

## 決定

- フラグメント移動で `scrollTop` が 0 のままの場合、16ms（1 フレーム相当）
  間隔で最大 12 回まで再試行する（位置保存の retry は従前の 3 回を維持）。
- スクロール目標の算出を `offsetTop` から
  `getBoundingClientRect()` の実座標差に変更した（`DomRect` feature を追加）。
- デバッグ用に `set_scroll_top` の前後の値を記録して失敗を再現した。

## 調査結果（重要）

デバッグの結果、**失敗時も `set_scroll_top(2375)` は成功していた**
（`before=0, after=2375, target=2594`）。つまりスクロールの適用自体は
機能しており、E2E が `document.querySelector('.reader-content').scrollTop`
を読む時点で 0 に戻っている。考えられる要因は DOM の差し替えによる
スクロール位置のリセットだが、ここまでの調査では特定できていない。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| retry 強化 + 実座標計算（採用） | レイアウト未確定による失敗を減らせる |
| E2E の待ち時間を延ばすだけ | 根本原因を隠すだけ |
| デバッグ値をプロダクションに残す | 本番コードをテスト目的で汚す |

## 影響

- レイアウト未確定によるスクロール失敗は大幅に減った。
- ただし 15 回中 1 回の頻度で失敗が残る（下記検証）。
- `DomRect` は既存 `web-sys` クレートの feature 追加であり、新規依存ではない。

## 検証

- E2E（`tools/verify-web-reader.py`）166 項目:
  - retry 強化のみ: 15 回中 1 回失敗。
  - `getBoundingClientRect()` 併用: 15 回中 1 回失敗。
  - デバッグ再現時: `set_scroll_top` は常に成功（`after=2375`）。
- `cargo test --all-targets --locked` は 121 件成功。
- 6 ゲート（fmt / check / test / clippy / wasm32 × 2）と `trunk build` が成功。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定（次ラウンド以降）

- E2E が読む `.reader-content` がスクロール適用後に差し替わる原因の特定。
  - `set_content` の再通知による `innerHTML` 再設定の可能性。
  - Leptos のリアクティブ更新による DOM 再マウントの可能性。
- `MutationObserver` で `.reader-content` の差し替えを監視し、0 に戻る
  タイミングを特定する。
- 原因特定後の根本修正（例: スクロールを `requestAnimationFrame` で
  再適用し、DOM 更新の後に確定させる）。
