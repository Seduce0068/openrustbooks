---
type: Decision
title: "ADR-0110: rumdl auto-fix の実装"
description: "rumdl の fix API を用いて Markdown lint の自動修正を実装し、auto_fix 設定を配線する決定を記録する。"
tags:
  - ADR
  - ai
  - lint
generated:
  by: dsh
  at: "2026-11-18T05:00:00+09:00"
status: accepted
---

## 背景

`core/src/ai_markdown/lint.rs` の `lint_and_fix()` は
`fixed_content = 入力` / `fixes_applied = 0` の no-op で、
コメントには「auto-fix not available in current rumdl version」とあった。
しかし調査の結果、rumdl 0.3.2 には
`document_run::DocumentRun::fix(max_iterations)` が公開されており、
コメントは誤りだった。REQ-010 は rumdl による自動整形を求めている。
また `LintConfig.auto_fix` / `PipelineConfig.auto_fix` は死設定だった。

## 決定

- `MarkdownLinter` に rumdl `Config` を保持させ、`lint()` / `lint_and_fix()`
  で同じ設定を使う（`new(&LintConfig)` のシグネチャは不変）。
- `lint_and_fix()` は `DocumentRun::new(...).fix(10)` を呼び、
  `fixed_content = 修正後文字列`、`fixes_applied = FixResult.rules_fixed` を返す。
- 修正後に再 lint し、`passed` / `issues` を修正後の内容で作り直す
  （修正前の issues のままだと retry 判定が誤るため）。
- `execute_pipeline` は `self.config.auto_fix` が true のときだけ
  `lint_and_fix()` を呼び、false のときは `lint()` のみ呼ぶ
  （死設定を解消）。
- 反復上限は rumdl 自身の wasm 実装に倣い 10。
- 新規依存は無い。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| rumdl fix API を実装（採用） | REQ-010 を満たし、新規依存も不要 |
| no-op を設計として明文化 | 要件未達が残る |
| `auto_fix` 設定を削除 | config.toml / DTO / wasm API の互換性を壊す |

## 影響

- AI 生成後に lint 自動修正が走り、修正後本文が `final_content` として返る。
- `auto_fix=false` なら内容は変わらず lint 検査のみ行う。
- 修正後再 lint により、retry 判定が実際の修正結果を反映する。
- デフォルト `auto_fix=true`（AI 自体は ADR-0099 で既定無効）。
- rumdl は改行種別と BOM を保持する（lint.rs 側の責務）。

## 検証

- 追加テスト 5 件（fix 適用と再 lint、クリーン本文は不変、
  auto_fix=false は不変、既存 lint 挙動の回帰、フラグ分岐）。
- `cargo test --all-targets --locked`: core 163 + desktop 28、すべて成功。
- `./scripts/verify.sh` を実行し、6 ゲート・trunk build・WASM パス検査・
  E2E 228 項目がすべて成功した。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定（autofix-safety-auditor の指摘）

- `FixResult.converged` / `conflicting_rules` / `conflict_cycle` を破棄しており、
  未収束時の fail-closed／ベストエフォートの判断が未定。
- `fixes_applied` は「修正したルール数」であり編集回数ではない（名称の誤解に注意）。
- `refine` / `front_matter` のリトライ入力は元原稿を使うため、
  修正後本文（`fixed_content`）が AI の再生成に渡らない。
- `lint_and_fix_markdown` コマンドは本文を直接書き換え得るが、
  UI 未接続のため適用前差分確認の要否は未確認。
- `LintConfig.auto_fix` のコメント「Auto-fix on save」と実際の保存処理は
  乖離している（保存時 lint は未配線）。
- `warnings_as_errors` / `config_file` / `rule_config` は rumdl 側に未配線。
- wasm の `WasmMarkdownLinter` はスタブのまま（native と挙動が異なる）。
