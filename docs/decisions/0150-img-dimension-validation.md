---
type: Decision
title: "ADR-0150: img 寸法属性のサニタイズ検証"
description: "EPUB HTML の img の width/height を数値検証し、不正値を削除する決定を記録する。"
tags:
  - ADR
  - epub
  - security
generated:
  by: dsh
  at: "2026-11-18T19:15:00+09:00"
status: accepted
---

## 背景

ADR-0149 で画像の自然寸法予約を実装した。しかし
`core/src/sanitize.rs:94` は `img` に `width` / `height` を許可する一方、
`attribute_filter` は `src` のローカル参照のみを検証していた。
ラスター画像は自然寸法で上書きされるが、SVG は既存属性を尊重するため、
作者指定の不正値（単位付き・極端な数値等）が残り得た。

## 決定

- `image_dimensions::MAX_DIMENSION`（16_384）を再利用し、
  `is_valid_dimension()` を追加する。
  - ASCII 数字 1〜5 桁のみ
  - 値は `1..=MAX_DIMENSION`
  - 空 / 単位付き / 負 / 小数 / 0 / 上限超 / 非 ASCII 数字（全角等）は `None`
- `attribute_filter` で `img` の `width` / `height` に適用し、
  無効値は属性ごと削除する（fail-closed）。
- `img` の `src` に対する `is_local_reference()` 検証は維持。
- 検証対象は **HTML の `img` 要素のみ**。
  SVG ルート要素の属性は対象外（ADR-0149 の SVG 既存属性尊重を維持）。
- Web 側 `parse_svg_dimension()` は `trim()` と `parse::<u32>()` を使うため
  受理範囲が core と非対称だが、既存属性がある場合は付与しないため
  結果は fail-closed 方向（**推測**）として許容する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| サニタイザで属性単位に検証（採用） | 上書き経路に依存せず 안전 |
| Web 側だけ検証 | core のサニタイズを通る他経路で不正値が残る |
| 属性を全面禁止 | ADR-0149 の寸法予約と矛盾 |

## 影響

- 作者指定の `width="100"` / `height="200"` は維持。
- `width="100px"` / `-1` / `0` / `16385` / 全角 `１２３` は削除。
- 先頭ゼロ `007` と境界値 `16384` は維持（数値として有効）。
- 新規依存なし、unsafe なし。

## 検証（Runtime Proof）

- 追加テスト 4 件（有効値維持、不正値削除、境界と先頭ゼロ、
  外部 src 削除との並存）。
- `cargo test --all-targets --locked`: core **196** + desktop 56 + web 2、
  wasm-test 4。
- `cargo audit --no-fetch`: 脆弱性 0、警告 baseline 10。
- `./scripts/verify.sh` rc=0、E2E **231 項目すべて成功**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- 作者指定 width/height を自然寸法で上書きする現在の web 側挙動は
  ADR-0136 / 0149 の未決定を完全に解消していない。
- 先頭ゼロ `007` が CLS 予約値として問題ないか（数値としては 7）。
- core / web の受理範囲の非対称（`+20` 等）を統一するか。
- 上限 16384 の最終根拠、CLS 直接計測 E2E、
  5 ブラウザ実機検証（ADR-0149 から継続）。
