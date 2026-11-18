---
type: Decision
title: "ADR-0128: 設定フォームの読み取り表示と保存見送り"
description: "SettingsForm に Tauri IPC の読み取り表示とブラウザ版 fallback を追加し、保存接続は安全性の観点から見送る決定を記録する。"
tags:
  - ADR
  - web
  - desktop
  - ui
generated:
  by: dsh
  at: "2026-11-18T09:35:00+09:00"
status: accepted
---

## 背景

ADR-0127 の第 1 段として SettingsForm と `get_config` / `update_config` の
接続を検討した。しかし監査で次が判明した。

- `get_config` は `ConfigView`（secret / パス除外 DTO）を返すが、
  `update_config` は完全な `AppConfig` を要求する。
- DTO をそのまま送ると serde デシリアライズ失敗。
- Web で完全な `AppConfig` を再構成すると、隠しフィールド
  （`data_dir` / `api_key` / `extra_headers` / `local_roots[].path` 等）が
  既定値で上書きされるリスクがある（`merge_secrets` は明示リストのみ復元）。
- `withGlobalTauri: true` は CSP 強化とセットで検討すべきで、
  単独有効化は攻撃面を広げる。

## 決定

- `web/src/lib.rs` に Tauri invoke の薄いラッパを追加する（下準備）。
  - `window.__TAURI` → `core` → `core.invoke` を `js_sys::Reflect` で取得。
  - 欠落／型不一致なら `Err("desktop only")`（panic しない）。
- SettingsForm は**読み取り表示のみ**を実装する。
  - `get_config` 成功時に DTO をシグナルへ格納し、
    `prop:value` / `prop:checked` で表示。
  - API Key は実値を signal / DOM に書かず、`has_api_key` を
    「設定済み／未設定」と表示。入力欄は空のまま。
  - ブラウザ単体では「デスクトップ版のみ利用可能」と表示（panic なし）。
  - Save Settings は `disabled` で「保存はデスクトップ版のみ
    （パス検証と CSP 強化後に接続）」。
- `desktop/tauri.conf.json` は変更しない（`withGlobalTauri` は見送り）。
- E2E に 1 件追加（`/settings` の fallback 表示と JS エラー 0 件）、
  期待総数 228 → 229。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 読み取りのみ（採用） | 秘密と隠しフィールドを守りつつ UI の土台ができる |
| DTO をそのまま update_config へ | serde 失敗で保存できない |
| 完全 AppConfig を Web で再構成 | 隠しフィールド上書きリスク |
| withGlobalTauri を単独追加 | CSP 強化とセットでないと攻撃面が残る |

## 影響

- 実デスクトップでも `withGlobalTauri` が無いため、現状は読み取りも
  動かず fallback 表示になる（下準備のみ）。
- 新規依存・unsafe なし。
- 既存の IME／入力欄フォーカス保护は維持。

## 検証

- 追加テスト 1 件（E2E）、期待総数 229。
- `cargo test --all-targets --locked`: core 186 + desktop 28 + web 2、
  wasm-test 3、audit 脆弱性 0 件、E2E 229 項目すべて成功。
- `./scripts/verify.sh` rc=0。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定（settings-wiring-auditor の指摘）

- 保存接続の設計: patch DTO（`ConfigUpdateRequest`）を Rust 側で受け、
  `current` へマージする方式が望ましい（desktop/src の変更が必要）。
- `withGlobalTauri` と CSP 強化（`'wasm-unsafe-eval'`、nonce/hash 化、
  外部フォントのローカル化）の同時実施。
- Rust 側パス検証（todo.md 最優先）と A05+B01+B02 の実施順。
- API キー削除コマンドの要否、`extra_headers` 部分編集、
  `google_drive: None` の意味、`recent_paths` の代替表現。
- Leptos 0.8.22 に `on_mount` が無いため `Effect::new` で代替したが、
  初回限定実行の厳密性と Undo 履歴への影響は未検証。
