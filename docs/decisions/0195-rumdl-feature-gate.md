---
type: Decision
title: "ADR-0195: rumdl featureless の静的ゲート"
description: "core/Cargo.toml の rumdl default-features=false をローカルと CI の両方で静的検査する決定を記録する。"
tags:
  - ADR
  - ci
  - deps
generated:
  by: dsh
  at: "2026-11-19T07:53:00+09:00"
status: accepted
---

## 背景

rumdl の feature は Cargo.lock に記録されないため、
`core/Cargo.toml` の 2 行（native / wasm）が
`default-features = false` の唯一の担保だった（ADR-0155 / 0168）。
将来 `true` に戻ると tower-lsp / notify / jemalloc 等が再加入し、
lock 差分なしで CI ディスク・ビルドが劣化し得る（R6: TD-M06）。

## 決定

- `scripts/check-rumdl-featureless.sh` を追加する:
  - `set -euo pipefail`。
  - 第 1 引数で対象ファイルを差し替え可能、既定は
    スクリプト位置基準の `../core/Cargo.toml`（CWD 非依存）。
  - `rumdl = { version = "0.3", default-features = false }`
    がちょうど 2 行あることを厳密検査する。
  - 不一致／行数違反／ファイル不在は行番号付きで報告し rc=1。
- `scripts/verify.sh` の `check_rumdl_featureless()` は
  新規スクリプトを呼ぶ薄いラッパーへ置換する（単一情報源）。
- `.github/workflows/ci.yml` の `extra_gates` に
  `bash scripts/check-rumdl-featureless.sh` を追加する。
  - 共通ワークフロー（`rust-gates.yml@v1`）自体は変更しない。
- 期待値（version 0.3 / 2 行）を固定する。更新時はゲートも同時に変更する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 独立スクリプト＋ verify / CI から利用（採用） | 単一情報源でローカルと CI の乖離を防ぐ |
| verify.sh にだけ追加 | CI では発動しない（共通ワークフローは verify.sh を呼ばない） |
| 共通ワークフローを変更 | 外部リポジトリ／v2 が必要で本次範囲外 |

## 影響

- `core/Cargo.toml` にインデントや余分な feature が入るとゲートが落ちる
  （fail-closed、意図的）。
- rumdl 更新時に期待文字列と行数を同時に更新する必要がある。
- `wasm` feature は同名 `init` export 衝突で禁止（ADR-0160 を継続）。
- workspace 版 rumdl は削除済み（ADR-0168）。再導入すると decl_count=3 で失敗する。

## 検証（Runtime Proof）

- bash -n / shellcheck / shfmt -d: 両スクリプトすべて成功。
- ネガティブテスト（一時ファイル、リポジトリの Cargo.toml は無変更）:
  - `default-features=true` 化 → rc=1（行番号付き）。
  - 1 行／3 行／インデント付き → すべて rc=1。
  - positive（既定パス／一時コピー）→ rc=0。
  - 実 Cargo.toml の SHA-256 は前後同一。
- `./scripts/verify.sh` rc=0、
  `rumdl-featureless rc=0`、E2E **250 項目すべて成功**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- 共通ワークフロー側で `verify.sh` を実行する入力を追加するか
  （CI-H01、外部リポジトリ変更が必要）。
- rumdl 更新時の期待値更新手順を自動化するか。
- ファイル読取不可時のエラー文言を詳細化するか（現状は awk の stderr 依存）。
