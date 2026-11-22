---
type: Decision
title: "ADR-0327: WDIO trunk serve の lock 固定"
description: "WDIO の trunk serve に --locked --offline を付け、readiness 後 / cleanup 前にも lock 不変を検査する決定。"
tags:
  - ADR
  - WDIO
  - reproducibility
generated:
  by: dsh
  at: "2026-11-22T23:25:00+09:00"
status: accepted
---

## 背景

ci-wdio-test.sh は cargo build 前後に Cargo.lock / package-lock.json の差分を検査していたが、
trunk serve には --locked / --offline が無く、readiness 中 / E2E 中の lock 変更を検出できなかった。

## 決定

- `trunk serve --port ... --locked --offline` で起動する。
- lock 不変検査を 4 時点で行う:
  - ビルド前
  - ビルド後
  - readiness 成立直後
  - cleanup 前（E2E 成功経路）
- 差分検出時は fail-closed で終了する。
- port / PGID / listener の既存検証構造は変更しない。
- runbook-webdriver.md に同じ手順を記載する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| serve も lock 固定 + 4 時点検査（採用） | 検証対象と lock 状態を一致させられる |
| serve は開発サーバだから例外 | WDIO が検証する WebView と lock が乖離する |
| E2E 後の検査のみ | readiness 中の変更を検出できない |

## 検証（Runtime Proof）

- bash -n / shellcheck: rc=0。
- shfmt -d: 差分なし。
- markdownlint: 0 issues。
- `./scripts/verify.sh`: 全ゲート成功、E2E **256 passed / 0 failed**、WDIO gate rc=0。

## 未決定

- npm run test:wdio 失敗経路でも cleanup 前 lock 差分を可視化するか（現状は失败で set -e により trap へ入る）。
