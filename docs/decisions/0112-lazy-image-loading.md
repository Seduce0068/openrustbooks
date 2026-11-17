---
type: Decision
title: "ADR-0112: 画像遅延読み込みの判断"
description: "IntersectionObserver による画像遅延読み込みは全面切替を見送り、寸法予約→段階導入の順で検討する決定を記録する。"
tags:
  - ADR
  - web
  - performance
generated:
  by: dsh
  at: "2026-11-18T05:20:00+09:00"
status: accepted
---

## 背景

多数の画像を含む EPUB では表示前の画像まで読み込むため初期表示が遅い。
`IntersectionObserver` feature は `web/Cargo.toml` / `core/Cargo.toml` に
宣言済みだが未使用（image-flow-researcher の確認）。
ADR-0020 / 0034 / 0046 で「遅延読み込みとの併用」が未決定だった。

## 現状（image-flow-researcher の調査）

- `resolve_images()` (web/src/lib.rs:789-831) が全画像を Blob URL 化し `src` を設定。
  SVG は `sanitize_svg_bytes()` でサニタイズ後 Blob 化（fail-closed）。
- `decoding="async"` は付与済み、`loading="lazy"` は明示的に不使用。
- 完了判定は `images_settled()` (277-304): `src` あり画像が全て `complete()`
  になるか、`IMAGE_SETTLE_MS = 2000` で打ち切り。
- 位置補正 `apply_position_frame()` (551-647) は画像待ち→scrollTop 適用→retry。
- 同章内 `#id` リンク (2463, 2518) は `scroll_to_fragment()` を即時呼びし、
  画像待ち・retry が無い。
- 巨大画像警告・縮小は `naturalWidth/Height` に依存する。
- E2E は `naturalWidth/Height` 待ちと `loading="lazy"` 非付与の検証を含む
  （期待総数 228）。

## 決定

- **全面切替は見送る**。最初に検討すべきは `IntersectionObserver` ではなく、
  EPUB 画像の自然寸法を `width` / `height` 属性として埋め込み、 layouts を
  先に確定させること（CLS 対策）。
- 遅延読み込みを導入するなら、次を前提にした**段階導入**とする。
  1. 保存位置近傍の画像は observer に任せず強制ロードする。
  2. 同章内フラグメント移動も画像待ち・retry 経路に統一する。
  3. `images_settled()` を「可視／位置に影響する画像」のレイアウト確定待ちに
     置き換える（`complete` はレイアウト確定の保証ではない）。
  4. observer コールバックに世代ガード（ADR-0054）を付け、
     章切替時の Blob URL revoke と競合させない。
  5. E2E を再設計し、期待総数を更新する。
- 既存の `decoding="async"` と CSP `img-src 'self' blob: data:` は維持する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 全面 IO 切替 | 位置復元・フラグメント・警告/縮小・E2E の 5 系統が同時に壊れるため却下 |
| 寸法予約→段階導入（採用） | 中核機能（位置復元）を守りつつ初期表示を改善できる |
| 現状維持 | 画像の多い本の初期表示が遅いまま |

## 影響

- コード変更は無い。既存の 193 テスト、E2E 228 項目はそのまま。
- 今後の実装は ADR-0020 / 0034 / 0046 の回帰テストを最初に整備する必要がある。

## 検証

- 2 観点のエージェント調査（コードフローと modern-web-guidance）を実施。
- コード変更が無いためゲートは既存の緑を維持。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 根拠（modern-web-guidance / 一次情報）

- [web.dev: lazy-loading](https://web.dev/lazy-loading) （`loading="lazy"` が標準推奨、距離は API 不可変）
- [MDN: Intersection Observer API](https://developer.mozilla.org/en-US/docs/Web/API/Intersection_Observer_API)
- [MDN: HTMLImageElement.complete](https://developer.mozilla.org/en-US/docs/Web/API/HTMLImageElement/complete)
  （`complete` はレイアウト確定を保証しない）

## 未決定

- EPUB の自然寸法をどの層で取得・埋め込むか（core の OPF/画像解析 vs web の事前デコード）。
- 保存位置近傍の「近傍」の定義（Observer の `rootMargin` 値）。
- 段階導入時の既定値（最初はオプトイン）。
- EPUB 内 CSS、縦書き・ページ送り（ADR-0012 から継続）。
