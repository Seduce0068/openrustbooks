---
type: Decision
title: "ADR-0246: workspace 依存の第一段統一"
description: "feature 差分が無い core 依存を workspace.dependencies 参照へ統一する決定を記録する。"
tags:
  - ADR
  - deps
  - cargo
generated:
  by: dsh
  at: "2026-11-19T21:46:00+09:00"
status: accepted
---

## 背景

`core/Cargo.toml` は workspace に既に定義済みの依存を直接 `version` で
重複宣言していた（R6 R2-crates#5）。
Cargo の feature は加算で解決され、後から member 側で
`default-features = false` へ打ち消せないため、
feature 差分のある依存を安易に workspace 化すると
意図しない依存が復活するリスクがある。

## 決定

- version / feature が workspace と完全一致する 18 件を
  `{ workspace = true }` へ変更する:
  - 通常依存: serde / serde_json / thiserror / tracing / anyhow /
    comrak / async-trait / similar / futures / bytes / toml
  - native: tokio / tracing-subscriber / tiktoken-rs / async-openai /
    figment / directories / reqwest
- 除外する:
  - rumdl 2 行（`check-rumdl-featureless.sh` が完全一致を要求、
    ADR-0242）
  - chrono / uuid / wasm dev tokio（feature 加算）
  - workspace 未定義の依存（zip / quick-xml / imagesize / wasm-bindgen 系等）
- serde derive と comrak `default-features = false` は
  workspace 側が継承する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 完全一致のみ workspace 化（採用） | 意味を変えず重複を削減できる |
| 全依存を workspace 化 | feature 加算で tower-lsp / tokio / notify が復活する |
| 現状維持 | 定義の二重管理が残る |

## 影響

- `core/Cargo.toml` は 20 行追加 / 19 行削除。
- `Cargo.lock` は差分なし（feature は lock に記録されない）。
- `check-rumdl-featureless.sh` は期待 2 行を維持。
- E2E 255 維持。

## 検証（Runtime Proof）

- `bash scripts/check-rumdl-featureless.sh`: OK（2 行維持）。
- `cargo metadata --locked --no-deps --format-version 1`: rc=0。
- `cargo check --all-targets --locked` /
  `cargo test --all-targets --locked` /
  `cargo clippy --all-targets --locked -- -D warnings`: 成功。
- wasm32 core / web check、`wasm-pack test --node --locked`（15 passed）: 成功。
- `./scripts/verify.sh`: 全 14 ゲート成功、E2E **255 項目すべて成功**。
- `Cargo.lock` SHA-256 は変更前後で同一。

## 未決定

- feature 加算を伴う chrono / uuid / wasm dev tokio の workspace 化。
- MSRV 1.95 における workspace 側 `default-features` override 制限の真偽
  （ADR-0242 の「推測・未検証」を継続）。
- ~~Wasm API 第二段以降の AI cluster（ADR-0156 supersede 必要）~~
  ADR-0256 で削除を決定・完了（ADR-0156 の AI 部分を supersede）。
- バンドルサイズ before / after 計測（ADR-0160 / 0242 を継続）。 比較可能 before は [ADR-0267](0267-bundle-size-baseline.md) に記録済み。
