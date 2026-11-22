---
type: Decision
title: "ADR-0320: config 退避時の権限失敗ロールバック"
description: "権限付与失敗時に退避を元へ戻し、原文保持を優先する決定。"
tags:
  - ADR
  - config
  - security
generated:
  by: dsh
  at: "2026-11-22T13:42:00+09:00"
status: accepted
---

## 背景

backup_failed_config は rename 成功後に set_permissions(0o600) が失敗すると、
機微情報を含む config.toml を 0644 のまま `*.failed-*` に残していた。
呼び出し側は「退避失敗」と記録するが、実際には退避済みで状態が食い違った。

## 決定

- 本体を `backup_failed_config_with` へ分離し、
  restrict / rollback を注入可能にする（unsafe / 新規依存なし）。
- restrict(backup) 失敗時に `rename(backup, config_file)` で元へ戻す。
- ロールバック成功時は元ファイルへ 0600 適用を試みる。
- ロールバック失敗時は退避先を残し、原文保持を優先する（core::io の既存パターン）。
- いずれの場合も固定文言
  `failed to secure the configuration backup` の CoreError を返す。
- パス / 設定値はエラーへ出さない。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 元へロールバック + 0600 再適用（採用） | データ保持と権限制限を両立できる |
| 退避先を削除して fail | 原文を失うため却下 |
| 権限失敗を無視 | 機微ファイルが world-readable に残る |

## test 方針

- 実環境で chmod 失敗を起こすには他人所有ファイル / 特権が必要で CI 再現が困難なため、
  注入口による模擬 test を採用する。
- 模擬 test で本番と同じ分岐（restrict 失敗 → rollback → 再 restrict → 固定文言）を通す。

## 検証（Runtime Proof）

- core test: **355 passed / 0 failed**（backup_tests 9 件含む）。
- `cargo fmt --all -- --check`: rc=0。
- `cargo check --all-targets --locked`: rc=0。
- `cargo clippy --all-targets --locked -- -D warnings`: rc=0。
- wasm core / web check: rc=0。
- `./scripts/verify.sh`: 全ゲート成功、E2E **256 passed / 0 failed**、WDIO gate rc=0。

## 未決定

- ロールバック失敗時に残った `*.failed-*` の権限露出リスクの扱い。
- 実環境での chmod 失敗再現 test。
