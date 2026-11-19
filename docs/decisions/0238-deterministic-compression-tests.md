---
type: Decision
title: "ADR-0238: 圧縮率依存テストの決定的化"
description: "ZIP エントリの方式とメタデータを直接検証し、deflate の圧縮率に依存するテストを排除する決定を記録する。"
tags:
  - ADR
  - testing
  - archive
generated:
  by: dsh
  at: "2026-11-19T19:35:00+09:00"
status: accepted
---

## 背景

`core/src/archive.rs` と `core/src/wasm.rs` の ZIP 一覧 test は
`compressed_size < size` という deflate の圧縮率に依存していた（R6 R4-unit#4）。
deflate の出力サイズは仕様で保証されず、
backend / zip バージョン変更で壊れ得る。
ADR-0004 には deflate backend 指定漏れでビルドが失敗した前例もある。

## 決定

- 比率 assertion を削除し、次を直接検証する:
  - `ZipFile::compression() == CompressionMethod::Deflated`
  - `ArchiveEntry.compressed_size == ZipFile::compressed_size()`
- 対象は `archive.rs` の native test と `wasm.rs` の wasm-bindgen-test の 2 箇所。
- `wasm.rs` の test module に `Cursor` / `ZipArchive` を追加する。
- `WasmArchive::new()` が Vec を消費するため、
  テスト内で元バイト列を 1 回 `clone()` して直接検証に使う。
- 既存の内容一致 test は維持し、`read_entry` の動作はそちらで担保する。
- 本番コード・公開 API・依存は変更しない。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| `ZipFile` の公開 API で直接検証（採用） | API 変更なしで決定的に方式を確認できる |
| `ArchiveEntry` に `compression_method` を追加 | pub 構造体 / wasm DTO への破壊的変更で却下 |
| 反復データで圧縮率を実質確実にする | 依然として契約ではないため却下 |
| 固定バイト数で検証 | 実装詳細を固定するだけのため却下 |

## 影響

- 圧縮 backend / zip バージョンが変わっても、
  方式が Deflated でありメタデータが一致すれば test は通る。
- エントリ一覧順・内容一致・limit の既存 test は不変。
- E2E 254 維持。

## 検証（Runtime Proof）

- `cargo test -p openrustbooks-core`: 270 passed / 0 failed。
- `cargo test --all-targets --locked`: core 270 / desktop 79 / web 2。
- `wasm-pack test --node --locked`: 14 passed / 0 failed。
- `cargo clippy --all-targets --locked -- -D warnings`、
  wasm32 core / web check、`./scripts/verify.sh`: すべて成功。
- E2E **254 項目すべて成功**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- compression method を `ArchiveEntry` / wasm DTO へ公開するか
  （利用者が方式を知る必要が生じた場合のみ）。
- deflate backend を変更した際の回帰確認手順。
- wasm DTO を JS API として維持する前提の明文化。
- 「決定的 test」の基準を他 unit test へ横展開するか。
