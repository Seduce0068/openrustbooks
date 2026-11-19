---
type: Decision
title: "ADR-0259: Wasm バインディングの責務別分割"
description: "Wasm バインディング本編を epub / archive / lint モジュールへ分割し、公開パスと fail-closed 契約を維持する決定を記録する。"
tags:
  - ADR
  - wasm
  - architecture
generated:
  by: dsh
  at: "2026-11-20T02:22:00+09:00"
status: accepted
---

## 背景

ADR-0257 で tests 379 行を `wasm/tests.rs` へ切り出し、
`core/src/wasm.rs` は 537 行（本編）になった。
EPUB / ZIP / lint の 3 クラスタが 1 ファイルに混在しており、
公開パス（`pub use wasm::*`）と native 向け cfg を維持しつつ
子モジュールへ分離する（R6 R2-crates#4）。

## 決定

- `core/src/wasm.rs` には共有基盤のみを残す:
  - module doc、`console_log!` extern / macro、`init_core()`、
    `init()`（wasm / native）、`image_dimensions_wasm`、
    `render_markdown_wasm`、`sanitize_svg_wasm`、`js_error()`、
    native `utils`、tests 参照、子モジュール宣言と再エクスポート。
- 子モジュールへ分割:
  - `wasm/epub.rs`（185 行）: `WasmEpubBook` と serializer / DTO。
  - `wasm/archive.rs`（97 行）: `WasmArchive` と 4 つの固定エラー / DTO。
  - `wasm/lint.rs`（158 行）: `WasmMarkdownLinter` と lint DTO。
- 公開パスは親の `pub use` で維持する:
  `WasmArchive` / `WasmEpubBook` / `WasmLintFixResult` /
  `WasmLintIssue` / `WasmMarkdownLinter`。
- `WasmArchiveEntry` は元どおり private とし、
  test のみ `#[cfg(all(test, target_arch = "wasm32"))] use` で親へ取り込む。
- 4 つの `ERR_*` は `pub(super)` で親へ見せ、
  test 用に `#[cfg(all(test, target_arch = "wasm32"))] use` で取り込む。
- `console_log!` は `$crate::wasm::log(...)` の絶対パスを用い、
  子モジュールからのテキストスコープ依存を排除した。
- `#[cfg(target_arch = "wasm32")]` は子モジュール宣言側に付け、
  native では子をコンパイルしない。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 責務別 3 モジュール分割（採用） | 公開パスを再エクスポートで維持でき、native cfg も親集約で安全 |
| 全コードを親に残す | 537 行の混在が続き ADR-0257 の方針と不整合 |
| support モジュール新設 | 共有 item は 4 つのみで、今回は親残留が単純 |

## 影響

- `core/src/wasm.rs`: 537 → **134** 行。
- 子モジュール合計 440 行、tests 376 行は無変更（SHA-256 一致）。
- 公開 Rust API / JS export 面 / fail-closed 固定文言 / redaction 境界は不変。
- `unsafe` は追加していない。

## 検証（Runtime Proof）

- `cargo test --all-targets --locked`: core **284** / desktop 79 / web 2。
- `wasm-pack test --node --locked`: **14 passed**。
- wasm32 `cargo check --all-targets`（tests 含む）、
  core / web check、`cargo clippy --all-targets --locked -- -D warnings`、
  `./scripts/verify.sh`: すべて成功。
- E2E **255 項目すべて成功**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- support モジュール新設と `js_error` / `console_log!` の移行
  （ADR-0257 の未決定を今回は親集約で解消）。
- integration test 化（private item の可視性拡張または固定文言リテラル化が必要）。
- 外部 JS 利用者の有無と破壊確認。
- バンドルサイズの before / after 実測（ADR-0256 を継続）。
- `ERR_ENTRIES` のテスト追加（ADR-0223 の 4 文言のうち 1 つは未検証）。
- `wasm.rs` → `wasm/mod.rs` への改名（ADR-0257 を継続）。
