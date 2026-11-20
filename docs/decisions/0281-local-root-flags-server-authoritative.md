---
type: Decision
title: "ADR-0281: local root flags の server-authoritative 化"
description: "設定 DTO で read_only / watch が writable でも、マージ時に現在値を復元して権限昇格を防ぐ決定を記録する。"
tags:
  - ADR
  - config
  - security
generated:
  by: dsh
  at: "2026-11-21T04:05:00+09:00"
status: accepted
---

## 背景

`LocalRootView` は `read_only` / `watch` / 配列順をクライアント writable で公開する。
`merge_secrets()` は id 一意一致時に `path` だけを現在値から復元していたため、
IPC を呼べるスクリプトが `read_only=false` を送ると、隠された実パスを持つ root が
writable へ昇格し得た。incoming 側の id 重複も検査されていなかった。

## 決定

- `merge_secrets()` で id 一意一致時に
  `path` に加えて `read_only` / `watch` も現在値を採用する。
- 空 id / 不一致 / current 側で重複 / incoming 側で重複した group は
  `path` を空にし、flags も現在値を採用しない（新規 root として扱う）。
- `read_only` / `watch` は server-authoritative とし、
  設定 DTO は往復互換のため残すが UI 編集対象にしない。
- E2E check 名・件数（256）は変更しない。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| id 一致時に flags も復元（採用） | 権限昇格を防ぎつつ DTO 互換を維持できる |
| flags を DTO から削除 | 既存設定の往復互換を壊す |
| クライアント値を信頼 | read-only root が writable へ昇格する |

## 影響

- JS が `read_only=false` / `watch=true` を送っても、既存 id の root では無視される。
- 新規 id の root は incoming の flags を維持するが、
  path は空のため実パスを持たない。
- 非一意 id の root は空 path で保存され、
  アクセス時の resolve で拒否される（可用性側の残リスク）。

## 検証（Runtime Proof）

- `cargo test -p openrustbooks-desktop --locked merge_secrets`:
  **16 passed / 0 failed**。
- 新規 test:
  - `merge_secrets_restores_local_root_flags_by_id`
  - `merge_secrets_ignores_duplicate_incoming_local_root_ids`
  - `merge_secrets_skips_current_side_duplicate_local_root_flags`
  - `merge_secrets_does_not_inherit_flags_for_new_local_root_ids`
  - `merge_secrets_preserves_local_root_order_and_name`
- `./scripts/verify.sh`: 全ゲート成功、E2E **256 passed / 0 failed**。

## 未決定

- 非一意 id の root を保存前に拒否するか（現状は空 path で保存）。
- flags 変更専用 command を設けるか。
- 実 Tauri WebView で DTO 往復が同じく fail-closed であることの確認。
