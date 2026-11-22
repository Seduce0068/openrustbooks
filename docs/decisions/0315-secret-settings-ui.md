---
type: Decision
title: "ADR-0315: desktop 設定画面の API キー設定と削除"
description: "既存 redaction 契約を維持したまま、desktop のみ API キーを設定・削除できる UI を接続する決定。"
tags:
  - ADR
  - desktop
  - security
generated:
  by: dsh
  at: "2026-11-22T11:42:00+09:00"
status: accepted
---

## 背景

API Key 欄は常に readonly で `has_api_key` 表示のみだった。
`update_ai_config` / `clear_secrets` command は存在したが web から呼ばれず、
初回インストール後に UI から AI を使い始められなかった。

## 決定

- desktop に専用 command `set_openai_api_key(expected, api_key)` を追加する。
- 入力は CAS（UI が読んだ revision）でコミットし、
  空文字は明示的な未設定（None）として扱う。
- `clear_secrets` を `targets.api_key=true`、他秘密は false で呼ぶ UI を追加する。
- API キーは command 引数としてのみ渡し、
  ConfigView / status / log / DOM の永続表示へ残さない。
- 入力欄は `type=password`、呼び出し後は成否にかかわらず DOM 値をクリアする。
- ブラウザでは `editable=false` により readonly / disabled を維持する。
- capability scope 静的ゲートの許可 command に 2 件を追加する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 専用 command + UI（採用） | 完全な AiConfig DTO を JS へ渡さずに済む |
| update_ai_config をそのまま接続 | web DTO に無いフィールドを扱う必要があり危険 |
| 値を ConfigView に含める | redaction 契約に反するため却下 |

## 検証（Runtime Proof）

- desktop test: **149 passed / 0 failed**。
- web test: 9 passed / 0 failed。
- `cargo fmt --all -- --check` / `cargo clippy --all-targets --locked -- -D warnings`: rc=0。
- wasm core / web check / `npx tsc --noEmit`: rc=0。
- `npm run test:wdio`: 1 passing。
- `./scripts/verify.sh`: 全ゲート成功、E2E **256 passed / 0 failed**、WDIO gate rc=0。

## 未決定

- client_secret / refresh_token の UI 編集。
- 削除失敗時も DOM 入力をクリアするか（現状は成功時のみ）。
