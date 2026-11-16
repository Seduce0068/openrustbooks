---
type: Decision
title: "ADR-0058: 本文領域のアクセシブル名"
description: "本文領域の aria-labelledby を章タイトルに結びつける決定を記録する。"
tags:
  - ADR
  - web
  - accessibility
generated:
  by: dsh
  at: "2026-11-17T06:40:00+09:00"
status: accepted
---

## 背景

ADR-0057 の未決定「ページ見出しへの `aria-labelledby` 設定など、さらに詳細な本文構造の提供」。
`.reader-content` は `aria-label="Chapter content"` のみで、スクリーンリーダーでは
どの章を読んでいるか分かりにくかった。

## 決定

- ツールバーの章タイトルの `<span>` に `id="reader-chapter-title"` を付ける。
- `.reader-content` に `aria-labelledby="reader-chapter-title"` を追加する。
- 既存の `aria-label="Chapter content"` は後方互換（既存 E2E が検証している）
  のため維持する。ARIA 仕様で `aria-labelledby` が `aria-label` より優先される
  ため、スクリーンリーダーには章タイトルが読み上げられる。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| aria-labelledby を追加（採用） | 既存属性を壊さず章タイトルをラベルにできる |
| aria-label を削除して統一 | 既存 E2E と後方互換が壊れる |
| 本文の h1 を参照 | 動的な innerHTML の id に依存し壊れやすい |

## 影響

- スクリーンリーダーで本文領域に入ったときに章タイトルが読まれる。
- 視覚的な表示は変わらない。
- 追加依存は無い。

## 検証

- ヘッドレス Chrome 157 と `tools/verify-web-reader.py` の E2E 176 項目が
  5 回連続で成功。`.reader-content` の `aria-labelledby` が
  `reader-chapter-title` を指し、参照先に章タイトルが入ることを確認した。
- `cargo test --all-targets --locked` は 121 件成功。
- 6 ゲート（fmt / check / test / clippy / wasm32 × 2）と `trunk build` が成功。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- 本文の見出し構造（h1/h2）とランドマークの連携。
- `epub:type="noteref"` / `footnote` の明示的な解釈（ADR-0055 から継続）。
- `IntersectionObserver` による画像の遅延読み込み（ADR-0034 から継続）。
- EPUB 内 CSS、縦書き・ページ送り（ADR-0012 から継続）。
