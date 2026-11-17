---
type: Decision
title: "ADR-0067: オプトインの画像縮小"
description: "巨大画像を Canvas で縮小する設定を既定オフで追加する決定を記録する。"
tags:
  - ADR
  - epub
  - web
  - performance
generated:
  by: dsh
  at: "2026-11-17T09:00:00+09:00"
status: accepted
---

## 背景

ADR-0064 で巨大画像の警告を追加したが、メモリ使用量そのものは未解消。
画像の自動縮小は既存 E2E（naturalHeight === 4000）やスクロール位置補正に
影響するリスクがあるため、既定オフのオプトインとして追加する。

## 事前調査

- 既存ソリューション: `web-sys` の `HtmlCanvasElement` / `CanvasRenderingContext2d`
  を使えば、新規クレートなしで画像の再描画と Blob 化ができる。
- 新規依存クレートは追加せず、既存 `web-sys` の feature
  （`HtmlCanvasElement`, `CanvasRenderingContext2d`）だけを有効化した。
- 外部サービスへの送信は無く、すべてブラウザ内で完結する。

## 決定

- `ReaderSettings` に `downscaleImages` を追加。既定 false、既存設定にキーが
  無い場合は false（後方互換）。保存・エクスポート・インポート対象。
- ツールバーにトグルボタン（`<button id="reader-downscale">`）を置く。
- 有効時、本文画像のうち次を満たすものだけを縮小する。
  - naturalWidth または naturalHeight が 4000px を超える。
  - naturalWidth が本文領域の幅より大きい（CSS の `max-width: 100%` で
    表示幅が決まる画像だけ。狭い画像を縮小すると表示サイズが変わり
    スクロール位置補正が壊れるため除外）。
- 長辺が `max(2000, 本文領域の幅)` になる比率で Canvas に描き直し、
  Blob URL に差し替える。古い URL は revoke し、新しい URL は既存の
  管理リストに追加して章切り替え時に破棄する。
- 表示サイズは変わらないため、スクロール位置の再適用は行わない。
- 無効時は何もしない（既定の挙動を完全に維持）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| オプトインの縮小（採用） | 既存利用者に影響せず、必要な人だけが有効化できる |
| 常に自動縮小 | 既存 E2E と位置補正を壊すリスクがある |
| 警告のみ（現状） | メモリ使用量は解消されない |

## 影響

- 有効にした利用者は、巨大画像のメモリ使用量を抑えられる。
- 縮小は再エンコード（PNG）のため、元画像の画質・色が若干変わる可能性がある。
- 画像の論理的な表示サイズは変わらない（CSS が表示幅を決めるため）。
- 既存の警告（ADR-0064）は縮小前のサイズに基づいて表示される。

## 検証

- ヘッドレス Chrome 157 と `tools/verify-web-reader.py` の E2E 193 項目が
  5 回連続で成功。
  - 既定では 4500x100 の画像が縮小されない。
  - トグル有効時は自然幅が 2000px 以下に縮小される。
  - トグルの `aria-pressed` が更新される。
- 途中で E2E のリロード後に nodeId が古くなる問題が発生したが、
  毎回 `DOM.getDocument` を取得するよう修正して解消した。
- `cargo test --all-targets --locked` は 125 件成功。
- 6 ゲート（fmt / check / test / clippy / wasm32 × 2）と `trunk build` が成功。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- 縮小後の画質比較と、JPEG 再エンコードの選択。
- `IntersectionObserver` による画像の遅延読み込み（ADR-0034 から継続）。
- EPUB 内 CSS、縦書き・ページ送り（ADR-0012 から継続）。
