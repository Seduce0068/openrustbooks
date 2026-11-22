---
type: Decision
title: "ADR-0344: レガシー config command の登録解除"
description: "UI 起点 CAS を満たさない 4 つの config command を invoke_handler から外し、 patch / API キー / clear 経路へ集約する決定。"
tags:
  - ADR
  - desktop
  - IPC
generated:
  by: dsh
  at: "2026-11-23T03:30:00+09:00"
status: accepted
---

## 背景

`update_config` / `update_ai_config` / `update_lint_config` / `reset_config` は
内部で `config_with_revision()` の revision を expected として使い、
UI が `get_config` で読んだ revision を無視した。
ADR-0269 の UI 起点 CAS 契約と不均一で、web / e2e / scripts からは未参照だった。

## 決定

- 上記 4 command を `invoke_handler` から削除し、本体・専用 helper・専用 test も削除する。
- 残す config command:
  - `get_config`（revision 供給）
  - `update_config_patch`（`patch.expected` を CAS に使用）
  - `set_openai_api_key`（引数 `expected` を CAS に使用）
  - `clear_secrets`
- `clear_secrets` は現在値に対する targeted delete であり、
  古い view を丸ごと上書きしないため UI 起点 CAS の対象外とする。
  内部 revision を使う例外として本 ADR に記録する。
- `load_failure_message` を削除し、`clear_secrets` は
  `state.inner().config_load_failed()` を直接判定して固定文言を返す。

## 検証（Runtime Proof）

- desktop test: **185 passed / 0 failed**。
- `cargo fmt --all -- --check`: rc=0。
- `cargo check --all-targets --locked`: rc=0。
- `cargo clippy --all-targets --locked -- -D warnings`: rc=0。
- `./scripts/verify.sh`: 全ゲート成功、E2E **263 passed / 0 failed**、WDIO gate rc=0。

## 未決定

- `clear_secrets` も `expected` 必須化して `CommandError` を返すか。
- test 名 `update_config_normalization_drops_js_injected_hidden_fields` の改称（機能影響なし）。
