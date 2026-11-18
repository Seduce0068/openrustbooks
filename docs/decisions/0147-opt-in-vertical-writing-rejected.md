---
type: Decision
title: "ADR-0147: オプトイン縦書きの一旦見送り"
description: "縦書きのアプリ側実装を試行したが、位置保存・復元を検証できなかったため見送った決定を記録する。"
tags:
  - ADR
  - web
  - epub
generated:
  by: dsh
  at: "2026-11-18T18:40:00+09:00"
status: rejected
---

## 背景

ADR-0145 の第二段として、Web `ReaderSettings` に `writingMode` を追加し、
既定は横書きのままオプトインで縦書きを試す実装を行った。

## 試行した内容

- `ReaderSettings.writing_mode`（既定 `horizontal-tb`）とツグルボタン
  `#reader-writing-mode` を追加。
- `.reader-container.vertical-writing` 配下で
  `.reader-content` に `writing-mode: vertical-rl` を適用。
- アプリ側のスクロール読み書き・フラグメント移動を
  `scrollTop` / `scrollLeft` と `rect.top` / `rect.left` へ分岐。
- 保存 JSON のキー 4 つと `scrollTop` というレガシー名は維持。
- 監査指摘を受け、トグル時に両軸をリセットし保存位置も 0 に更新した。

## 検証結果

- 横書き E2E は 229/0、全ゲート成功。
- CDP 手動検証では `writing-mode: vertical-rl`、
  クラス付与、`aria-pressed=true` を確認。
- しかし横方向 overflow を作った状態でも
  `scrollLeft` が 0 のまま変化せず、
  localStorage の `scrollTop` も 0 のままだった
  （`scrollWidth=6150` / `clientWidth=1136` の環境で再現）。
- リロード後の位置復元も `scrollLeft=0` であり、
  GOAL の受入基準「位置保存／復元が動くこと」を満たせなかった。

## 決定

- 縦書きのアプリ側実装は **一旦見送り**、`web/src/lib.rs` と
  `web/index.html` を HEAD に戻した。
- 画像縮小の安定化（ADR-0148）を先に完了させ、
  縦書きは ADR-0145 / 0146 の方針を維持したまま後日再検討する。
- 次回は `vertical-rl` の実際のスクロール軸と符号を
  5 ブラウザで実機確認してから保存スキーマを設計する。

## 影響

- ユーザー向けの縦書きツグルは提供されない（現状どおり横書きのみ）。
- E2E の軸非依存ヘルパー（ADR-0146）は維持し、横書きベースラインは健全。
