---
type: Decision
title: "ADR-0100: AI サービスの有効性ガードと破棄"
description: "AI を無効化した後に service が残存して送信できる問題を修正する決定を記録する。"
tags:
  - ADR
  - security
  - ai
  - desktop
generated:
  by: dsh
  at: "2026-11-18T02:50:00+09:00"
status: accepted
---

## 背景

`AppState::ai_service()` は `config.ai.enabled` を確認せず、service の有無だけを
見ていた。設定を false に戻しても `ai_service` が None に戻される箇所が無く、
一度有効化された後は設定上無効でも AI コマンドが送信できる状態だった
（ai-consent-auditor の経路 B/C、ADR-0099 の未決定）。

## 決定

- `AppState::ai_service()` は `is_ai_enabled()` を先に確認し、
  `config.ai.enabled == false` なら None を返す。
- `AppState::disable_ai_service()` を追加し、`ai_service` を None に戻す。
- `update_config` / `update_ai_config(enabled=false)` / `reset_config` のとき
  `disable_ai_service()` を呼ぶ。
- AI コマンドのシグネチャと Tauri 登録は変更しない
  （`ai_service()` 側のガードで全コマンドに適用される）。
- RwLock は `config` の取得・解放後に `ai_service` を取得する二段構えとし、
  同時保持によるデッドロックを避ける。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| `ai_service()` をガード（採用） | 1 箇所の変更で全 AI コマンドに効く |
| 各 AI コマンドにチェックを追加 | 6 箇所の重複となり漏れやすい |
| 設定変更時だけ破棄 | 既存 service の参照が残る可能性がある |

## 影響

- 設定上 AI が無効なら、どの AI コマンドも「AI service not initialized」を返す。
- AI を false に戻すと service が破棄され、再送信には明示的な再有効化が必要。
- 新規テスト 3 件を追加。

## 検証

- `cargo test --all-targets --locked`: core 136 + desktop 10、すべて成功。
- `./scripts/verify.sh` を実行し、6 ゲート・trunk build・WASM パス検査・
  E2E 228 項目がすべて成功した。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 解決済み

- `base_url` を JS から任意変更できる問題は ADR-0101 で許可リスト方式に制限済み。
- `extra_headers` を JS から任意変更できる問題は ADR-0102 で DTO から除外・マスク済み。
- 送信プレビュー＋ネイティブの同意ダイアログは ADR-0104 で実装済み。

## 未決定

- `IntersectionObserver` による画像の遅延読み込み（ADR-0034 から継続）。
EOF
