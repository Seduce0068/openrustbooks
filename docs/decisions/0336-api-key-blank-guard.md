---
type: Decision
title: "ADR-0336: API キー空入力の no-op 化"
description: "空／空白のみの API キー入力を保存 command へ送らず、設定済みキーを誤って削除しないようにする決定。"
tags:
  - ADR
  - web
  - security
generated:
  by: dsh
  at: "2026-11-23T02:00:00+09:00"
status: accepted
---

## 背景

API キー欄の placeholder は「設定済みの場合は変更不要」だったが、
空のまま保存ボタン／Enter を押すと `set_api_key` が呼ばれ、
desktop 側で空文字が None として扱われ設定済みキーが消えた。

## 決定

- `start_api_key_save` で `api_key.trim().is_empty()` なら command を呼ばない。
- DOM 入力はクリアし、status に固定文言
  「入力がありません。変更はありません。」を設定する。
- `saving` は立てず、既存の成功／失敗文言は維持する。
- 削除は専用ボタン（`clear_secrets`）で行い、今回は確認ダイアログを追加しない。

## 検証（Runtime Proof）

- web test: **31 passed / 0 failed**（空／空白／非空の3件を追加）。
- `cargo fmt --all -- --check`: rc=0。
- `cargo check -p openrustbooks-web --target wasm32-unknown-unknown --locked`: rc=0。
- `npx tsc --noEmit`: rc=0。
- check-e2e-count-consistency: expected=263 / 実数 263。
- `./scripts/verify.sh`: 全ゲート成功、E2E **263 passed / 0 failed**、WDIO gate rc=0。

## 未決定

- 削除ボタンの確認ダイアログ（E2E 契約と既存 dialog 構造の都合で今回は据え置き）。
- command 未呼び出し自体の E2E 固定（現状は pure helper と status 文言で担保）。
