---
type: Decision
title: "ADR-0136: EPUB 画像の寸法予約方針"
description: "画像の自然寸法を width/height 属性で予約する段階導入の設計方針を記録する。"
tags:
  - ADR
  - epub
  - web
  - performance
generated:
  by: dsh
  at: "2026-11-18T11:45:00+09:00"
status: accepted
---

## 背景

ADR-0112 は画像遅延読み込みの前に「自然寸法を `width` / `height` で予約」する方針を決定し、
「どの層で取得・埋め込むか」を未決定として残した（ADR-0112:86）。
現状 `EpubCoverImage` はバイト列のみ、`resolve_images()` は Blob URL 化のみで
属性付与は行わない。

## 現状（2 観点の調査）

- `core/src/sanitize.rs:94` は `img` の `width` / `height` を既に許可
  （allowlist 変更不要）。ただし値の検証は行わない。
- `web/src/lib.rs:911-953` の `resolve_images()` が `chapter_resource()` の
  バイト列を Blob URL 化。`src` 解決失敗時は `src` のみ削除する。
- `.reader-html img { max-width:100%; height:auto }`（`web/index.html:401-404`）。
  両属性を与えるとブラウザが `aspect-ratio: auto W/H` を派生させ、
  ロード前にボックスを確保できる（**推測**を含む）。
- 画像デコード系クレートは workspace に直接依存なし。
  `png 0.18.1` は desktop 側の推移依存としてロックに存在するのみ。
- `images_settled()`（web/src/lib.rs:387-426）は `complete() && naturalWidth > 0`
  を要求。巨大画像警告（ADR-0064）と縮小（ADR-0067）も `naturalWidth/Height` 依存。

## 決定

- **今回は実装しない**（GOAL の制約：調査と決定のみ）。段階導入の方針を固定する。
- 配置:
  - 寸法取得は **core** の pure 関数
    `image_dimensions(bytes) -> Option<(u32, u32)>`（native / wasm 共通）。
  - 属性注入は **web の `resolve_images()`**（`chapter_resource()` が
    バイト列を持ち、DomParser 直後に `set_attribute` すれば
    `body.inner_html()` に反映される）。`sanitize_epub_html(&str)` は
    バイト列を持たないため core 側注入は API 変更が必要で非推奨。
- 形式:
  - PNG / JPEG / WebP / GIF: `imagesize 0.15.0`（MIT・軽量）を第一候補とする。
    新規依存を一切認めない場合は自作ヘッダーパーサへ落とさず**見送り**。
  - SVG: DomParser でルート要素の `width` / `height` を読み、既存値を尊重。
    viewBox のみの場合は絶対サイズがブラウザ依存（ADR-0113 の推測）のため
    最初の段階では付与しない。
- 付与規則:
  - 両方の属性を自然ピクセルで埋める（片方だけでは比率が決まらない）。
  - 属性は「縦横比の予約」専用とし、巨大画像警告・縮小の判定入力には使わない。
  - 値は 0 / 非数 / 単位付き / 上限超（例 16384px）を拒否し、
    取得できない画像には属性を付けない（fail-closed）。
  - `src` を削除した画像は `width` / `height` も削除する（ゴーストボックス対策、
    **推測**を含むため実装時に E2E 確認）。
- **非目標**: `loading="lazy"`（ADR-0112 / 0034）、EPUB 内 CSS（ADR-0115）、
  `images_settled()` の緩和、警告・縮小基準の変更。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| core 取得＋web 注入（採用） | 双方の責務を明確にし、既存 API を活かせる |
| core サニタイズ時に注入 | バイト列が無く API 変更が必要 |
| web でデコード後に計測 | CLS 対策にならず、描画経路も複雑化 |
| 自作ヘッダーパーサ | 車輪の再発明で非推奨 |

## 影響

- コード変更なし。既存の 246 テスト＋wasm 3、E2E 229 項目は維持。
- 実装時に位置復元系 E2E（ADR-0020 / 0037 / 0046）の回帰テストを先に整備する。

## 検証

- 2 観点のエージェント調査（寸法 API とレイアウト / E2E）を実施。
- コード変更が無いためゲートは既存の緑を維持。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- `imagesize` の wasm32 ビルド可否と feature 絞り込みの影響（未ビルド確認）。
- 作者指定 `width` / `height` の扱い（信頼／上書き／クランプ）。
- 縮小後の属性更新の有無（比率の丸れによる微シフト、推測）。
- viewBox のみ SVG の既定 natural size（ADR-0113 から継続、推測）。
- CLS 直接計測 E2E（`PerformanceObserver('layout-shift')`）の導入。
- `.reader-cover`（`web/index.html:339-345`）を対策範囲に含めるか。
- `IMAGE_SETTLE_MS = 2000` の根拠（ADR-0113 から継続）。
- 5 ブラウザ実機検証（現状は Chrome 157 のみで実証）。
