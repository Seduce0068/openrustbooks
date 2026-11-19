---
type: Decision
title: "ADR-0203: CoverImage DTO の縮小"
description: "Web 側の CoverImage を受信に必要な 2 フィールドへ縮小し、allow(dead_code) を削除する決定を記録する。"
tags:
  - ADR
  - web
  - data
generated:
  by: dsh
  at: "2026-11-19T09:34:00+09:00"
status: accepted
---

## 背景

`web/src/lib.rs` の `CoverImage` は `id` / `media_type` を保持していたが、
表紙 Blob URL 生成は `href` / `bytes` のみを使うため
`#[allow(dead_code)]` が付いていた（R6 TD-L04）。
デシリアライズしたメタデータを捨てるより、必要なフィールドだけを受け取るべき。

## 決定

- `CoverImage` を `href: String` / `bytes: Vec<u8>` のみへ縮小する。
- `#[allow(dead_code)]` を削除する。
- core 側の `WasmEpubCoverImage`（`id` / `href` / `media_type` / `bytes`）は変更しない。
  serde の既定動作で未知フィールドは無視される。
- `object_url_for(&cover.bytes, &cover.href)` → `set_cover_url` の経路は維持する。
- MIME 推定は従来どおり `href` の拡張子から行い、
  `media_type` は元々利用していなかった。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| Web DTO を縮小（採用） | core / native 契約を壊さず警告も消える |
| core DTO も縮小 | ペイロードは減るが native API に波及する |
| allow を維持 | 不要フィールドと警告が残る |

## 影響

- `cover_image()` の JSON に `id` / `media_type` が含まれても
  serde が無視するため表示は変わらない。
- `from_value::<Option<CoverImage>>(...).ok()` の
  fail-open（失敗時に表紙なし）は既存挙動のまま。
- E2E 251 に表紙専用ケースは無く、
  E2E 単体では表紙回帰を検出できない（未決定）。
- 新規依存なし、unsafe なし。

## 検証（Runtime Proof）

- `cargo fmt --all -- --check`、
  `cargo check -p openrustbooks-web --target wasm32-unknown-unknown --locked`、
  wasm ターゲット clippy: 成功（dead_code warning なし）。
- `./scripts/verify.sh`: 全ゲート成功、E2E **251 項目すべて成功**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- core の `WasmEpubCoverImage` も `id` / `media_type` を削除し、
  wasm ↔ web のペイロードを縮小するか。
- `EpubCoverImage` の native 公開 API まで縮小するか。
- `.ok()` の fail-open を観測可能にするか。
- `#[serde(deny_unknown_fields)]` で契約を明示するか。
- E2E に表紙 Blob URL の直接チェックを追加するか。
