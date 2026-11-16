---
type: Decision
title: "ADR-0043: SVG Blob URL のサニタイズ"
description: "SVG 画像を Blob URL 化する前にスクリプト実行経路を除去する決定を記録する。"
tags:
  - ADR
  - epub
  - web
  - security
generated:
  by: dsh
  at: "2026-11-17T01:18:00+09:00"
status: accepted
---

## 背景

ADR-0013 で SVG 画像は Blob URL で表示する判断をしたが、
「blob URL を直接開いた場合のスクリプト実行余地」が未決定として残っていた。
`<img>` 文脈ではスクリプトは実行されないが、Blob URL を新しいタブで開くと
アプリのオリジンで SVG が評価され、`<script>` や `on*` 属性が実行される
可能性がある（localStorage へのアクセス等）。

## 決定

### SVG サニタイズ

- MIME が `image/svg+xml` のリソースについて、Blob URL を作る前に
  SVG を `image/svg+xml` としてパースし、以下を除去する。
  - `<script>` 要素
  - `<foreignObject>` 要素（HTML スクリプトの混入経路）
  - SMIL 要素（`<set>` / `<animate>` / `<animateMotion>` /
    `<animateTransform>`）。後から `on*` 属性や `href` を書き換えられる。
  - `on*` イベントハンドラ属性
  - `href` / `xlink:href` の `javascript:` スキーム
- `javascript:` の判定では、ブラウザが無視する空白・制御文字を判定前に
  取り除く（`java\nscript:` のような回避を防ぐ）。
- パース・シリアライズは DOM API（`DomParser` / `outerHTML`）を使い、
  新規依存・新規 web-sys feature を追加しない。
- サニタイズに失敗した場合は Blob URL を作らず、`src` を削除する
  （fail-closed）。
- 非 SVG 画像（PNG / JPEG / GIF / WebP 等）は一切変更しない。

### 既存判断との関係

ADR-0012 / ADR-0013 で「DOMPurify などの外部ライブラリを追加しない」と
判断した方針を維持する。今回はブラウザ標準の DOM API だけを使う。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| DOM API でサニタイズ（採用） | 依存追加なし、Blob 生成時に確実に除去できる |
| DOMPurify を追加 | ADR-0012 / 0013 の方針に反し依存が増える |
| `<img>` のサニタイズだけに頼る | Blob URL を直接開かれた場合に無防備 |

## 影響

- SVG を含む EPUB でも、Blob URL を直接開かれてもスクリプトが実行されない。
- 通常の SVG 画像（図形・テキスト・グラデーション等）は従来どおり表示される。
- SMIL アニメーションを使った SVG は静止画として表示される（画像としての
  利用が主目的のため、受容可能な制約）。
- 追加依存は無く、既存の `web-sys` 機能の範囲で実装している。

## 検証

- ヘッドレス Chrome 157 と `tools/verify-web-reader.py` の E2E 154 項目が
  5 回連続で成功。スクリプト・onload・foreignObject・javascript リンク・
  SMIL を含む SVG について、Blob の内容からこれらが除去され、
  `window.__pwned` が設定されないことを確認した。
- 敵対的レビューで 2 つのバイパス（改行入り `javascript:` と SMIL による
  属性書き換え）を発見し、対策と回帰テストを追加した。
- `cargo test --all-targets --locked` は 119 件成功。
- 6 ゲート（fmt / check / test / clippy / wasm32 × 2）と `trunk build` が成功。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- `<use href="data:...">` や `<image href="http://...">` による外部リソース
  参照の制御（プライバシー懸念。スクリプト実行ではない）。
- XML 処理命令（`<?xml-stylesheet?>`）の扱い。
- `IntersectionObserver` による画像の遅延読み込み（ADR-0034 から継続）。
- EPUB 内 CSS、縦書き・ページ送り（ADR-0012 から継続）。
