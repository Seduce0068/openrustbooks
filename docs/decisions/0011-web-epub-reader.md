---
type: Decision
title: "ADR-0011: Web 版でローカル EPUB をテキスト閲覧する"
description: "WasmEpubBook を使ったブラウザ内 EPUB 読書の初期 UI と安全な表示境界を記録する。"
tags:
  - ADR
  - epub
  - web
  - wasm
  - security
generated:
  by: opencode
  at: "2026-10-18T14:40:22+09:00"
status: accepted
---

## 背景

ADR-0010 で `WasmEpubBook` を JavaScript に公開したが、Web の `Reader` は
プレースホルダーのままで、利用者が EPUB を開く経路がなかった。章 XHTML の
描画にはサニタイズと相対リソース URL の設計が必要で、まだ決定していない。

## 決定

Web 版 `Reader` にローカル `.epub` ファイルを開く最小 UI を実装する。

- ブラウザ File API で選択ファイルを読み、`WasmEpubBook` に渡す。ファイルは
  サーバーへ送信しない。
- spine の章を選択できる UI を出し、TOC の表示名が一致すればラベルに使う。
  ラベルが無い場合は章のアーカイブ内パスを使う。
- 本文は `chapter_text()` の結果を `<pre>` のテキストノードとして表示する。
  `chapter_html()` を `inner_html` に挿入しない。画像・CSS 等のリソースもまだ表示しない。
- 前後ボタンと章選択を用意する。目次フラグメントに基づく章内移動はしない。
- XHTML の描画、サニタイズ、Blob URL 化、保存、デスクトップ版は今回の範囲外。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| `chapter_text()` をテキスト表示（採用） | 未サニタイズ XHTML を実行可能な HTML として挿入しない |
| `chapter_html()` を `inner_html` に挿入 | スクリプト・危険属性・外部 URL の対策が未決定のため不採用 |
| React 等の別 UI 実装を追加 | 既存 Leptos UI と二重化するため不採用 |

## 影響

- ブラウザで EPUB をローカル選択し、章を切り替えて本文テキストを読める。
- 本文 HTML の書式、画像、CSS、ページ送り・縦書きは利用できない。
- 読み込みサイズによるメモリ使用については別途計測が必要。

## 検証

- `cargo check -p openrustbooks-web --all-targets`、core/web の wasm check、
  `cargo test --all-targets`、`cargo clippy --all-targets -- -D warnings` が成功。
- `trunk build --offline --dist <一時出力先>` が成功。
- 表示はテキストノードのみで、`inner_html` を使用しないことをコードレビューで確認。

## 未決定

- XHTML の安全なサニタイズ、スタイル・画像リソースの URL 化。
- 目次フラグメントによる章内移動と、表示位置の保持。
- 大容量 EPUB のメモリ使用と、対象ブラウザでの実機検証。
