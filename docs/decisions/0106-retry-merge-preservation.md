---
type: Decision
title: "ADR-0106: リトライ設定のマージ保持"
description: "max_retries を DTO から除外し、IPC 往復で現在値を保持する決定を記録する。"
tags:
  - ADR
  - config
  - desktop
generated:
  by: dsh
  at: "2026-11-18T04:00:00+09:00"
status: accepted
---

## 背景

ADR-0105 で `AiConfig.max_retries` を追加した際、最初は `AiConfigView` にも
含めていた。しかし古い DTO JSON（キーなし）を `update_config` へ渡すと
serde の既定値 3 で保存され、利用者の設定（例 5）が静かに消える問題が
発見された（ADR-0105 の未決定、retry-impact-auditor の指摘）。

## 決定

- `AiConfigView` から `max_retries` を削除し、DTO JSON に出さない。
- `merge_secrets()` で `ai.max_retries` を現在値から保持する。
- 既存テストを「DTO に含まれない」検証に変更し、
  マージ保持のテストを追加する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| DTO 除外＋マージ保持（採用） | ADR-0098 / 0102 / 0103 と同じ方針で安全 |
| `serde(default)` だけ | キー欠落時に既定値で上書きされる |
| DTO から削除しない | 古い JSON 往路で値が消える |

## 影響

- IPC の `get_config` レスポンスに `max_retries` が出なくなる
  （現状 UI 未接続のため即時の影響なし）。
- 保存済み `max_retries` は設定更新の往復で保持される。
- 新規依存は無い。

## 検証

- 既存テストを変更し、新規テスト 1 件を追加。
- `cargo test --all-targets --locked`: core 151 + desktop 23、すべて成功。
- `./scripts/verify.sh` を実行し、6 ゲート・trunk build・WASM パス検査・
  E2E 228 項目がすべて成功した。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- service-refresh-auditor の指摘: `update_config` が AI 設定変更時に
  service を再初期化しないため、保存設定と実行中 service が乖離する。
  差分トリガー／fail-closed／async 化の設計判断が必要（別タスク）。
- `update_ai_config` の再初期化エラーが `let _ =` で握り潰される。
- REQ-010 の「最大 3 回再生成」と `0..=max_retries` の解釈差（ADR-0105 から継続）。
EOF
