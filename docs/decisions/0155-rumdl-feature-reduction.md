---
type: Decision
title: "ADR-0155: rumdl の feature 削減"
description: "native 側 rumdl を featureless にし、LSP / ファイル監視 / 並列処理スタックを依存ツリーから除去する決定を記録する。"
tags:
  - ADR
  - deps
  - ci
generated:
  by: dsh
  at: "2026-11-18T20:20:00+09:00"
status: accepted
---

## 背景

ADR-0151 の探索で、native 側 rumdl が
`default-features = true`（`parallel` + `native`）で入っており、
tower-lsp / tokio / notify / rayon / blake3 等を引き込んでいることが判明した。
CI は tikv-jemalloc-sys のディスク問題を env_vars の debuginfo 破棄で回避していた。
core は `rumdl_lib::config::Config` と rules API のみを使用
（`core/src/ai_markdown/lint.rs:4-12`）。

## 決定

- native 側も `rumdl = { version = "0.3", default-features = false }` に変更
  （`core/Cargo.toml:66`）。wasm 側は既に featureless のため維持。
- `Cargo.lock` を再ロックし、28 パッケージを削減。
- `.github/workflows/ci.yml` の `env_vars` は **維持**。
  - tikv-jemallocator は rumdl 0.3.2 の non-optional 依存のため残存し、
    ディスク回避の前提が変わらないため。
- ルート `Cargo.toml:28` の workspace 依存 `rumdl`（default=true）は
  現状どの member も `workspace = true` を使っていないため解決には影響しないが、
  将来の罠として未決定に残す。

## 影響

- 削除された主要依存（28 個）:
  `tower-lsp`, `tower-lsp-macros`, `lsp-types`, `tower`, `notify`,
  `notify-types`, `inotify`, `inotify-sys`, `kqueue`, `kqueue-sys`,
  `blake3`, `constant_time_eq`, `postcard`, `cobs`, `embedded-io`,
  `env_logger`, `env_filter`, `colored`, `ec4rs`, `etcetera`,
  `arrayvec`, `auto_impl`, `dashmap`, `rayon`, `rayon-core`,
  `hashbrown`, `objc2-core-services`。
- **残存**: `serde_yaml 0.9.34+deprecated`、`tikv-jemallocator`、`tokio`。
  - 前 2 つは rumdl 0.3.2 で non-optional（feature gate なし）。
  - `tokio` は core 自身の native 依存。
- `cargo audit --no-fetch` の警告は baseline 10 のまま（差分なし）。
- `compute_content_hash` の blake3 → std::hash 切替は core では
  FileIndex を破棄しているため観測されない（**推測**）。
- `parallel`（rayon）削除による code_block_tools の直列化は
  既定無効機能のため体感差なしと推測されるが、**未計測**。
- wasm feature を避ける理由（rumdl の `#[wasm_bindgen(start)]` が
  web クレートと衝突）は ADR-0003:40-42 を前提としており、
  本 ADR で supersede する。

## 検証（Runtime Proof）

- `cargo check --all-targets --locked` / `cargo test --all-targets --locked` /
  `cargo clippy --all-targets --locked -- -D warnings`: 成功。
- wasm32 core / web check、wasm-pack test --node（4 件）: 成功。
- `cargo audit --no-fetch`: 脆弱性 0、警告 baseline 10。
- `./scripts/verify.sh` rc=0、E2E **233 項目すべて成功**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- ルート workspace の `rumdl` エントリを `default-features=false` に
  揃えるか削除するか。
- `parallel` 再導入の要否（性能ベンチ未実施）。
- 上流 rumdl が serde_yaml / tikv-jemallocator を optional 化するか
  （0.3.4 でも未変化、0.3 固定下では解消不可）。
- CI env_vars 削除の可否（共通ワークフロー側のディスク対策と
 実機 df 確認が必要）。
- ADR-0003 本文を修正するか、本 ADR の supersede のみで十分か。
