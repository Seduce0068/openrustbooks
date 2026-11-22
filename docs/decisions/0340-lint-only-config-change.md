---
type: Decision
title: "ADR-0340: lint 設定変更では AI service を再生成しない"
description: "commit_validated_config の再準備判定を ai 設定差分に限定し、lint のみ変更が生成失敗でブロックされないようにする決定。"
tags:
  - ADR
  - desktop
  - config
generated:
  by: dsh
  at: "2026-11-23T02:38:00+09:00"
status: accepted
---

## 背景

commit_validated_config は `ai_or_lint_changed()` が true なら
`prepare_ai_service()` を呼んでいた。
lint 設定のみを変更した場合でも AI service 再生成が走り、
provider 障害やオフライン時に無関係な lint 設定まで保存できなかった。

## 決定

- `ai_changed(current, merged)` を追加し、AI 設定の JSON 差分のみを判定する。
- `commit_validated_config` の第 4 引数を `ai_changed: bool` に変更し、
  AI 設定が変わった時だけ service を再準備する。
- lint のみの変更では現在 service を維持したまま config をコミットする。
- 旧 `ai_or_lint_changed` は既存 test 用に test module 内へ移す。

## 検証（Runtime Proof）

- 新規 test: lint のみ変更は `ai_changed=false` / `ai_or_lint_changed=true`、
  AI 変更は `ai_changed=true`。
- `cargo fmt --all -- --check`: rc=0。
- `cargo check --all-targets --locked`: rc=0。
- `cargo clippy --all-targets --locked -- -D warnings`: rc=0。
- `./scripts/verify.sh`: 全ゲート成功、E2E **263 passed / 0 failed**、WDIO gate rc=0。

## 未決定

- レガシー config command の CAS 集約（ADR-0269 の未決定を追跡）。
