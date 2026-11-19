---
type: Decision
title: "ADR-0247: EPUB 所有 API への limits 指定"
description: "EpubBook の所有権移譲 API に ArchiveLimits を指定可能にし、借用 API と同じ検証契約を共有する決定を記録する。"
tags:
  - ADR
  - epub
  - api
generated:
  by: dsh
  at: "2026-11-19T22:00:00+09:00"
status: accepted
---

## 背景

`ArchiveReader` は limits 付きの所有 API `from_owned(bytes, limits)` を持つが、
`EpubBook::open_owned(bytes)` は既定 limits 固定だった（R6 R2-epub#1）。
将来 limits を要する呼び出しが借用 API `with_limits(&bytes, ...)` へ逃げると、
最大 256 MiB のバイト列を二重コピーする。
ADR-0051 は所有 API 追加と `with_limits` からの委譲を決定済みで、
ADR-0240 は全生成経路が `from_owned` に収束することを記録している。

## 決定

- `EpubBook::open_owned_with_limits(bytes: Vec<u8>, limits: ArchiveLimits)`
  を追加し、`Self::from_archive(ArchiveReader::from_owned(bytes, limits)?)`
  へ委譲する。
- 既存 `open_owned()` は
  `open_owned_with_limits(bytes, ArchiveLimits::default())` へ委譲する。
- 検証とエラー分類は借用経路 `with_limits()` と同じ `from_archive()` を共有する:
  - `UnsupportedFormat`: mimetype 不一致
  - `Archive`: コンテナ / OPF の読み出し・解析失敗
  - `Validation`: パス逸脱・limits 超過
- `wasm.rs` / web / E2E は変更しない（ADR-0190 の未決定を維持）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 所有 API に limits を追加（採用） | 二重コピーせず制限を指定できる |
| 借用 API を使う | 256 MiB のバイト列を二重コピーする |
| 新 API なし | 将来の limits 要件に応えられない |

## 影響

- `CoreError::Validation` の文言は下位 `archive.rs` の既存文言を引き継ぐ。
  生エラーやアーカイブ内パスを status へ出す際は
  ADR-0212 の固定文言変換に依存する（core 単体では redaction を保証しない）。
- 既定 limits（4096 entries / 64 MiB / 256 MiB）は
  ADR-0190 の暫定値で、実測による根拠は未決定。
- E2E 255 維持。

## 検証（Runtime Proof）

- unit test 2 件追加:
  - `max_entries = 1` の所有経路が `CoreError::Validation` を返す。
  - 既定 limits で旧 `open_owned` とメタデータ／章／本文が一致。
- `cargo test --all-targets --locked`: core **276** / desktop 79 / web 2。
- `wasm-pack test --node --locked`: 15 passed。
- `cargo clippy --all-targets --locked -- -D warnings`、
  wasm32 core / web check、`./scripts/verify.sh`: すべて成功。
- E2E **255 項目すべて成功**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- wasm 境界で limits を公開するか
  （web UI の 256 MiB 事前チェックとの関係、ADR-0190 を継続）。
- 既定 limits の数値根拠（未計測）。
- manifest item の `id` 欠落時のスキップをエラー化するか
  （ADR-0240 / R2-epub#3）。
- `epub.rs` のテスト／本編分割（R2-epub#2）。
- `document.rs` の重複型整理（R2-epub#4、公開 API 互換の判断が必要）。
