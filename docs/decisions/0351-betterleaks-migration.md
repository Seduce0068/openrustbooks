---
type: Decision
title: "ADR-0351: Betterleaks へのシークレットゲート移行"
description: "Gitleaks を Betterleaks v1.10.1 へ置き換え、pre-commit 主軸 + CI 最終ゲートを確立する決定。"
tags:
  - ADR
  - CI
  - security
generated:
  by: dsh
  at: "2026-11-23T13:10:00+09:00"
status: accepted
---

## 背景

CI の extra_gates は gitleaks の存在検査で常時 fail していた。
ユーザー指示により、後継ツールである Betterleaks v1 系へ移行する。

## 決定

- pre-commit は Betterleaks v1.10.1 の公式 hook
  (`betterleaks git --pre-commit --redact --staged --verbose`) を主軸とする。
- `scripts/check-secrets.sh` は Betterleaks の全履歴スキャン
  (`betterleaks git <repo> --redact --verbose`) を CI / verify.sh の最終ゲートとして実行する。
- `scripts/install-betterleaks.sh` が v1.10.1 の tarball を取得し、
  公式 `checksums.txt` とスクリプト内固定 digest の二重照合後に配置する。
- 未導入 / 走査失敗 / 検出は fail-closed。
- 既知 false positive は fingerprint 単位で `.betterleaksignore` に
  例外 ID・理由・有効期限を添えて管理し、ルール全体の無効化は行わない。
- 全履歴スキャンを維持するため、作業ツリー走査での置き換えは行わない。

## 検証（Runtime Proof）

- `betterleaks version 1.10.1`。
- pinned install: `sha256 verified (pinned + release)` / version 一致。
- 負向: ダミー文字列を含む一時 git repo で rc=1 / 検出。
- 正向: クリーン repo と本リポジトリで rc=0 / no leaks found。
- `bash -n` / shellcheck / shfmt -d / markdownlint: rc=0。
- `pre-commit install`: `.git/hooks/pre-commit` 生成を確認。
- `./scripts/verify.sh`: 全ゲート成功、E2E 263 / 0、WDIO gate rc=0。

## 未決定

- `.betterleaksignore` の有効期限を自動検証するゲートの追加。
