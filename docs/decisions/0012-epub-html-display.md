---
type: Decision
title: "ADR-0012: EPUB の HTML をサニタイズして表示する"
description: "章 XHTML の許可リスト方式サニタイズ、画像の Blob URL 化、アンカー移動の決定を記録する。"
tags:
  - ADR
  - epub
  - web
  - wasm
  - security
generated:
  by: opencode
  at: "2026-11-03T02:30:00+09:00"
status: accepted
---

## 背景

ADR-0011 は Web 版の `Reader` に `chapter_text()` を `<pre>` で表示する
最小 UI を実装し、`chapter_html()` の `inner_html` 挿入をサニタイズ方針が
未決定であるとして避けた。その後、core には `chapter_resource()`、
`cover_image()`、`chapter_title()`、TOC のフラグメントが揃ったが、
Web UI はテキスト表示のままで、EPUB の見出し・強調・画像・表を
表示できていなかった。

## 決定

EPUB の章 XHTML を許可リスト方式でサニタイズし、画像を Blob URL に
置き換えて `inner_html` に設定する。

### サニタイズ

- `core::sanitize::sanitize_epub_html` を追加し、`ammonia` 4.2 を使う。
  新規依存は `ammonia` のみ。
- 許可する要素は、本文の構造・見出し・段落・リスト・表・引用・コード・
  ルビ・画像・リンクに限定する。`script`・`style`・`iframe`・`object`・
  `embed`・`form`・`input`・`button`・`select`・`textarea`・`link`・
  `meta`・`base` は許可しない。
- `script`・`style`・`title`・`template`・`noscript` は内容ごと削除する。
- 共通属性は `id`・`class`・`lang`・`dir`・`title`・`role` に限定する。
  イベントハンドラ属性と `style` 属性は許可しない。
- `a` は `http`・`https`・`mailto` の `href` を許可し、
  `rel="noopener noreferrer"` を付与する。`javascript:` と `data:` は
  削除する。
- `img` は相対 URL の `src` だけを許可する。スキーム付き URL、
  プロトコル相対 URL (`//host/...`)、`data:` は削除する。
- CSS は適用しない。EPUB 内のスタイルシートとインラインスタイルを
  使わないため、出版社の組版は再現しない。

### 画像と表紙

- Web 側はサニタイズ済み HTML を `DOMParser` で解析し、`img[src]` を
  `WasmEpubBook::chapter_resource` で解決する。
- 解決したバイト列を拡張子から推定した MIME タイプの `Blob` にし、
  `URL.createObjectURL` で Blob URL を生成して `src` に設定する。
- 解決できない画像は `src` を削除し、alt テキストだけを残す。
- Blob URL は章の切り替えと本の開き直しで `revokeObjectURL` する。
- 表紙は `cover_image()` のバイト列を同じ方法で Blob URL にし、
  サイドバーに表示する。

### 章内移動

- TOC エントリの `fragment` を章の選択肢に保持し、章を表示した後に
  `document.getElementById(fragment).scrollIntoView()` で移動する。
- 本文内の `#fragment` リンクは同じ方法でスクロールする。外部リンクは
  `window.open` で別タブに開く。相対リンクによる章の切り替えは未対応。

### 表示

- `Reader` の本文を `<pre>` から `div.reader-html` に変更し、
  サニタイズ済み HTML を `inner_html` に設定する。
- 表紙・章タイトル・前後移動・章選択は既存のツールバーを拡張する。
- Markdown プレビューも同じサニタイザを通し、生 HTML の挿入をやめる。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| `ammonia` による許可リスト（採用） | Rust/Wasm で動作し、HTML5 パーサと実績のある許可リストを持つ |
| `iframe sandbox` に XHTML を入れる | Blob URL・base URL・CSP の設計が増え、親画面との連携も複雑になる |
| DOMPurify を JavaScript で使う | JS 依存とバンドル設定が増え、Rust 側のテストで検証できない |
| 自前で `quick-xml` から再構築 | 壊れた HTML の補正と許可リストの維持コストが高い |
| EPUB の CSS をそのまま適用 | `url()` による外部参照とレイアウト崩れの対策が未決定 |

## 影響

- 章の見出し・強調・リスト・表・ルビ・画像をブラウザで表示できる。
- EPUB の CSS、インライン SVG、外部画像、フォーム、メディアは表示しない。
- Blob URL の寿命管理を UI が担当する。章を切り替えるたびに作り直す。
- `ammonia` の追加により core の依存が 1 つ増える。Wasm のサイズも増える。
- サニタイズは core の純粋関数であり、native のテストで検証できる。

## 検証

- `core` のサニタイズ 10 テスト（script、イベントハンドラ、埋め込み、
  フォーム、style、外部画像、javascript:、表、ID、ラッパー除去）。
- `cargo test -p openrustbooks-core` は 105 件成功。
- `cargo check -p openrustbooks-web --target wasm32-unknown-unknown` 成功。
- 5 ゲートと `trunk build` の結果は実装コミットで確認する。

## 未決定

- EPUB 内 CSS の安全な適用（`url()` の除去、スコープ付きスタイル）。
- インライン SVG と MathML の表示。
- 音声・動画・メディアオーバーレイ。
- 相対リンクによる章間移動と、履歴・読書位置の保存。
- 大容量 EPUB のメモリ使用と、画像のデコード・縮小。
- 縦書き・ページ送り・表示設定。
