---
type: Decision
title: "ADR-0287: IPC エラーの redaction 統一"
description: "desktop command の生エラー文字列を廃し、固定 code と固定文言だけを IPC へ返す決定。"
tags:
  - ADR
  - security
  - IPC
generated:
  by: dsh
  at: "2026-11-22T02:55:00+09:00"
status: accepted
---

## 背景

legacy config / AI / fs / document / window command は
`CoreError::to_string()` をそのまま Tauri IPC へ返していた。
CoreError の Display はパス / provider host / response body / secret を
含み得るため、描画側へ機微情報が漏れる経路が残っていた。

## 決定

- `desktop/src/commands/mod.rs` に `ipc_message(code)` / `ipc_error(CoreError)` を追加する。
- 全 desktop command の error 変換を共通マッパーへ統一する。
- `CoreError::code()` に `AiProvider` / `OpenAiError` → `AI_PROVIDER` を追加する。
- desktop 独自の window error は `WINDOW_INTERNAL` の固定文言へ落とす。
- 未知 code は code 文字列自体を返さず既定文言へ落とす。
- `CommandError` の JSON 契約（code + message）は既存互換を維持する。
- web の status 文言に `AI_PROVIDER` を追加する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| IPC 境界で固定文言へ変換（採用） | CoreError の内部情報は保持しつつ外部露出を止められる |
| CoreError::Display 自体を変更 | 内部ログ / test 契約を壊すため却下 |
| 各 command で個別変換 | 漏れが再発しやすいため却下 |

## 影響

- CoreError::Display は内部向けに従来どおり生値を保持する。
- IPC へは固定文言のみが出る。
- `CONFIG_LOAD_FAILED` / `AI_PROVIDER` / `WINDOW_INTERNAL` の
  固定文言が追加される。
- E2E check 名・件数・settings の成功文言は不変。

## 検証（Runtime Proof）

- `cargo test --all-targets --locked`:
  core 304 / desktop 112 / web 4 = **420 passed / 0 failed**。
- `cargo fmt --all -- --check`: rc=0。
- `cargo clippy --all-targets --locked -- -D warnings`: rc=0。
- `cargo check -p openrustbooks-core --target wasm32-unknown-unknown --locked`: rc=0。
- `cargo check -p openrustbooks-web --target wasm32-unknown-unknown --locked`: rc=0。
- `./scripts/verify.sh`: 全ゲート成功、E2E **256 passed / 0 failed**、WDIO gate rc=0。

## 未決定

- Tauri event / log target の Webview 出力に code 以外の情報を出さない運用の明文化。
- fs / config の CoreError 内部文字列に残るパス表記の完全削除。
