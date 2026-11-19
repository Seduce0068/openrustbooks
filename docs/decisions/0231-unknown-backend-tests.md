---
type: Decision
title: "ADR-0231: 未検出バックエンド / provider のテスト"
description: "未検出ストレージバックエンドと未知 AI provider のエラー分岐を unit test で固定する決定を記録する。"
tags:
  - ADR
  - testing
  - core
generated:
  by: dsh
  at: "2026-11-19T17:31:00+09:00"
status: accepted
---

## 背景

設定・ストレージ・AI 主要モジュールのテストは core 267 件まで拡充済みだったが、
次の 2 経路に専用 test が無かった（R6 R4-unit#3 残）:
- `StorageManager::set_default_backend()` の未検出 backend
  （`core/src/io.rs:232`）
- `create_client()` の未知 provider
  （`core/src/ai_markdown/client.rs:159`）

## 決定

- `io.rs` の tests に最小 `KnownBackend` stub を追加し、
  登録済み id の成功経路と `"missing"` のエラー経路を検証する。
- エラーは `CoreError::Storage` で
  `Storage error: Backend not found: missing` 完全一致を固定する。
  失敗後も既定 backend が不変であることを確認する。
- `client.rs` の tests に未知 provider の test を追加する。
  `default_provider = "unknown-provider"` と
  ダミーの `api_key = "sk-SHOULD_NOT_LEAK_API_KEY"` を設定し、
  `CoreError::AiProvider` と固定文言 "Unknown provider" を確認する。
- API キー／秘密マーカーがエラー文言へ出ないことを検証する。
- 本番コードは変更しない（test のみ）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 最小 stub でエラー分岐を直接テスト（採用） | ネットワーク／FS なしで決定的に検証できる |
| `LocalStorage` を再利用 | id `"local"` 固定で未検出 id の testing に不向き |
| 本番コードの文言を固定化 | 誤用 id / provider のエコー契約を変えるため本次対象外 |

## 影響

- core test が **267 → 270**（+3）。
- `Arc<dyn AiClient>` は `Debug` を実装しないため、
  既存の `.err().expect()` パターンを採用。
- 任意の id / provider 文字列はエラーへエコーされる
  （現状契約、未決定として記録）。
- E2E 254 維持。

## 検証（Runtime Proof）

- `cargo test -p openrustbooks-core --locked`: **270 passed / 0 failed**。
- `cargo clippy -p openrustbooks-core --all-targets --locked -- -D warnings`、
  wasm32 core check、`./scripts/verify.sh`: すべて成功。
- E2E **254 項目すべて成功**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- `{id}` / `{provider}` のエコー契約を維持するか、
  固定文言へ変更するか（任意入力にパス／秘密が渡された場合の責任分界）。
- id / provider に許容する値の契約（安全な識別子／列挙値）。
- `create_client("openai")` 成功経路の unit test 追加要否
  （現状は `OpenAiClient::new` 直接 test に留まる）。
