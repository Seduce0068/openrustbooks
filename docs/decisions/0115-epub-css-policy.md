---
type: Decision
title: "ADR-0115: EPUB 内 CSS の適用方針"
description: "EPUB 内 CSS は現状どおり適用しないことを維持し、将来のオプトイン・スコープ付き段階導入の方針を記録する。"
tags:
  - ADR
  - epub
  - security
generated:
  by: dsh
  at: "2026-11-18T06:00:00+09:00"
status: accepted
---

## 背景

`core/src/sanitize.rs` は `style` タグを内容ごと削除し、`style=` 属性も
許可リストに入れていない（ADR-0012 の fail-closed 方針）。
このため EPUB 独自の組版が失われるが、安全に適用するには
`url()` / `@import` の完全拒否、セレクタのスコープ化、
CSP `style-src` との整合が必要。

## 現状（css-flow-researcher の調査）

- `sanitize_epub_html()` は `clean_content_tags` に `style` を含み、
  `removes_style_attributes_and_tags` テストが除去を固定している。
- `chapter_resource()` は CSS バイト列を取得できるが、
  スタイルシートとして解釈・適用するコードは存在しない。
- `resolve_images()` は `DOMParser` 後の `img[src]` だけを Blob URL 化し、
  `style` / `<link>` は扱わない。
- CSP は `style-src 'self' 'unsafe-inline' https://fonts.googleapis.com` で、
  `'unsafe-inline'` があるため `<style>` を入れれば適用されてしまう
  （サニタイザが主防御）。
- SVG 側では XML 処理命令による外部 CSS 読み込みを既に削除済み
  （ADR-0045）で、HTML 側だけ緩めるのは整合性を欠く。

## 決定

- **現状維持（CSS 非適用）を継続する**。コード変更は行わない。
- 将来導入する場合は、次の段階的方針を取る。
  1. オプトイン設定（既定無効）で、EPUB リーダー限定に適用する。
  2. CSS は `chapter_resource()` で取得し、core 側で厳格フィルタする。
  3. 挿入は `inner_html` ではなく CSSOM（`CSSStyleSheet.replaceSync()` 等）
     経由とし、全セレクタを `.reader-html ` で prefix する。
  4. 許可候補: 色、フォントサイズ／ウェイト／ファミリ（ローカル一般名のみ）、
     行間、文字間、text-align、余白、border 等。
  5. 必須拒否: 全 `url()`、`@import`、`@font-face`、`@page`、`!important`、
     `position: fixed|absolute|sticky`、`z-index`、疑似要素 `content`、
     `html` / `body` / `:root` / `*` セレクタ。
  6. 章切替時に stylesheet を破棄する（Blob URL の revoke と同様）。
  7. ADR-0091 の nonce 対応とセットで `style-src` から `'unsafe-inline'`
     を外す。`style=` 属性は nonce が効かないため再許可しない。
- CSS パーサーの新規追加は、crates.io 候補の調査（活発さ・ライセンス・
  脆弱性・Wasm サイズ）を経て別 ADR で判断する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 現状維持（採用） | ADR-0012 の決定を維持。SVG 側の対策との整合も保てる |
| `<style>` / `style=` をそのまま許可 | `url()` 通信・UI redressing・レイアウト破壊で却下 |
| iframe sandbox / Shadow DOM | ADR-0012 で iframe は既に却下。Shadow DOM はネットワークを隔離しない |

## 影響

- コード変更なし。既存の 195 テスト、E2E 228 項目はそのまま。
- 出版社組版は従来どおり再現されない（既知の制約）。

## 検証

- 2 観点のエージェント調査（コードフローと modern-web-guidance /
  OWASP / MDN / ammonia docs）を実施。
- コード変更が無いためゲートは既存の緑を維持。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 根拠（一次情報）

- [OWASP XSS Prevention Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Cross_Site_Scripting_Prevention_Cheat_Sheet.html)
- [MDN CSP style-src](https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Content-Security-Policy/style-src)
- [MDN CSP style-src-attr](https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Content-Security-Policy/style-src-attr)
- [ammonia 4.2.5 docs](https://docs.rs/ammonia/4.2.5/ammonia/)

## 未決定

- CSS パーサークレートの選定（新規依存の事前調査が必要）。
- オプトイン設定の DTO／UI 表示方法。
- `style-src-attr 'none'` への移行時期（ADR-0091 の nonce 対応と同時が望ましい）。
- 縦書き・ページ送り（ADR-0012 から継続）。
