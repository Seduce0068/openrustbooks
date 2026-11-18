---
type: Decision
title: "ADR-0168: workspace の rumdl 宣言削除"
description: "未使用の workspace.dependencies.rumdl を削除し、default features 復活の罠を排除する決定を記録する。"
tags:
  - ADR
  - deps
generated:
  by: dsh
  at: "2026-11-18T23:58:00+09:00"
status: accepted
---

## 背景

`Cargo.toml` の `[workspace.dependencies]` に
`rumdl = { version = "0.3", default-features = true }` が残っていた。
しかしどの member も `rumdl = { workspace = true }` を使わず、
core は native / wasm の両方で `default-features = false` を直接指定している
（ADR-0155）。将来誤って workspace 版を参照すると default features
（tower-lsp / notify / jemalloc 等）が復活する罠があった。

## 決定

- workspace.dependencies から `rumdl` 行を削除する（1 行のみ）。
- `core/Cargo.toml` の直接指定（native / wasm とも `default-features = false`）は維持。
- `Cargo.lock` は差分なし（lock は workspace 宣言と feature を記録しない）。
- rumdl 依存自体は core の直接依存として残る
  （README / architecture の rumdl 記載は訂正不要）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 削除（採用） | member に参照が無く、誤参照をコンパイルエラーで防げる |
| `default-features = false` に揃える | 未使用エントリを残す意味が薄い |

## 影響

- 解消する未決定:
  - ADR-0155「workspace エントリを揃えるか削除するか」→ **削除**
  - ADR-0163「workspace.dependencies の rumdl エントリ整理」
  - ADR-0160「workspace rumdl `default-features=true` 残存」
- 解消しない未決定: `parallel` 再導入、上流 optional 化、CI env_vars、
  ADR-0003 本文修正、`converged` 公開、i18n、バンドルサイズ計測等。
- featureless は Cargo.lock で担保されない（core/Cargo.toml の直接指定が前提）。

## 検証（Runtime Proof）

- `cargo check --all-targets --locked` /
  `cargo test --all-targets --locked` /
  `cargo clippy --all-targets --locked -- -D warnings` /
  wasm32 core / web check / `cargo audit --no-fetch`: すべて成功。
- `Cargo.lock` の SHA-256 は変更前後で同一
  （`3b99516e...6537d`）。
- `./scripts/verify.sh` rc=0、E2E **242 項目すべて成功**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- CI で `core/Cargo.toml` の rumdl 2 行が featureless であることを
  静的検査するゲートを追加するか（今回のスコープ外、推測）。
