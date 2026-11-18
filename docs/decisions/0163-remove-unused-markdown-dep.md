---
type: Decision
title: "ADR-0163: 未使用 Markdown 依存の削除"
description: "直接未使用となった pulldown-cmark をマニフェストから削除する決定を記録する。"
tags:
  - ADR
  - deps
generated:
  by: dsh
  at: "2026-11-18T22:40:00+09:00"
status: accepted
---

## 背景

ADR-0162 で comrak 0.59（featureless）を Markdown Preview に採用した結果、
`pulldown-cmark 0.13`（`simd` feature 付き）はコードから未使用になった
（ADR-0162 の未決定）。Rust ソース / スクリプトの grep は 0 件。

## 決定

- `Cargo.toml:26`（workspace.dependencies）と `core/Cargo.toml:21` から
  `pulldown-cmark` を削除する。
- `comrak` / `ammonia` は維持し、ADR-0162 の二段サニタイズ契約を変更しない。
- `Cargo.lock` は cargo で更新（計 11 行減: `getopts` ブロック、
  core の dependencies 1 行、pulldown-cmark の getopts 参照）。
- **脆弱性対策ではない**: RUSTSEC-2026-0433 は pulldown-cmark 0.13.6 で
  修正済み（ADR-0086）。削除は未使用依存の整理であり、
  rumdl 経由の推移依存は残る（Cargo.lock に `pulldown-cmark` /
  `pulldown-cmark-escape` が残存）。

## 影響

- `cargo audit --no-fetch`: 脆弱性 0、警告 baseline 10 のまま（差分なし）。
- 直接依存は消えるが推移依存として残るため、
  「依存ツリーから完全に消えた」と表現しない。
- README.md:74 / docs/architecture.md:178 の依存一覧は、comrak / ammonia / rumdl の
  実装事実へ更新済み（未決定だったドキュメント整合は解消）。
- workspace の rumdl `default-features=true`（Cargo.toml:27）は
  ADR-0155 の未決定として維持。

## 検証（Runtime Proof）

- `cargo check --all-targets --locked` /
  `cargo test --all-targets --locked` /
  `cargo clippy --all-targets --locked -- -D warnings`: 成功。
- wasm32 core / web check: 成功。
- `cargo audit --no-fetch`: 脆弱性 0、警告 10（変更前後同数）。
- `./scripts/verify.sh` rc=0、E2E **238 項目すべて成功**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- workspace.dependencies の rumdl エントリ整理（ADR-0155 と連動）。

## 解消済み

- README / architecture の依存一覧更新（pulldown-cmark の直接依存記載を削除し、
  comrak / ammonia / rumdl の実装事実へ更新）。推移依存として残る旨は本文に記載済み。
