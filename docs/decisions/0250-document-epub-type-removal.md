---
type: Decision
title: "ADR-0250: Document 側 EPUB 型の削除"
description: "DocumentContent::Epub と重複する 3 構造体を削除し、EPUB は EpubBook 経路へ一本化する決定を記録する。"
tags:
  - ADR
  - core
  - api
generated:
  by: dsh
  at: "2026-11-19T22:49:00+09:00"
status: accepted
---

## 背景

`core/src/document.rs` の `DocumentContent::Epub` と
`EpubSpineItem` / `EpubManifestItem` / `EpubTocItem` は
リポジトリ内参照がゼロだった（R6 R2-epub#4）。
EPUB 閲覧は `EpubBook` / `WasmEpubBook` の別経路で提供され、
`DocumentContent` 自体は `lib.rs` で re-export されていない。
ただし `pub mod document` 経由で公開到達可能で、
serde variant としては破壊的変更になる。

## 決定

- `DocumentContent::Epub { spine, manifest, toc }` variant を削除する。
- `EpubSpineItem` / `EpubManifestItem` / `EpubTocItem` を削除する。
- `DocumentContent` の他 variant（`Text` / `Markdown` / `Pdf` / `Zip` /
  `Binary`）、impl、`DocumentFormat::Epub`、`epub.rs`、desktop / web / wasm は
  変更しない。
- `epub.rs` 型への一本化は行わない（変換対象ゼロのため別案件）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| dead な型を削除（採用） | 公開面の重複を減らし、EPUB 経路を明確化できる |
| 残置 | dead な公開 API / serde variant が残る |
| `epub.rs` へ一本化 | 型構造が非互換で変換対象もなくコストが見合わない |

## 影響

- Rust API 破壊: 外部クレートが 3 構造体を名指しするか、
  `DocumentContent` を網羅 match するとビルド不能。
- serde 破壊: `{"type":"epub", ...}` のデシリアライズは失敗する。
- リポジトリ内には利用例・永続化が無く、E2E 255 は維持。
- version は 0.1.0 を維持（0.x の破壊変更として扱う）。

## 検証（Runtime Proof）

- 残存参照: 全 `*.rs` で 4 名称 0 件。
- `cargo test --all-targets --locked`: core **279** / desktop 79 / web 2。
- `wasm-pack test --node --locked`: 15 passed。
- `cargo clippy --all-targets --locked -- -D warnings`、
  wasm32 core / web check、`./scripts/verify.sh`: すべて成功。
- E2E **255 項目すべて成功**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- 外部 Rust / serde 利用者の有無と deprecation shim の要否
  （ADR-0243 / 0244 を継続）。
- `epub.rs` 型への一本化（`serde` derive / `properties` / 入れ子 TOC の整理）。
- version 0.1.0 維持か 0.2.0 への昇格か
  （ADR-0239 の単一情報源、ADR-0076 の形式 version との整合）。
