---
type: Decision
title: "ADR-0358: general の死設定を読み取り専用化し未配線を告知"
description: "language / theme / log_level の DTO 契約を維持したまま、UI を読み取り専用化し desktop 側で現在値を復元する決定。"
tags:
  - ADR
  - settings
  - wasm
generated:
  by: dsh (lead)
  at: "2026-11-28T05:30:00+09:00"
status: accepted
---

## 背景

3 設定は DTO を往復するだけで表示・ログ出力へ反映されず、UI は編集可能だった。
ADR-0304 の除外 9 件には含まれず、未配線告知も無かった。

## 決定

- DTO からフィールドを除外しない（ADR-0322 の 12 ペア契約と import/export 互換を維持）。
- web の 3 input を常時 readonly にし、on:input を削除。id / label / prop:value は維持。
- 一般設定へ固定日本語の未配線告知を 1 行追加。
- desktop の merge_secrets で 3 値を現在値へ復元し、JS 書き換えの永続化を fail-closed で防ぐ。

## 検証（Runtime Proof）

- `cargo fmt --all -- --check` / `cargo check --all-targets --locked` / `cargo test --all-targets --locked` /
  `cargo clippy --all-targets --locked -- -D warnings`: rc=0。
- `cargo check -p openrustbooks-web --target wasm32-unknown-unknown --locked`: rc=0。
- `npm run build:wdio` / `bash scripts/ci-wdio-test.sh`: rc=0（settings 3 passing）。
- E2E `Checker(expected=273)` は不変（既存 check 名・件数を変更せず）。

## 未決定

- 将来 i18n / theme を実際に配線する場合の DTO 復活手順（ADR-0304 の未決定を継承）。
