---
type: Decision
title: "ADR-0193: 設定フォームの読み取り専用化"
description: "ブラウザ版の設定フォームを契約どおり読み取り専用にし、変更できると誤解させない決定を記録する。"
tags:
  - ADR
  - web
  - ux
generated:
  by: dsh
  at: "2026-11-19T07:24:00+09:00"
status: accepted
---

## 背景

`SettingsForm()` は読み取り表示のみの契約（ADR-0128）だったが、
チェックボックス／テキスト入力が `on:input` / `on:change` で signal を更新し、
利用者は変更が効いたと誤解し得た（R6 PR-M11）。
保存ボタンは disabled で、ブラウザ版では設定を永続化できない。

## 決定

- text / number（Model / Ollama Model / Font Size）と API Key の password に
  `readonly=true` を付ける（`prop:value` は維持）。
- checkbox 6 点（Enable AI / Line Numbers / Word Wrap / Vim Mode /
  Enable Linting / Auto-fix）と select（Provider）に `disabled=true` を付ける
  （`prop:checked` / `prop:value` は維持）。
- 全 `on:input` / `on:change` による setter 更新を削除する。
  `fetch_config()` の結果を表示する signal と status 文言
  「デスクトップ版のみ利用可能」は維持する。
- フォーム先頭に常時説明を追加する:
  「ブラウザ版は表示のみです。設定の変更・保存はデスクトップ版で行ってください。」
- E2E に 1 件追加し、期待総数 249 → **250**（check 233 + equal 17）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 入力不可＋常時説明（採用） | 誤解を防ぎつつ表示契約を守れる |
| 入力は効いたまま保存 only disabled | 利用者を誤導する |
| ConfigPatch 保存を配線 | `withGlobalTauri` / CSP 設計が必要で範囲外 |

## 影響

- ブラウザ版では設定の変更・保存ができないことが UI で明確になる。
- API Key の実値は従来どおり DOM に出さず、
  `has_api_key` を「設定済み／未設定」と表示する（ADR-0128 を維持）。
- E2E は既存の fallback 表示チェックを維持し、
  新しいチェックで `readOnly` / `disabled` / 説明文を検証する。
- 新規依存なし、unsafe なし。

## 検証（Runtime Proof）

- E2E「ブラウザ版の設定フォームは入力不可で常時説明を表示する」が ok。
  - text / number / password の `readOnly`。
  - checkbox / select の `disabled`。
  - `.settings-notice` の表示。
- 既存「ブラウザ版の設定画面は例外なくデスクトップ版のみと表示する」も維持。
- E2E **250 項目すべて成功**。
- 全単体テスト core 246 + desktop 62 + web 2、wasm-test 10、
  audit 脆弱性 0 件。
- `python3 -m py_compile`、`cargo fmt --all -- --check`、wasm check、
  `./scripts/verify.sh`: すべて成功。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- `withGlobalTauri` 未設定のため desktop でも読み取りが動かず
  fallback 表示になる問題（ADR-0128 / 0151 を継続）。
- ConfigPatch 経由の保存接続と CSP / capability 強化の同時実施。
- 設定の明示的復元 UI（ADR-0157 から継続）。
