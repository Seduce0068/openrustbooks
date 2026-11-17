---
type: Decision
title: "ADR-0068: 表示設定の一括リセット"
description: "増えた表示設定をまとめて既定値に戻すボタンを追加する決定を記録する。"
tags:
  - ADR
  - epub
  - web
generated:
  by: dsh
  at: "2026-11-17T09:10:00+09:00"
status: accepted
---

## 背景

文字サイズ・行間・ダーク・OS 追従・スワイプ閾値・画像縮小など表示設定が
増え、これらを個別に既定へ戻すのは手間だった。設定が分かりにくくなった
ときの復旧手段として一括リセットが必要。

## 決定

- ツールバーに `<button id="reader-reset-settings">` を置く。
- クリックで全表示設定を既定値に戻す。
  - `fontScale=1`, `lineHeight=1`, `dark=false`, `followSystem=false`
  - `swipeThreshold=50`, `verticalScrollThreshold=30`, `downscaleImages=false`
- localStorage の設定キーも `ReaderSettings::default()` で上書きする。
- 状態メッセージに `Display settings reset.` を表示する。
- 読書位置・履歴・エクスポート・インポートは触らない。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 一括リセットボタン（採用） | 復旧が 1 クリックで済む |
| 個別に戻す | 設定が増えた今は手間がかかる |
| `Clear data` に兼ねさせる | 読書位置データまで消え、役割が異なる |

## 影響

- 表示設定だけが既定に戻り、読書位置や履歴は維持される。
- 既存の `Clear data`（読書位置の削除）とは役割を分離したまま。
- 追加依存は無い。

## 検証

- ヘッドレス Chrome 157 と `tools/verify-web-reader.py` の E2E 197 項目が
  8 回中 7 回成功（1 回は初期表示の環境フレークで、再実行で成功）。
- 設定変更後にリセットすると、全 7 項目が既定値へ戻ることと
  `Display settings reset.` が表示されることを確認した。
- `cargo test --all-targets --locked` は 125 件成功。
- 6 ゲート（fmt / check / test / clippy / wasm32 × 2）と `trunk build` が成功。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- リセット操作の確認ダイアログ（誤操作の可能性は低いため今回は省略）。
- `IntersectionObserver` による画像の遅延読み込み（ADR-0034 から継続）。
- EPUB 内 CSS、縦書き・ページ送り（ADR-0012 から継続）。
