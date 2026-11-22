---
type: Decision
title: "ADR-0349: cargo audit 警告の ID + 期限管理"
description: "既知 advisory 警告を ID と有効期限で管理し、新規 ID / 期限切れを fail-closed で拒否する決定。"
tags:
  - ADR
  - CI
  - dependency
generated:
  by: dsh
  at: "2026-11-23T04:13:00+09:00"
status: accepted
---

## 背景

verify.sh の audit ゲートは `--deny warnings` を使わず、
「unmaintained 等 10 件は既知 baseline」というコメントだけで警告を流していた。
advisory DB 更新時に警告が増えても detect できず、実数もコメントと乖離していた。

## 決定

- `cargo audit --json` の結果を Python で解析する。
- vulnerabilities は 1 件でもあれば fail-closed。
- warnings は次の ID + 期限付き allowlist のみ許可する:
  - RUSTSEC-2024-0370 / 0429 / 0436 / 0449
  - RUSTSEC-2025-0141
  - RUSTSEC-2026-0543
  - 有効期限はすべて 2027-03-31。
- 未登録 ID / 期限切れ / JSON 解析失敗 / cargo-audit 不在は fail-closed。
- advisory DB の鮮度ゲート（ADR-0294）は従来どおり先に実行する。

## 検証（Runtime Proof）

- bash -n / shellcheck: rc=0。
- `./scripts/verify.sh`: audit rc=0（許可リスト 6 IDs）、
  全ゲート成功、E2E **263 passed / 0 failed**、WDIO gate rc=0。

## 未決定

- chrono を time クレート等へ移行して警告そのものを解消するか。
