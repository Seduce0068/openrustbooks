---
type: Decision
title: "ADR-0102: extra_headers の IPC マスク"
description: "OpenAiConfig.extra_headers を IPC DTO と Debug 出力からマスクする決定を記録する。"
tags:
  - ADR
  - security
  - desktop
generated:
  by: dsh
  at: "2026-11-18T03:15:00+09:00"
status: accepted
---

## 背景

`OpenAiConfig.extra_headers` は DTO（`OpenAiConfigView`）で JS へそのまま公開され、
`update_config` / `update_ai_config` からも任意値が設定できた。
利用者が `Authorization` 等を入れると平文で config.toml と IPC に露出する
（ADR-0098 の未決定、baseurl-auditor の指摘）。

## 決定

- `OpenAiConfigView` から `extra_headers` を削除し、JSON にキーも値も出さない。
- `merge_secrets()` を拡張し、incoming の `extra_headers` が空またはマスク
  sentinel（`***`）を含む場合は現在の実値を保持する。
  GUI の往復で消えないようにし、削除は config.toml 直接編集に限定する。
- `OpenAiConfig` の手実装 Debug で `extra_headers` の値を `"***"` にマスク
  （キー名は維持）。
- 既存の `api_key` / `client_secret` / `refresh_token` マージ、
  `disable_ai_service()`、`validate_ai_base_urls()` は変更しない。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| DTO から削除＋マージ保持（採用） | 機微ヘッダを JS に出さず、設定の往復も安全 |
| 値だけマスク | キー名から利用目的が推測される |
| headers 機能を削除 | 正当な proxy / gateway 利用を壊す |

## 影響

- 設定画面から `extra_headers` の閲覧・編集・削除ができなくなる。
  削除・変更は config.toml 直接編集が必要（UI 案内は将来課題）。
- 既存 config.toml の headers は保持され、AI リクエストには引き続き送信される。
- 新規依存は無い。

## 検証

- 追加テスト 5 件（DTO JSON 非含有、マージ保持、実値置換境界、Debug マスク、
  TOML 往復）。
- `cargo test --all-targets --locked`: core 148 + desktop 13、すべて成功。
- `./scripts/verify.sh` を実行し、6 ゲート・trunk build・WASM パス検査・
  E2E 228 項目がすべて成功した。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- ipc-auditor の指摘: `general.data_dir` / `cache_dir` / `lint.config_file` /
  `local_roots[].path` / `recent_paths` などのパス情報がまだ DTO に含まれる（中優先度）。
- `local_roots` を JS から変更できる問題（ADR-0097 の containment はルート内部のみ）。
- 同意ダイアログの UI。
EOF
