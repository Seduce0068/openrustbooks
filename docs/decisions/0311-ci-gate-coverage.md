---
type: Decision
title: "ADR-0311: CI ゲートの静的検査とシークレットスキャン"
description: "no-external-font-refs / 配布パス / gitleaks を verify.sh と CI extra_gates の両方へ配線する決定。"
tags:
  - ADR
  - CI
  - security
generated:
  by: dsh
  at: "2026-11-22T10:31:00+09:00"
status: accepted
---

## 背景

verify.sh だけが no-external-font-refs / 配布パス / シークレットスキャンを実行し、
CI の extra_gates には無かった。
gitleaks を導入する場合は pre-commit を主軸とし、
CI を最終ゲートとして併設する必要がある。

## 決定

- `check-no-external-font-refs.sh` を追加し、verify.sh の inline gate を置換する。
- `check-distribution-path.sh` を追加する。
  - 絶対パス / repo 外 / `readlink -m` 正規化を検査する。
  - manifest の必須キー、`e2e expected: 256`、`passed >= 1 / failed: 0`、
    SHA256 行形式を検査する。
  - 毎回の自己検査で repo 内 / 相対 / `..` / repo 内 symlink /
    不正 manifest を拒否する。
- `check-secrets.sh` を追加し、gitleaks detect を実行する。
  未導入時は fail-closed（rc=127）とする。
- `.pre-commit-config.yaml` に gitleaks hook を追加する（pre-commit 主軸）。
- verify.sh と `.github/workflows/ci.yml` の extra_gates の両方へ配線する。
- 共通 workflow 自体は変更しない。
- trunk build + E2E / WDIO / 配布保持 / artifact upload は
  billing 解消後チェックリストへ明示する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 静的ゲートを CI に追加（採用） | 共通 workflow を変えずに再発を検出できる |
| 共通 workflow 直接変更 | 事前承認 / self-test / tag 更新が必要で今回は行わない |
| gitleaks 未導入をスキップ | fail-open になるため却下 |

## 検証（Runtime Proof）

- `bash -n` / `shellcheck` / `shfmt -d` / PyYAML parse / markdownlint: rc=0。
- no-external-font: 実 file rc=0 / 混入 file rc=1。
- distribution-path: 正常 rc=0 / repo 内・相対・`..`・symlink・
  expected=253 / passed:0 / failed:1 / SHA256 欠落 rc=1/2。
- secrets: 実走査 1376 commits / no leaks found / rc=0、
  gitleaks 不在 PATH で rc=127（fail-closed）。
- `./scripts/verify.sh`: 全ゲート成功、E2E **256 passed / 0 failed**、WDIO gate rc=0。

## 未決定

- manifest の期待件数 256 を `Checker(expected=N)` から動的取得するか（現状は現行契約として固定）。
- self_test 一時ディレクトリの EXIT trap。
- CI 側 gitleaks の導入方法（共通 workflow 側または extra_gates 内のチェックサム付き取得 / go install）。
- billing 解消後の実 CI 検証（CI-H01）。
