---
type: Decision
title: "ADR-0039: 強制カラーモードへの対応"
description: "Windows のハイコントラスト（Forced Colors Mode）でリーダーの視認性を確保する決定を記録する。"
tags:
  - ADR
  - epub
  - web
  - accessibility
generated:
  by: dsh
  at: "2026-11-17T00:18:00+09:00"
status: accepted
---

## 背景

ADR-0038 まででライト/ダークと `prefers-color-scheme` 追従に対応したが、
Windows のハイコントラスト（強制カラーモード）ではブラウザが作者指定の色を
システムカラーに上書きし、ボタンや境界の視認性が失われる恐れがある
（ADR-0033 の未決定「`forced-colors` や `prefers-contrast` への対応」）。

## 決定

### 強制カラーモードのスタイル

- `@media (forced-colors: active)` で主要な色トークンをシステムカラー
  キーワード（`Canvas` / `CanvasText` / `ButtonText` / `Highlight` /
  `GrayText` / `LinkText` / `ButtonFace`）に上書きする。
- ボタン・入力欄の境界は `border` で `CanvasText` を使い、背景色や
  `box-shadow` だけに依存しない。
- `:focus-visible` のアウトラインを `2px solid Highlight` + `outline-offset: 2px`
  で明示し、フォーカスリングが消えないようにする。平時はブラウザ既定の
  フォーカス表示に任せる（既存の挙動を変えない）。

### `forced-color-adjust`

`forced-color-adjust: none` は使わない。色が情報そのものになる要素は本文画像
だけだが、それは `<img>` でブラウザが適切に扱うため、作者が色を固定する
必要はない。

### `prefers-contrast`

今回は対応しない。既存の配色はコントラストの最小要件を満たしており、
ガイドの指摘どおり `prefers-contrast: more` は「低コントラストのアクセントを
補強する場合に限る」ため、現時点で必要性がない。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| システムカラーキーワードで上書き（採用） | 追加依存なし、OS の配色に自動追従する |
| `forced-color-adjust: none` で作者色を維持 | 利用者が選んだハイコントラスト配色を奪う |
| 対応しない | ハイコントラスト利用者が操作要素を識別できない |

## 影響

- Windows のハイコントラスト有効時でも、ボタン・入力欄・フォーカスリングが
  システムカラーで視認できる。
- 平時の表示は一切変わらない（`@media (forced-colors: active)` の中だけ
  上書きしている）。
- 追加依存は無く、既存の CSS だけで実装している。

## 検証

- ヘッドレス Chrome 157 と `tools/verify-web-reader.py` の E2E 143 項目が
  5 回連続で成功。CDP の `Emulation.setEmulatedMedia` で
  `forced-colors: active` をエミュレーションし、強制カラーモードが有効に
  なること、ボタンの境界が残ること、章移動ができることを確認した。
- `cargo test --all-targets --locked` は 119 件成功。
- 6 ゲート（fmt / check / test / clippy / wasm32 × 2）と `trunk build` が成功。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- `prefers-contrast: more` への対応（必要になった時点で判断）。
- `forced-colors` とダークテーマの併用時の詳細な配色調整。
- EPUB 内 CSS、縦書き・ページ送り（ADR-0012 から継続）。
