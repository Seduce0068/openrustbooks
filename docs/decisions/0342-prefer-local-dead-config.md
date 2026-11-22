---
type: Decision
title: "ADR-0342: prefer_local の dead config 化"
description: "未配線の ai.prefer_local を DTO / UI から除外し、config.toml の値だけを保持する決定。"
tags:
  - ADR
  - config
  - DTO
generated:
  by: dsh
  at: "2026-11-23T03:10:00+09:00"
status: accepted
---

## 背景

`ai.prefer_local` は core 設定・desktop DTO・web DTO・設定 UI に露出していたが、
provider 選択は `default_provider` のみを参照し、値は一切効いていなかった。
ADR-0304 の dead config 方針に含まれていなかったため、利用者は効かない設定を操作できた。

## 決定

- core の `AppConfig.ai.prefer_local` と config.toml 読み書きは維持する。
- desktop `AiConfigView` / `ConfigPatch` と web `SettingsAiView` からフィールドを削除する。
- `merge_secrets()` が `current.ai.prefer_local` を復元し、
  JS 側から送られた値は受け付けない。
- 設定画面のチェックボックスを `<p class="settings-notice">"ローカル優先: 未配線"</p>` に置換する。
- `check-config-dto-contract.sh` に `EXCLUDED_FROM_BOTH = {"prefer_local"}` を追加し、
  どちら側への再登場も fail-closed で拒否する。
- E2E mock_config / fixture / requirements から dead キー・記述を削除するが、
  check 名と `Checker(expected=263)` は変更しない。

## 検証（Runtime Proof）

- `cargo test --all-targets --locked`: core 357 / desktop 187 / web 31 = **575 passed / 0 failed**。
- `cargo fmt --all -- --check`: rc=0。
- `cargo check --all-targets --locked`: rc=0。
- `cargo clippy --all-targets --locked -- -D warnings`: rc=0。
- wasm core / web check、`npx tsc --noEmit`: rc=0。
- check-config-dto-contract / check-e2e-count-consistency: rc=0。
- `./scripts/verify.sh`: 全ゲート成功、E2E **263 passed / 0 failed**、WDIO gate rc=0。

## 未決定

- provider 選択へ `prefer_local` を実際に配線するか（実装する場合は本 ADR を置き換える）。
