---
type: Decision
title: "ADR-0103: 設定 DTO のパス情報除外"
description: "設定 IPC の DTO からローカルパス情報を除外し、マージで保持する決定を記録する。"
tags:
  - ADR
  - security
  - desktop
generated:
  by: dsh
  at: "2026-11-18T03:30:00+09:00"
status: accepted
---

## 背景

`ConfigView` が `GeneralConfig`（`data_dir` / `cache_dir`）、`LintConfig`
（`config_file` / `rule_config`）、`StorageConfigView`（`local_roots[].path` /
`recent_paths`）、`OllamaConfig`（`options`）、`AiConfigView`（`model_overrides`）
を丸ごと JS へ返していた（ipc-auditor の指摘、ADR-0102 の未決定）。
ユーザー名や最近開いた文書名が露出するリスクがあった。

## 決定

- DTO 用 VIEW 型を追加し、次を除外する。
  - `GeneralConfigView`: `data_dir` / `cache_dir`
  - `LintConfigView`: `config_file` / `rule_config`
  - `StorageConfigView`: `recent_paths`、`local_roots[].path`
    （`LocalRootView` は `id` / `name` / `read_only` / `watch` のみ）
  - `OllamaConfigView`: `options`
  - `AiConfigView`: `model_overrides`
- `merge_secrets()` を拡張し、除外フィールドは `current` から保持する。
  - `local_roots[].path` は `id` が一意に一致した場合のみ復元し、
    空 id / 不一致 / 重複 id では誤マージしない。
  - 他の除外フィールドは JS に開示されないため無条件に `current` を採用。
- `update_lint_config` も `merge_secrets` を通し、
  `config_file` / `rule_config` が消えないようにした。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| VIEW 型で除外＋マージ保持（採用） | JS に出さず、往復で設定が消えない |
| ファイル名のみ返す | `recent_paths` の UX は改善するが今回は対象外（将来検討） |
| `ConfigView` を返さない | 将来の UI 接続を完全に阻害する |

## 影響

- 設定画面からパス情報の閲覧・変更ができなくなる
  （現状 UI 未接続のため即時の影響はなし）。
- パスの変更・新規ルート追加は config.toml 直接編集が必要。
- `create_storage()` は `AppState` の `AppConfig` を直接参照するため、
  DTO 除外の影響を受けない（`merge_secrets` の保持が前提）。
- 新規依存は無い。

## 検証

- 追加テスト 5 件（禁止キー非含有、維持キー存在、除外フィールド保持、
  local_root の id マージ、あいまい id のスキップ）。
- `cargo test --all-targets --locked`: core 148 + desktop 18、すべて成功。
- `./scripts/verify.sh` を実行し、6 ゲート・trunk build・WASM パス検査・
  E2E 228 項目がすべて成功した。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 解決済み

- 送信プレビュー＋ネイティブの同意ダイアログは ADR-0104 で実装済み。

## 未決定

- `recent_paths` をファイル名のみで返す代替表現（dto-ux-auditor の推奨）。
- `data_dir` / `cache_dir` / `config_file` の「設定済み bool」表現。
- 将来 UI 接続時に config.toml の場所を案内する表示。
