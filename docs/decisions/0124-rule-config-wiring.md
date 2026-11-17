---
type: Decision
title: "ADR-0124: rule_config の rumdl 配線と config_file の未対応"
description: "rule_config を rumdl Config に反映し、config_file は extends のリスクから未対応とする決定を記録する。"
tags:
  - ADR
  - ai
  - lint
  - security
generated:
  by: dsh
  at: "2026-11-18T08:35:00+09:00"
status: accepted
---

## 背景

`LintConfig.rule_config` と `config_file` は設定・DTO に存在したが、
`MarkdownLinter::new()` は `Config::default()` しか使わず両方を無視していた
（ADR-0123 の未決定）。

## 決定

- `rule_config` を rumdl `Config.rules` に反映する。
  - キーは `MD` + ASCII 3 桁のみ（`MD１２３` 等の Unicode 数字も拒否）。
  - 値は JSON object のみ。各値を `serde_json::from_value::<toml::Value>` で変換し、
    失敗は `CoreError::validation`（fail-closed）。
  - 未知ルール ID（`MD999` 等）は `create_rule_by_name()` で拒否。
  - 変換済み `Config` を `all_rules()` と `self.config` の両方に使う。
  - 優先順位は `disable_rules` > `enable_rules` > `rule_config` のパラメータ
    （ルール固有設定で除外ルールを復活させない）。
- `config_file` は **未対応のまま**。rumdl の `extends` が
  相対／絶対／`~/`／`$VAR`／`${VAR}` を展開して任意ファイルを読み込むため、
  ADR-0097 の fail-closed 方針と矛盾する。
  将来対応するなら、パス allowlist＋`extends`／環境変数展開の無効化が必須条件。
- wasm 側（`WasmMarkdownLinter`）は今回の対象外（ADR-0110 / 0123 から継続）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| rule_config を配線（採用） | ローカル設定のみ、型検証も可能で安全 |
| config_file も配線 | `extends` による任意パス読込で却下 |
| 両方未対応 | 設定項目が死に続ける |

## 影響

- `.markdownlint.yaml` 相当のファイルは相変わらず読まれない。
- `rule_config` でルール固有パラメータ（例 `MD013.line_length`）が効く。
- rumdl 側の型違反時挙動（stderr 警告＋既定値フォールバック、
  呼び出し側エラーなし／warning どまり）は既知の制限として残る
  （完全 fail-closed ではない）。
- 新規依存なし、unsafe なし。

## 検証

- 追加テスト 6 件（有効キー、line_length が実際に効くこと、
  不正キー、非 object、変換失敗、既定挙動の不変）。
- `cargo test --all-targets --locked`: core 186 + desktop 28 + web 2、
  wasm-test 3、すべて成功。
- `./scripts/verify.sh` を実行し、6 ゲート・trunk build・WASM パス検査・
  E2E 228 項目がすべて成功した。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定（rule-config-auditor の指摘）

- 深い入れ子／巨大配列／キー数の上限（現在なし、ローカル設定のため自己 DoS が主）。
- rumdl の型違反警告を利用者へ表面化するか、事前型検証を行うか。
- `severity` / `enabled` のサポート（`RuleConfig.severity` は None のまま、
  `apply_per_rule_enabled()` は未呼び出し）。
- opt-in ルールの扱い、`rule_config` を DTO に公開するか。
- `config_file` 対応の条件（allowlist＋`extends` 無効化）とパス root の定義。
- wasm 側 lint 設定の揃え方。
