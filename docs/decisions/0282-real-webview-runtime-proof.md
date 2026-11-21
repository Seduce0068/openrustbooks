---
type: Decision
title: "ADR-0282: 実 WebView 起動と設定保存の Runtime Proof"
description: "Tauri CLI 導入後に実 WebView で起動ブロッカーと保存ブロッカーを修正し、設定保存を検証した決定を記録する。"
tags:
  - ADR
  - tauri
  - dogfood
generated:
  by: dsh
  at: "2026-11-21T15:45:00+09:00"
status: accepted
---

## 背景

Tauri CLI 2.12.5 を導入後、`cargo tauri dev` の実起動で次が判明した。

- `core::init()` が native で `tracing_subscriber` を導入し、
  `tauri_plugin_log` と競合して
  "attempted to set a logger after the logging system was already initialized"
  で起動失敗した。
- `beforeDevCommand` の `cd ../web` が Tauri CLI フックの cwd と合わず失敗した。
- `tauri_plugin_updater` 2.14.1 は `plugins.updater.pubkey` を必須とし、
  設定が無いと "Error deserializing 'plugins.updater'" で起動失敗した。
- 実 WebView から `update_config_patch` を呼ぶと、
  `invalid args 'patch'` で reject された。
  web 側が引数を `{ patch: ... }` ではなく flat に渡していた。

## 決定

- desktop では `core::init()` を呼ばず、`tauri_plugin_log` を唯一の logger とする。
  wasm 側の `init()` は維持する。
- `beforeDevCommand` / `beforeBuildCommand` を `cd web && ...` に変更する。
- 未配線の `tauri_plugin_updater` を登録から外し、`Cargo.toml` / `Cargo.lock` からも削除する。
  公開鍵 / endpoint / 署名は導入しない。
- `check-updater-honesty.sh` は updater 登録が無いことを必須とする。
- `update_config_patch` の引数を `{ patch: ... }` に変更する。
- Tauri の reject が JSON 文字列の場合も `code` を抽出する。
- E2E mock は `args.patch.expected` を必須とし、flat 引数を许容しない。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 実 WebView で修正し Runtime Proof を残す（採用） | Tauri mock では検出できない不具合を発見できる |
| mock E2E 256/0 をもって完了とする | 実際には保存が失敗していたため却下 |
| updater にダミー pubkey を設定 | ADR-0222 の未配線契約と矛盾するため却下 |

## 影響

- desktop の `tracing::debug!` は既定では出力されない。
  `log::` マクロは `tauri_plugin_log` の Stdout / LogDir / Webview へ出力される。
- `RUST_LOG` による tracing filter は desktop では無効になる。
- `createUpdaterArtifacts: true` は残置する（成果物フラグのみ）。
- E2E check 名・件数（256）は変更しない。

## 検証（Runtime Proof）

- `cargo tauri --version`: `tauri-cli 2.12.5`、rc=0。
- `GDK_BACKEND=x11 cargo tauri dev`: WebView ウィンドウ出現を確認。
- Wayland 側 `kdotool search --name openrustbooks`: window id を確認。
- X11 backend で `/settings` 表示:
  - `Revision: 0`
  - editable フォーム
  - 「更新確認: 未配線（updater は実装されていません）」
- 実 WebView の一時 IPC probe:
  - `window.__TAURI__` / `core.invoke` が存在
  - `get_config` 成功、`revision=0`、`log_level=info`
  - `update_config_patch` 成功、`revision=1`、`log_level=debug`
  - `config.toml` に `log_level = "debug"` が書き込まれた
- 検証後、config.toml をバックアップから復元し、probe と一時 devUrl を削除。
- `./scripts/verify.sh`: 全ゲート成功、E2E **256 passed / 0 failed**。

## 未決定

- desktop の tracing イベントを `tauri_plugin_log` へ橋渡しするか、
  `log::` へ統一するか。
- 実スクリーンリーダーによる読み上げ確認。
- updater を正式に導入する場合の公開鍵 / endpoint / 署名設計。
- `createUpdaterArtifacts: true` の残置をゲートで固定するか。
