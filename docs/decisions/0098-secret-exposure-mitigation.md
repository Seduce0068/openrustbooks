---
type: Decision
title: "ADR-0098: 秘密情報の露出緩和"
description: "API キー等の IPC 露出をマスクし、Debug とファイル権限を硬化する決定を記録する。"
tags:
  - ADR
  - security
  - desktop
generated:
  by: dsh
  at: "2026-11-18T02:20:00+09:00"
status: accepted
---

## 背景

`core/src/config.rs` の `api_key` / `client_secret` / `refresh_token` は
`config.toml` へ平文保存され、`desktop/src/commands/config.rs` の `get_config`
が `AppConfig` を丸ごと JS へ返していた（todo.md D05、code-scanner #2、
secret-auditor の精査）。UI 未接続のため顕在化はしていないが潜在リスク。

## 決定

- **保存形式は維持する**。`skip_serializing` は今回使わない。
  キーチェーン未実装で環境変数経由のキー注入も figment の `split("_")` では
  効かず、機能回帰するため。
- `desktop/src/commands/config.rs` に許可リスト型 DTO
  （`ConfigView` / `AiConfigView` / `OpenAiConfigView` /
  `StorageConfigView` / `GoogleDriveConfigView`）を導入する。
  - 秘密 3 フィールドは JSON キーごと消え、`has_api_key` /
    `has_client_secret` / `has_refresh_token` の bool のみを返す。
  - `get_config` / `reset_config` の戻り値を DTO に変更。
  - `merge_secrets(current, incoming)` を追加し、`update_config` /
    `update_ai_config` が None / 空文字 / `"***"` を受け取ったとき
    保持中の実値をマージする（保存のたびにキーが消えるのを防ぐ）。
- `OpenAiConfig` / `GoogleDriveConfig` の `Debug` を手実装し、
  秘密フィールドを `"***"` にマスクする。
- `AppConfig::save()` は UNIX 系で 0600 権限で書き込む
  （新規作成・既存上書きの両方）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| IPC マスク＋Debug＋権限（採用） | 機能を維持しつつ露出面を減らせる |
| `skip_serializing` | 再起動後にキーが復元できず機能回帰する |
| OS キーチェーン移行 | 新規依存と設計判断が必要で今回は見送り |

## 影響

- Tauri IPC の `get_config` レスポンスに秘密値が含まれなくなる。
- 将来 UI を接続するとき、`has_*` フラグを見て送信内容を組み立てる必要がある。
- `reset_config` は従来どおりデフォルトを保存するため、キーは消える。
- 新規依存は無い。

## 検証

- 追加テスト 8 件（core 3 / desktop 5）。
  - DTO JSON に秘密キー名・ダミー値が含まれないこと。
  - 秘密フィールド省略 TOML のデシリアライズ互換。
  - 0600 作成と 0644 からの引き下げ。
  - sentinel（None / 空 / `"***"`）のマージ復元と明示的新値の保持。
- `cargo test --all-targets --locked`: core 134 + desktop 7、すべて成功。
- `./scripts/verify.sh` を実行し、6 ゲート・trunk build・WASM パス検査・
  E2E 228 項目がすべて成功した。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- OS キーチェーン移行（`api_key` / `client_secret` / `refresh_token`）。
- ~~`extra_headers` に認証情報が入り得る問題~~ ADR-0102 で解決済み
  （DTO から削除し、Debug でマスク）。
- `desktop/src/commands/document.rs` の `tokio::fs` 直呼び
  （fs-bypass-auditor の指摘。既定 `local_roots` が空で backend 経由に
  置換すると回帰するため保留、ADR-0097 から継続）。
- `IntersectionObserver` による画像の遅延読み込み（ADR-0034 から継続）。
EOF
