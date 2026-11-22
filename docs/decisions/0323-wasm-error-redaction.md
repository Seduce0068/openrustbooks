---
type: Decision
title: "ADR-0323: Wasm JS 境界のエラー redaction"
description: "CoreError を共通 fixed_message(code) 経由で JS へ返し、生文字列の漏洩を防ぐ決定。"
tags:
  - ADR
  - wasm
  - security
generated:
  by: dsh
  at: "2026-11-22T14:53:00+09:00"
status: accepted
---

## 背景

`core/src/wasm.rs::js_error()` と wasm 子モジュールの一部が
`CoreError::to_string()` を JS へ返していた。
href / パス / 内部型名を含む生文字列が公開 Wasm API から読めた。

## 決定

- `core::error::fixed_message(code)` を単一情報源として追加する。
- desktop の `ipc_message()` と Wasm の `js_error()` の両方が同じ mapper を参照する。
- `CONFIG_LOAD_FAILED_MESSAGE` を core 側 pub const 化し、
  desktop 定数との一致を test で固定する。
- wasm epub / lint の CoreError を `js_error()` 経由に統一する。
- serde_wasm_bindgen エラーなど非 CoreError も固定文言へ変換する。
- lint 設定エラーは `Invalid lint config` 固定文言（ADR-0160 の接頭辞を維持）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 共通 mapper（採用） | desktop / wasm の redaction 方針を統一できる |
| 境界ごとに個別文言 | drift と漏洩再発のリスクがある |
| 生 Display を返す | secret / path 露出のため却下 |

## 検証（Runtime Proof）

- core 357 / desktop 166 / web 19 = 542 tests、すべて成功。
- href redaction test: 不正 href が固定文言へ変換され、
  マーカー / スキーム / 内部型名が出ないことを検証。
- lint test: `Invalid lint config` 完全一致と型名非含有を検証。
- `cargo fmt --all -- --check`: rc=0。
- `cargo check --all-targets --locked`: rc=0。
- `cargo clippy --all-targets --locked -- -D warnings`: rc=0。
- wasm core / web check + wasm core `--tests` check: rc=0。
- `./scripts/verify.sh`: 全ゲート成功、E2E **256 passed / 0 failed**、WDIO gate rc=0。

## 未決定

- wasm-bindgen-test-runner 導入後の wasm test 実実行。
- 既定文言 "Internal configuration error" を wasm 側で中立文言へ分離するか。
