---
type: Decision
title: "ADR-0319: 確認ダイアログ後の AI service 再取得"
description: "AI command が confirm 待ち中に無効化された古い service を使わないようにする決定。"
tags:
  - ADR
  - desktop
  - AI
generated:
  by: dsh
  at: "2026-11-22T13:22:00+09:00"
status: accepted
---

## 背景

generate / refine / convert_to_table / generate_front_matter は
confirm_send().await の前に ai_service() を取得していた。
ダイアログ表示中に commit_config が AI を無効化／provider を変更しても、
古い service で外部送信される可能性があった。

## 決定

- 4 コマンドすべてで `state.ai_service()` を confirm_send().await の後に取得する。
- service が None なら送信前に固定 IPC 文言で拒否する。
  - 現行 IPC 文言は `Internal configuration error`（`CONFIG_INTERNAL`）。
- parse_table_format は confirm 前、parse_table_alignment は confirm 後の
  既存順序を維持する。
- ダイアログ文言 / 既存 AI 成功契約 / lint command は維持する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| confirm 後再取得（採用） | 既存 UI 契約を変えずに陳腐化を防げる |
| confirm 前に service を固定 | 無効化後の送信を許す |
| AI_PIPELINE code を新設 | 既存の他エラー文言も変わるため今回は見送り |

## 検証（Runtime Proof）

- 単体 test: confirm フック内で disable_ai_service を呼び、
  4 コマンドすべてが pipeline 到達前に拒否されることを確認。
- desktop test: **158 passed / 0 failed**。
- `cargo fmt --all -- --check`: rc=0。
- `cargo check --all-targets --locked`: rc=0。
- `cargo clippy --all-targets --locked -- -D warnings`: rc=0。
- `./scripts/verify.sh`: 全ゲート成功、E2E **256 passed / 0 failed**、WDIO gate rc=0。

## 未決定

- `AI_PIPELINE` code と専用 IPC 文言の導入（影響範囲が広いため別タスク）。
- config.ai.enabled 変更経路の専用 test。
- 静的 TEST_CONFIRM_HOOK のスレッドローカル化。
