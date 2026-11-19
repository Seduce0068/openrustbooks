---
type: Decision
title: "ADR-0244: WasmDocument の削除"
description: "未使用の WasmDocument export を削除し、Wasm API の第二段を完了する決定を記録する。"
tags:
  - ADR
  - wasm
  - api
generated:
  by: dsh
  at: "2026-11-19T21:10:00+09:00"
status: accepted
---

## 背景

ADR-0243 の第一段で `greet` / `detect_format` を削除した後も、
`core/src/wasm.rs` の `WasmDocument` は JavaScript へ export されていた。
`web/src` / wasm unit test / ADR 決定から参照が無く、
`#[wasm_bindgen]` は Rust の `pub` と独立して JS export を生成するため、
属性ごと削除する必要があった（R6 R2-crates#3 第二段）。

## 決定

- `WasmDocument` の doc コメント、`#[cfg(target_arch = "wasm32")]` /
  `#[wasm_bindgen]` / `#[derive(Serialize, Deserialize)]` 属性、
  struct / impl を属性ごと削除する。
- `WasmEpubBook` / `WasmArchive` / `WasmMarkdownLinter` /
  AI cluster / `version()` は本次では変更しない
  （AI cluster の削除は ADR-0256 で完了）。
- `serde` import は他 API が使用するため維持する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 属性ごと削除（採用） | JS export が実際に減る |
| `pub` だけ外す | wasm-bindgen が独立に export するため却下 |
| AI cluster も同時削除 | 本次では別案件、ADR-0256 で完了（ADR-0156 の AI 部分を supersede） |

## 影響

- JS export 面がさらに 1 型減る（`WasmDocument`）。
- `core/src` / `web/src` の参照は 0 件で、wasm test 15 件は維持。
- `web/dist` の gitignore 済み古い成果物には旧 export が残るが、
  `verify.sh` は毎回再ビルドするため影響しない。
- 外部 JS 利用者がいた場合は破壊的変更になり得る
  （リポジトリ内に利用証拠なし）。

## 検証（Runtime Proof）

- `cargo test --all-targets --locked`: core 274 / desktop 79 / web 2。
- `wasm-pack test --node --locked`: **15 passed**。
- `cargo clippy --all-targets --locked -- -D warnings`、
  wasm32 core / web check、`./scripts/verify.sh`: すべて成功。
- E2E **254 項目すべて成功**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- 外部 JS 利用者の有無と、deprecation shim の要否（ADR-0243 を継続）。
- ~~AI cluster の削除または JS export のみの非公開化~~
  ADR-0256 で削除を決定・完了（ADR-0156 の AI 部分を supersede）。
- バンドルサイズの before / after 測定（ADR-0160 / 0242 / 0243 を継続）。
