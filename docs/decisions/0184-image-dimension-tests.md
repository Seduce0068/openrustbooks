---
type: Decision
title: "ADR-0184: 画像寸法パーサのテスト拡充"
description: "image_dimensions の JPEG / WebP ヘッダーと境界値テストを追加する決定を記録する。"
tags:
  - ADR
  - core
  - testing
generated:
  by: dsh
  at: "2026-11-19T05:10:00+09:00"
status: accepted
---

## 背景

`image_dimensions()` は PNG / JPEG / GIF / WebP をサポートするが、
unit test は PNG / GIF のみだった（R6 TD-M01）。
JPEG / WebP のヘッダー解析や境界値の回帰を検出できなかった。

## 決定

- production code は変更せず、テストのみ追加する。
- 追加テスト:
  - `jpeg_dimensions_from_sof0`: SOI + SOF0 の 12 バイトヘッダーで
    `(20, 10)` と `(3000, 4000)` を検証。
  - `webp_dimensions_from_vp8x`: VP8X 30 バイトヘッダーで
    `(20, 10)` と `(MAX_DIMENSION, MAX_DIMENSION)` を検証。
  - `webp_dimensions_from_vp8_lossy`: VP8 lossy 30 バイトヘッダーで
    `(20, 10)` と `(3000, 4000)` を検証。
  - `png_boundaries_table_driven`: 0 / 1 / MAX_DIMENSION / MAX_DIMENSION+1
    を width / height 両軸で表駆動検証。
- imagesize 0.15.0 のローカルソースとヘッダー配置を突合済み:
  - 全形式で `read_exact(&[u8;12])`（11 バイトの JPEG は失敗）。
  - JPEG は SOF0 の offset 7-8 = height、9-10 = width（**PNG と逆順**）。
  - VP8X は `(value - 1)` の u24 LE、VP8 lossy は u16 LE（+1 なし）。
- 期待総数は E2E 249 のまま。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| テストのみ追加（採用） | 回帰検知を高めつつ production を守る |
| production code 変更 | 不要（既存実装は imagesize 契約どおり） |

## 影響

- 新規依存なし、unsafe なし。
- `u32::try_from()` のオーバーフロー分岐は
  対応 4 形式（PNG u32 / JPEG・GIF u16 / WebP u24・14bit）では
  到達不能なためテスト対象外。
- VP8L は 14bit のため `MAX_DIMENSION + 1` を表現できず未テスト。
- VP8X / VP8 lossy の truncated ヘッダーも未テスト。

## 検証（Runtime Proof）

- `cargo test -p openrustbooks-core`: **220 passed**（新規 4 件含む、
  image_dimensions 絞り込みは 10 passed）。
- `cargo clippy --all-targets --locked -- -D warnings`、
  `./scripts/verify.sh`: 成功。
- E2E **249 項目すべて成功**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- JPEG / WebP の `MAX_DIMENSION + 1` 境界テスト
  （要件は PNG 表驱动的に網羅、形式横断の完全性はimprovement余裕）。
- VP8L のテスト方針。
- VP8X / VP8 lossy の truncated ヘッダーケース。
- 既存 `rejects_zero_and_oversized` と境界表の重複整理。
