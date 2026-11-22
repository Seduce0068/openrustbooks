---
type: Decision
title: "ADR-0328: reader 表示設定の初期値と権威"
description: "config.toml の reader 設定は初回のみ localStorage の初期値として使い、以後はブラウザ設定を権威とする決定。"
tags:
  - ADR
  - web
  - settings
generated:
  by: dsh
  at: "2026-11-22T23:51:00+09:00"
status: accepted
---

## 背景

設定画面の reader / editor 値を保存しても描画へ反映されず、
config.toml と localStorage のどちらが効くか分からなかった。
ADR-0217 は未接続を開示しつつ配線を未決定としていた。

## 決定

- Reader の `font_size` / `line_height` のみを配線する。
- localStorage に `SETTINGS_KEY` が存在しない初回のみ、
  config の値を最も近い FONT_SCALES / LINE_HEIGHTS の段階へ変換して保存する。
- 既存の表示設定がある場合は一切上書きしない。
- 以後はブラウザの表示設定が権威で、反映はリロード / 開き直し後に行われる。
- Editor の `tab_size` / `word_wrap` は未配線のままとし、
  設定画面に未配線 notice を表示する。
- 初回保存に失敗した場合は専用固定文言を status へ出す。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 初回のみ config を初期値化（採用） | 二重権威による上書きを防げる |
| 保存のたびに config を適用 | ユーザーの表示設定を失う |
| editor も配線 | DOM / キーボード E2E への影響が大きく今回は見送り |

## 検証（Runtime Proof）

- web test: **26 passed / 0 failed**（既存設定非上書き / 初回適用の2件を追加）。
- `cargo fmt --all -- --check`: rc=0。
- `cargo check -p openrustbooks-web --target wasm32-unknown-unknown --locked`: rc=0。
- `npx tsc --noEmit`: rc=0。
- check-e2e-count-consistency / check-config-dto-contract: rc=0。
- `./scripts/verify.sh`: 全ゲート成功、E2E **256 passed / 0 failed**、WDIO gate rc=0。

## 未決定

- Editor の tab_size / word_wrap の配線（localStorage 新キーと Tab キー挙動の変更が必要）。
- init 失敗時に signal を直接更新して即時描画する仕組み。
