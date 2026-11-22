---
type: Decision
title: "ADR-0335: lint を AI service 依存から分離"
description: "lint / auto-fix はローカル処理として LintConfig から直接構築し、既定の AI 無効でも動作可能にする決定。"
tags:
  - ADR
  - desktop
  - lint
generated:
  by: dsh
  at: "2026-11-23T01:54:00+09:00"
status: accepted
---

## 背景

lint_markdown / lint_and_fix_markdown は state.ai_service() を必須としていた。
AI は既定で無効（ai.enabled=false）なので、出荷既定では lint ボタンが
常に「AI service not initialized」で拒否された。
しかし lint / auto-fix は core::ai_markdown::lint::MarkdownLinter のローカル処理だった。

## 決定

- AppState に `lint_config()` を追加し、現在の LintConfig を返す。
- lint 2 コマンドは `MarkdownLinter::new(&state.lint_config())` から直接構築する。
- ai_service() / pipeline() への依存を lint 経路から削除する。
- エラーはすべて ipc_error を通し、rule_config の値やパスを露出しない。
- command 名 / 引数（content のみ）/ 戻り値 / 登録は変更しない。

## 検証（Runtime Proof）

- 新規 test 4 件:
  - 既定 AI 無効 + service なしで lint 成功（MD009 検出）。
  - disable_rules / enable_rules が結果へ反映。
  - warnings_as_errors が結果へ反映。
  - lint_config() が現在設定の clone を返す。
- desktop test: **180 passed / 0 failed**。
- `cargo fmt --all -- --check`: rc=0。
- `cargo check --all-targets --locked`: rc=0。
- `cargo clippy --all-targets --locked -- -D warnings`: rc=0。
- `./scripts/verify.sh`: 全ゲート成功、E2E **263 passed / 0 failed**、WDIO gate rc=0。

## 未決定

- 未使用の AppHandle 引数を command から削除するか（現状は test パターン維持のため残存）。
