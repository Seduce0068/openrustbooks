---
type: Decision
title: "ADR-0242: rumdl feature 選択の現状と ADR-0003 の履歴化"
description: "native / wasm とも rumdl featureless である現状を正とし、ADR-0003 の target 別 feature 選択を履歴として扱う決定を記録する。"
tags:
  - ADR
  - deps
  - wasm
  - ci
generated:
  by: dsh
  at: "2026-11-19T20:34:00+09:00"
status: accepted
---

## 背景

- ADR-0003 は rumdl の `default` / `wasm` feature を target 別に選択すると記録した。
  ただし後続の ADR-0155 / 0160 / 0168 / 0195 により上書き済みである。
- 現在 `core/Cargo.toml:66`（native）と `:72`（wasm）はともに
  `rumdl = { version = "0.3", default-features = false }`（featureless）である。
- `scripts/check-rumdl-featureless.sh` はこの完全一致 2 行を要求する。
  feature の追加、インデント変更、workspace 版の参照は rc=1 で失敗する（fail-closed）。
- ADR-0168 は workspace.dependencies の rumdl 宣言削除を決定した一方、
  ADR-0168:45 は「ADR-0003 本文修正」を未決定として残していた。

## 決定

- ADR-0003 の target 別 feature 選択は履歴とする。rumdl feature の現状は本 ADR を正とし、
  ADR-0003 本文は削除せず冒頭に `## 実装状況（2026-11-19）` を追加して参照させる。
- rumdl を workspace 依存へ一元化しない。core の native / wasm 2 行を直接指定として維持する。
- `wasm` feature は有効化しない（rumdl が公開する同名 `init` export の衝突、ADR-0160）。
- featureless 2 行とゲートの完全一致要件を維持し、Cargo.toml / Cargo.lock /
  ゲートスクリプトは本次で変更しない。

## 根拠

- Cargo の feature は加算で解決される。いったん有効化した `default` / `native` / `wasm`
  を下流の `default-features = false` で打ち消すことはできない。feature が 1 つでも
  加わると tower-lsp / tokio / notify / jemalloc 等が再加入し、
  ゲートの期待文字列との完全一致が崩れる（ADR-0155 / 0195）。
- MSRV 1.95（ADR-0214）の Cargo で workspace 経由の `default-features = true` を
  member 側から上書きできない制限がある可能性は **推測** であり、未検証である。
  この推測に依存せず、直接指定とゲートで featureless を担保する判断を採る。
- `scripts/check-rumdl-featureless.sh` は宣言行の完全一致と総数 2 行を厳密検査するため、
  workspace 版 rumdl を追加すると decl_count=3 で失敗する（ADR-0195）。

## 未決定

- rumdl 以外を含む workspace 依存の一元化方針（本 ADR は rumdl の workspace 化を否定）。
- 未実装 Wasm API の非公開化（R2-crates#3。ADR-0160 の公開 API 境界を継続）。
- バンドルサイズの before / after 計測とサイズゲート（ADR-0155 / 0160 から継続）。

## 参照

- [ADR-0003: 共通コアをターゲット別依存でNative/Wasm両対応にする](0003-native-wasm-target-split.md)
- [ADR-0155: rumdl の feature 削減](0155-rumdl-feature-reduction.md)
- [ADR-0160: Wasm Markdown linter の実配線](0160-real-wasm-lint.md)
- [ADR-0168: workspace の rumdl 宣言削除](0168-workspace-rumdl-cleanup.md)
- [ADR-0195: rumdl featureless の静的ゲート](0195-rumdl-feature-gate.md)
- [ADR-0214: MSRV 1.95 の宣言](0214-msrv.md)
