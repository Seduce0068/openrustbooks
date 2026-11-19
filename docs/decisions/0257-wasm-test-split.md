---
type: Decision
title: "ADR-0257: WASM binding tests の分離"
description: "wasm-bindgen-test 14 件を wasm/tests.rs へ切り出し、本編のモジュール分割は未決定として残す決定を記録する。"
tags:
  - ADR
  - wasm
  - testing
generated:
  by: dsh
  at: "2026-11-20T01:59:00+09:00"
status: accepted
---

## 背景

`core/src/wasm.rs` は実測 912 行（本編 533 / tests 379）で、
tests 14 件は `#[cfg(all(test, target_arch = "wasm32"))]` の下にあった
（R6 R2-crates#4）。
ADR-0255 の EPUB tests 分割と同じ手順で、
公開 API・可視性・test 本文を変えずにファイルだけ分割する。

## 決定

- `mod tests { ... }` の内側本文 376 行を
  `core/src/wasm/tests.rs` へバイト単位で移動する。
- `core/src/wasm.rs` の本編 1–533 は変更しない。
- 参照は次とする（`#[rustfmt::skip]` は本文の再インデント防止）:

```rust
#[cfg(all(test, target_arch = "wasm32"))]
#[rustfmt::skip]
#[path = "wasm/tests.rs"]
mod tests;
```

- `#[path = "tests.rs"]` は non-mod-rs ファイル `wasm.rs` から
  `core/src/tests.rs` を解決するため、`wasm/tests.rs` を採用する。
- `use super::{ ... }` は維持し、private item の可視性は変更しない。
- 本編の 4 モジュール分割（epub / archive / lint / support）は
  今回行わず、ADR の未決定として残す。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| tests のみ切り出し（採用） | 挙動・公開 API・test 内容を変えず行数を減らせる |
| 本編も同時分割 | 再エクスポート / native cfg / マクロスコープ / `pub(super)` の調整が別設計 |
| 設計受容（分割しない） | 912 行のままになり ADR-0255 の方針と不整合 |

## 影響

- `core/src/wasm.rs`: 912 → **537** 行。
- `core/src/wasm/tests.rs`: **376** 行、`#[wasm_bindgen_test]` 14 件。
- 本編 SHA-256 と tests 本文 SHA-256 は分割前と一致。
- wasm-bindgen の export 面、`pub use wasm::*`、web 側の参照は不変。
- E2E 255 維持。

## 検証（Runtime Proof）

- 本編 SHA-256 `8677fea7…d0435`、tests 本文 SHA-256
  `c7a0c150…3f76`（いずれも分割前と一致）。
- `wasm-pack test --node --locked`: **14 passed**。
- `cargo test --all-targets --locked`: core **284** / desktop 79 / web 2。
- `cargo clippy --all-targets --locked -- -D warnings`、
  wasm32 core / web check、`./scripts/verify.sh`: すべて成功。
- E2E **255 項目すべて成功**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- 本編 533 行の 4 モジュール分割
  （epub / archive / lint / support と `pub(super)` 化）。
- `wasm.rs` → `wasm/mod.rs` への改名
  （その場合 `#[path = "tests.rs"]` へ簡素化可能）。
- `js_error` / `console_log!` を親へ残すか support モジュールへ移すか。
- tests の integration test 化
  （その場合のみ private item の可視性拡張が必要）。
- 外部 JS 利用者の有無とバンドルサイズ実測（ADR-0256 を継続）。
