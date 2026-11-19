---
type: Decision
title: "ADR-0249: 表紙 media-type 契約の回帰固定"
description: "未宣言の media-type を許容し明示された非画像を拒否する ADR-0008 の契約を unit test で固定する決定を記録する。"
tags:
  - ADR
  - epub
  - testing
generated:
  by: dsh
  at: "2026-11-19T22:34:00+09:00"
status: accepted
---

## 背景

`is_image_media_type()` は ADR-0008 に従い、
未宣言（空文字）を許容し、明示された `image/` 以外を拒否する。
既存 test は「非画像は表紙なし」と「`image/png` は読める」を
カバーしていたが、「未宣言を許容する」回帰 test が無く、
単純な `starts_with("image/")` 化で ADR 違反になっても検出できなかった
（R6 R1-archive#5）。

## 決定

- `is_image_media_type()` の unit test を 1 件追加する:
  - `""` → true（未宣言は判定材料が無いため許容）
  - `"image/png"` → true
  - `" IMAGE/SVG+XML "` → true（前後空白と ASCII 大小文字を正規化）
  - `"text/html"` → false（明示された非画像は拒否）
- 本番コードは変更しない。
- E2E check 名・件数は変更しない（unit test のみ、255 維持）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| unit test で契約を固定（採用） | 既存決定を壊さず回帰を検出できる |
| 空 media-type を拒否 | ADR-0008 の「不完全な OPF を排除しない」決定に反する |
| E2E を追加 | core の純粋判定であり不要（総数ゲートを動かさない） |

## 影響

- core test が 278 → **279**。
- SVG の `image/svg+xml` 受入は media-type 文字列判定に限り、
  描画／サニタイズ範囲を拡張するものではない
  （`sanitize.rs` と E2E の既存検証を維持）。
- E2E 255 維持。

## 検証（Runtime Proof）

- `cargo test --all-targets --locked`: core **279** / desktop 79 / web 2。
- `wasm-pack test --node --locked`: 15 passed。
- `cargo clippy --all-targets --locked -- -D warnings`、
  wasm32 core / web check、`./scripts/verify.sh`: すべて成功。
- E2E **255 項目すべて成功**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- 空 media-type 許容の再評価（ADR-0240 の未決定を集約）。
- WebView 接続時の `media_type=""` 扱い（ADR-0008 を継続）。
- `image/*` の受入範囲、特に SVG の表示経路
  （ADR-0008 の未決定を継続）。
- EPUB ファイル分割と Document 側の重複型解消（R2-epub#2 / #4）。
