---
type: Decision
title: "ADR-0099: AI 機能の既定無効化"
description: "AI 機能を既定で無効にし、明示的な opt-in まで本文を送信しないようにする決定を記録する。"
tags:
  - ADR
  - security
  - ai
generated:
  by: dsh
  at: "2026-11-18T02:35:00+09:00"
status: accepted
---

## 背景

`core/src/config.rs` の `AiConfig::default()` は `enabled: true` で、
`desktop/src/main.rs` の setup で起動時に `init_ai_service()` が呼ばれていた。
UI 未接続のため実際の送信は起きていないが、機能を接続した瞬間に
同意なしで本文が送られるリスクがあった（todo.md D06）。

## 決定

- `AiConfig::default().enabled` を `false` に変更する。
- フィールドの doc コメントを「既定では無効、明示的な opt-in まで送信しない」に更新する。
- 既存設定ファイル／環境変数で明示的に `true` が設定されていれば従来どおり有効。
- 起動時の `init_ai_service()` は `is_ai_enabled()` ガードのまま（既定では呼ばれない）。
- 送信プレビュー／同意ダイアログの UI 実装は今回行わない。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 既定 false（採用） | 何も設定しなければ送信経路が生成されない |
| 既定 true のまま UI で警告 | UI 未接続の現状では無防備 |
| AI 機能を削除 | 要件 REQ-010 と矛盾する |

## 影響

- 新規インストールでは AI 機能が無効。
- 既存ユーザーで明示的に有効化している場合は従来どおり動く。
- 新規テスト 2 件（既定 false、TOML で明示 true が復元）を追加。
- `desktop/src/state/mod.rs` の設定往復テストを新既定値に更新。

## 検証

- `cargo test --all-targets --locked`: core 136 + desktop 7、すべて成功。
- E2E 228 項目: 3 回連続成功（3 回中 2 回はダウンロード関連の
  一時的なフレークが発生したが、再実行で解消。コード起因ではない）。
- `./scripts/verify.sh` の 6 ゲート・trunk build・WASM パス検査は成功。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 解決済み

- 送信時に `enabled` が再チェックされず、生成済み `ai_service` が残存する問題
  （経路 C）は ADR-0100 で解決済み。
- `update_ai_config` から JS が `enabled=true` と任意の `base_url` を設定できる
  問題（経路 B）は ADR-0101 で送信先を許可リストに制限済み。
- 送信プレビュー＋ネイティブの同意ダイアログは ADR-0104 で実装済み。

## 未決定（ai-consent-auditor の指摘）

- ~~wasm 側の `ai_enabled: true` ハードコード~~ ADR-0117 で既定 false に修正済み。
EOF
