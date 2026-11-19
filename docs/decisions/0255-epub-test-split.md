---
type: Decision
title: "ADR-0255: EPUB テストの分離"
description: "EPUB の 92 tests を epub/tests.rs へ切り出し、本編のモジュール分割は未決定として残す決定を記録する。"
tags:
  - ADR
  - epub
  - testing
generated:
  by: dsh
  at: "2026-11-20T01:01:00+09:00"
status: accepted
---

## 背景

`core/src/epub.rs` は実測 3780 行（本編 1–1880 / tests 1881–3780）で、
todo の「1316 行ファイルの分割」という記述は古い（R6 R2-epub#2）。
test 92 件のうち private item 直接参照は
`MAX_TOC_ENTRIES` と `is_image_media_type` の 2 つだけだった。

## 決定

- `#[cfg(test)] mod tests { ... }` の内側本文 1897 行を
  `core/src/epub/tests.rs` へバイト単位で移動する。
- `core/src/epub.rs` の本編 1–1880 は変更しない。
- 参照は次とする（`#[rustfmt::skip]` は本文の再インデント防止）:

```rust
#[cfg(test)]
#[rustfmt::skip]
#[path = "epub/tests.rs"]
mod tests;
```

- `#[path = "tests.rs"]` は non-mod-rs ファイル `epub.rs` から
  `core/src/tests.rs` を解決するため、
  配置先を指す `epub/tests.rs` を採用する（Rust Reference の path 規則）。
- `use super::*;` と、親に残す
  `MAX_TOC_ENTRIES` / `is_image_media_type` により可視性は不変。
- 本編の 3/4 モジュール分割（opf / navigation / text / xmlutil）は
  今回行わず、未決定として残す。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| tests のみ切り出し（採用） | 挙動・公開 API・test 内容を変えず行数を半減できる |
| 本編も同時分割 | 可視性 `pub(super)` の設計判断が必要で差分が大きい |
| 設計受容（分割しない） | todo の実態が 3780 行のままになり保守性が落ちる |

## 影響

- `core/src/epub.rs`: 3780 → **1884** 行。
- `core/src/epub/tests.rs`: **1897** 行、`#[test]` 92 件。
- 本編 SHA-256 と tests 本文 SHA-256 は分割前と一致（バイト単位で同一）。
- core test は 284 件（旧 GOAL の 279 は古い基準）、E2E 255 維持。
- 公開 6 型と fail-closed 契約は不変。

## 検証（Runtime Proof）

- 本編 1–1880: SHA-256 `d6316bb8…b832`（分割前と一致）。
- tests 本文: SHA-256 `8cb975e0…508c`（分割前と一致）。
- `cargo test -p openrustbooks-core -- --list`: `epub::tests::` 92 件。
- `cargo test --all-targets --locked`: core **284** / desktop 79 / web 2。
- `wasm-pack test --node --locked`: 15 passed。
- `cargo clippy --all-targets --locked -- -D warnings`、
  wasm32 core / web check、`./scripts/verify.sh`: すべて成功。
- E2E **255 項目すべて成功**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- 本編 1880 行の 3/4 モジュール分割
  （opf / navigation / text / xmlutil の境界と `pub(super)` 化）。
- `epub.rs` + `epub/` 構成から `epub/mod.rs` への改名
  （その場合 `#[path = "tests.rs"]` へ簡素化可能）。
- 共有 helper モジュール名（`xmlutil` / `path` / `util`）。
- tests の integration test 化
  （その場合のみ `is_image_media_type` / `MAX_TOC_ENTRIES` の
  `pub(crate)` 化または公開 API 追加が必要）。
- todo の行数・件数表記の将来的な更新（本件で R2-epub#2 を更新）。
