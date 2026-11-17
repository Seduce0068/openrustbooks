---
type: Decision
title: "ADR-0089: core 初期化処理の一本化"
description: "Rust / JS 双方の init() が同じ初期化処理を呼ぶようにする決定を記録する。"
tags:
  - ADR
  - wasm
  - refactor
generated:
  by: dsh
  at: "2026-11-17T22:50:00+09:00"
status: accepted
---

## 背景

`core/src/lib.rs` と `core/src/wasm.rs` に別々の `init()` が存在していた。
wasm32 では `pub use wasm::*` により名前が衝突し得るが、Rust 側
（`web/src/lib.rs`）は lib.rs の版を呼ぶため、JS 向け wasm-bindgen 版と
初期化処理が二重管理になっていた（todo.md D03）。

## 決定

- 初期化処理（panic hook + tracing_wasm）を `wasm::init_core()` に集約する。
- `core::init()`（Rust 側）と wasm-bindgen の `init()`（JS 側）は
  どちらも `init_core()` を呼ぶ。
- `console_log!` は wasm-bindgen 版に残す（JS 側の初期化通知）。
- native の `tracing_subscriber` 初期化は現状維持。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 共通関数へ集約（採用） | 二重管理を解消し、挙動差を防ぐ |
| wasm-bindgen 版を削除 | JS 側の初期化 API が失われる |
| re-export を削除 | 型の公開に影響する |

## 影響

- Rust / JS のどちらから初期化しても同じ設定が適用される。
- 既存の API（`core::init()` と wasm-bindgen `init()`）は維持される。
- 追加依存は無い。

## 検証

- `./scripts/verify.sh` を実行し、6 ゲート・trunk build・WASM パス検査・
  E2E 223 項目がすべて成功した。
- `cargo test --all-targets --locked` は 126 件成功。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- wasm-bindgen 版 `init()` の JS 側からの実際の呼び出し有無（現状は Rust 側の
  `main()` が `core::init()` を呼ぶ）。
- `IntersectionObserver` による画像の遅延読み込み（ADR-0034 から継続）。
- EPUB 内 CSS、縦書き・ページ送り（ADR-0012 から継続）。
EOF
