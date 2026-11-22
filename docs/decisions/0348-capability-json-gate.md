---
type: Decision
title: "ADR-0348: wdio capability の静的ゲート"
description: "wdio permission がテスト用 capability JSON に閉じ、tauri.conf.json へ混入しないことを検査する決定。"
tags:
  - ADR
  - Tauri
  - CI
generated:
  by: dsh
  at: "2026-11-23T04:02:00+09:00"
status: accepted
---

## 背景

`desktop/capabilities/wdio/wdio.json` は debug + feature 限定の
`wdio:default` / `wdio-webdriver:default` permission を持つが、
静的ゲートは tauri.conf.json と source のみを検査していた。
設定変更で test permission が本番設定へ混入しても検出できなかった。

## 決定

- `check-tauri-plugin-config.sh` で wdio capability JSON を検査する:
  - identifier は `wdio` 固定。
  - permissions は文字列配列で `wdio:default` / `wdio-webdriver:default` / `core:default` を含む。
  - `tauri.conf.json` に test permission 文字列が出ていないこと。
- 成功文言に wdio capability の分離を明記する。

## 検証（Runtime Proof）

- bash -n / shellcheck: rc=0。
- `bash scripts/check-tauri-plugin-config.sh`: rc=0。
- `./scripts/verify.sh`: 全ゲート成功、E2E **263 passed / 0 failed**、WDIO gate rc=0。

## 未決定

- capability 単位の実効 ACL を runtime で検証する仕組み（ADR-0332 / 0348 は静的ゲート）。
