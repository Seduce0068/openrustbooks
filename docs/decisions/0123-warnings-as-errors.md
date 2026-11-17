---
type: Decision
title: "ADR-0123: warnings_as_errors の配線"
description: "LintConfig.warnings_as_errors を lint 判定に反映し、Warning をオプトインで不合格扱いにする決定を記録する。"
tags:
  - ADR
  - ai
  - lint
generated:
  by: dsh
  at: "2026-11-18T08:00:00+09:00"
status: accepted
---

## 背景

`LintConfig.warnings_as_errors`（既定 false）は設定・DTO に存在したが、
lint 判定（Error severity のみ不合格）で使われていなかった
（ADR-0110 / 0116 の未決定）。

## 決定

- `MarkdownLinter` が `warnings_as_errors` を保持する（`new(&LintConfig)` の
  シグネチャは不変）。
- 判定を `lint::lint_passed(issues, warnings_as_errors)` に集約する。
  - Error: 常に不合格
  - Warning: `warnings_as_errors == true` のときのみ不合格
  - Info: 常に無視
- `lint_and_fix()` と `validate_content()`（auto_fix=false 経路）の
  両方から同じヘルパーを呼ぶ。
- `LintIssue.severity` 自体は書き換えない（UI / retry feedback の正確性を維持）。
- `passed = converged && lint_passed`（ADR-0111）と
  auto-fix の `fix(10)` / `apply_fix_result` は変更しない。
- DTO / merge_secrets は変更しない（`warnings_as_errors` は DTO に含まれ、
  UI から設定可能なため復元対象外が正しい）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 判定述語にフラグを反映（採用） | 最小変更で既存挙動（既定 false）を維持 |
| severity を Warning→Error に書き換え | issues の意味が壊れ、UI と retry が不正確になる |
| 固定で Warning も不合格 | 既定挙動が変わり回帰リスク |

## 影響

- `warnings_as_errors=true` のとき、修正不能な Warning が残ると
  lint_passed=false となり、最大リトライまで再生成が回る
  （トークン消費増は意図された挙動。最終結果は ADR-0116 の `Ok` 維持）。
- 新規依存なし、unsafe なし。

## 検証

- 追加テスト 7 件（既定 false / true、Info 無視、Error 非依存、
  フラグ保持、auto_fix=false 経路など）。
- `cargo test --all-targets --locked`: core 180 + desktop 28 + web 2、
  wasm-test 3、すべて成功。
- `./scripts/verify.sh` を実行し、6 ゲート・trunk build・WASM パス検査・
  E2E 228 項目がすべて成功した。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定（warnings-auditor の指摘）

- `lint_markdown` コマンドに `passed` / strict 状態を返すか（現在は issues のみ）。
- retry 時の lint feedback で Warning を Error 扱いと明示するか。
- ADR-0116 の `strict_lint` オプトインを見送った判断との関係再整理。
- wasm の `WasmMarkdownLinter` が常に pass のスタブ（ADR-0110 / 0111 から継続）。
- `config_file` は rumdl 側未配線（ADR-0124 で判断）。`rule_config` は ADR-0124 で配線済み。
