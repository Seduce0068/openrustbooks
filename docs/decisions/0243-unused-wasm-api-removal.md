---
type: Decision
title: "ADR-0243: 未使用 Wasm API の削除"
description: "wasm_bindgen の JS export を属性ごと削除し、未使用の greet / detect_format を公開面から外す決定を記録する。"
tags:
  - ADR
  - wasm
  - api
generated:
  by: dsh
  at: "2026-11-19T20:59:00+09:00"
status: accepted
---

## 背景

`core/src/wasm.rs` の `utils::greet()` / `utils::detect_format()` は
JavaScript へ export されていたが、
`web/src` / wasm unit test / ADR 決定のいずれからも参照されていなかった
（R6 R2-crates#3 第一段）。
`wasm-bindgen` は Rust の `pub` と独立して JS export を生成するため、
`pub` を外すだけでは攻撃面は減らない。

## 決定

- `greet()` / `detect_format()` を `#[wasm_bindgen]` 属性ごと削除する。
- `utils::version()` は wasm / native 双方で維持する。
- `WasmDocument` と AI cluster
  （`WasmAiMarkdownService` / `WasmPipelineResult` / `WasmDiffResult`）は
  本次では変更しない（AI cluster は ADR-0256 で削除完了）。
- ADR-0223:20 / 調査記録の `detect_format` 言及は履歴として保持する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 属性ごと削除（採用） | JS export が実際に減る |
| `pub` だけ外す | wasm-bindgen は独立に export を生成するため却下 |
| AI cluster も削除 | 本次では別案件、ADR-0256 で完了（ADR-0156 の AI 部分を supersede） |

## 影響

- wasm `utils` の export は 3 → 1（`version()` のみ）。
- リポジトリ内の web / wasm test / E2E には参照が無く、
  E2E 254 を維持する。
- 古い `web/dist` の生成 JS は旧 export を持つが
  `.gitignore` 対象の stale 成果物で、`verify.sh` は毎回再ビルドする。
- 外部 JS 利用者がいた場合は破壊的変更になり得る
  （リポジトリ内に利用証拠なし）。

## 検証（Runtime Proof）

- `grep`: `greet` / `detect_format` / `std::path::Path` は
  `core/src/wasm.rs` に 0 件。
- `cargo test --all-targets --locked`: core 274 / desktop 79 / web 2。
- `wasm-pack test --node --locked`: **15 passed**。
- `cargo clippy --all-targets --locked -- -D warnings`、
  wasm32 core / web check、`./scripts/verify.sh`: すべて成功。
- E2E **254 項目すべて成功**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- 外部 JS 利用者の有無と、deprecation shim の要否。
- ~~AI cluster の削除または JS export だけの非公開化~~
  ADR-0256 で削除を決定・完了（ADR-0156 の AI 部分を supersede）。
- `WasmDocument` の削除（第二段の候補）。
- バンドルサイズの before / after 測定（ADR-0160 / 0242 を継続）。
