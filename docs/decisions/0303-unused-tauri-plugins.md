---
type: Decision
title: "ADR-0303: 未使用 Tauri plugin / 依存の削除"
description: "store / global-shortcut / shell / fs / opener / notification と未使用 crate を削除し、静的ゲートを log + dialog へ絞る決定。"
tags:
  - ADR
  - desktop
  - dependencies
generated:
  by: dsh
  at: "2026-11-22T07:27:00+09:00"
status: accepted
---

## 背景

desktop は store / global-shortcut / shell / fs / opener / notification を
登録のみで利用していなかった。
anyhow / tracing / tracing-subscriber / figment / thiserror / futures / objc
もコード上の使用が無い。
ADR-0280 は fs / dialog / shell の初期化が残る前提で書かれていた。

## 決定

- 未使用 6 plugin の登録と直接依存を削除する。
- 未使用 7 crate の直接依存を削除する。
- log と dialog は維持する（logger / 確認ダイアログで実利用）。
- wdio plugin は debug + feature のまま維持する。
- `check-tauri-plugin-config.sh` の必須 init を log / dialog のみへ更新し、
  削除 plugin の main.rs への再混入を fail-closed で検出する。
- ADR-0280 の fs / shell 初期化に関する記述は本 ADR が supersede する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 未使用 plugin / 依存を削除（採用） | 攻撃面と保守コストを減らせる |
| 登録のみ残す | 不要な初期化と依存が残る |
| dialog も削除 | AI 送信確認ダイアログで利用するため却下 |

## 検証（Runtime Proof）

- `cargo check -p openrustbooks-desktop --features wdio --locked`: rc=0。
- 静的ゲート正方向 rc=0 / 負向 5 ケースすべて期待どおり。
- `cargo check / test / clippy`: rc=0。
- `./scripts/verify.sh`: 全ゲート成功、E2E **256 passed / 0 failed**、WDIO gate rc=0。
- capability に削除 plugin 権限は無く、変更不要。

## 未決定

- desktop/src/window/mod.rs の global-shortcut 古いコメント削除。
- Cargo.toml への削除 plugin 再混入を静的ゲートで検出するか。
- workspace の未使用依存 warning（`uuid` 等）の整理。
