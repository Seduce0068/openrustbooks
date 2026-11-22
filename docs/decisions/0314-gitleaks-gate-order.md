---
type: Decision
title: "ADR-0314: gitleaks を CI 最終ゲートへ配置"
description: "シークレットスキャンを extra_gates の末尾に置き、未導入時でも他の静的ゲートが実行されるようにする決定。"
tags:
  - ADR
  - CI
  - security
generated:
  by: dsh
  at: "2026-11-22T11:17:00+09:00"
status: accepted
---

## 背景

gitleaks 未導入時の `exit 1` が extra_gates の途中にあったため、
後続 11 ゲート（rumdl / dependency / version / e2e-count / csp /
tauri-plugin / updater / capability / wasm×2 / wasm-test）が到達不能だった。
CI は常時 red のまま回帰を検出できなかった。

## 決定

- gitleaks の if / else / exit 1 ブロックを extra_gates の末尾へ移動する。
- 先頭 13 ゲートを先に実行し、最後にシークレットスキャンを行う。
- 未導入時は最後で fail-closed とし、検出/rc 伝播も維持する。
- 共通 workflow 自体は変更しない。
- gitleaks の導入選択肢（共通 workflow 側 / extra_gates 内）は
  docs/runbook-verified-build.md に参照する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 末尾配置（採用） | 共通 workflow を変えずに全ゲートを実行できる |
| 先頭で import | 共通 workflow 変更または外部取得が必要 |
| 未導入をスキップ | fail-open になるため却下 |

## 検証（Runtime Proof）

- PyYAML parse: rc=0 / extra_gates の bash -n: rc=0。
- markdownlint: 0 issues。
- gitleaks が末尾（ci-wasm-test.sh の後）に配置され、
  前方 13 ゲートが先に実行されることを確認。
- `./scripts/verify.sh`: 全ゲート成功、E2E **256 passed / 0 failed**、WDIO gate rc=0。

## 未決定

- CI 側 gitleaks の恒久導入方法（共通 workflow 側または extra_gates 内）。
- billing 解消後の実 CI 緑化と artifact upload（CI-H01）。
