---
type: Decision
title: "ADR-0260: EPUB 本編の責務別分割"
description: "EPUB 本編を opf / navigation / text / xmlutil モジュールへ分割し、公開 6 型と fail-closed 契約を維持する決定を記録する。"
tags:
  - ADR
  - epub
  - architecture
generated:
  by: dsh
  at: "2026-11-20T02:37:00+09:00"
status: accepted
---

## 背景

ADR-0255 で tests 1897 行を `epub/tests.rs` へ切り出し、
`core/src/epub.rs` は本編 1880 行になった（R6 R2-epub#2）。
container / OPF、nav / NCX、XHTML テキスト抽出、
共通 XML / パス helper が 1 ファイルに混在していた。

## 決定

- `core/src/epub.rs` には module doc、定数（全て）、公開 6 型、
  `impl EpubBook`（全公開メソッド）、`is_image_media_type`、
  子モジュール宣言と private 再取り込み、tests 参照のみを残す。
- 子モジュールへ分割:
  - `epub/opf.rs`（509 行）: container / OPF 解析と private データ構造。
  - `epub/navigation.rs`（473 行）: nav / NCX 解析。
  - `epub/text.rs`（277 行）: XHTML テキスト / タイトル抽出。
  - `epub/xmlutil.rs`（120 行）: 属性・パス・正規化の共通 helper。
- 境界を跨ぐ 23 item は `pub(super)` とし、
  親は private `use self::...` で従来名を再取り込む。
- 公開 6 型（ChapterLink / EpubBook / EpubChapter / EpubCoverImage /
  EpubMetadata / EpubTocEntry）と `lib.rs:26` の re-export は不変。
- tests.rs は無変更（SHA-256 `8cb975e0…508c`）。
- エラー文言、fail-closed 挙動、上限値（`MAX_TOC_ENTRIES = 10_000`）、
  media-type 契約、serde 属性は 1 文字も変更しない。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 責務別 4 モジュール分割（採用） | 公開型を動かさず、依存を parent → children の一方向に整理できる |
| 1880 行のまま残す | ADR-0255 の方針と不整合で保守性が低い |
| integration test 化を同時実施 | private item の可視性拡張が必要で test 本文変更になる |

## 影響

- `core/src/epub.rs`: 1884 → **556** 行。
- 子モジュール合計 1379 行、tests 1897 行は無変更。
- 子モジュールに bare `pub` は無く、公開面は拡大していない。
- EPUB の挙動・公開 Rust API・wasm 境界は不変。

## 検証（Runtime Proof）

- `cargo test --all-targets --locked`: core **284** / desktop 79 / web 2。
- `wasm-pack test --node --locked`: **14 passed**。
- `cargo clippy --all-targets --locked -- -D warnings`、
  wasm32 core / web check、`./scripts/verify.sh`: すべて成功。
- E2E **255 項目すべて成功**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- support モジュール新設の是非（今回は `xmlutil` と親定数へ集約、
  ADR-0259 の前例に整合）。
- integration test 化（その場合のみ `is_image_media_type` /
  `MAX_TOC_ENTRIES` の可視性拡張が必要）。
- 外部利用者の有無と破壊確認。
- バンドルサイズ before / after 実測（ADR-0256 / 0259 を継続）。 比較可能 before は [ADR-0267](0267-bundle-size-baseline.md) に記録済み。
- `opf.rs` が 509 行と依然大きいことの再評価
  （1 ファイル何行までという基準は未定義）。
- `epub.rs` → `epub/mod.rs` への改名（ADR-0255 / 0259 を継続）。
