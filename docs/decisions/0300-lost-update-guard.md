---
type: Decision
title: "ADR-0300: 設定保存と autosave の lost update 防止"
description: "settings 保存中の入力を遮断し、editor の last_text を doc_id 変更・手動保存時に同期する決定。"
tags:
  - ADR
  - web
  - concurrency
generated:
  by: dsh
  at: "2026-11-22T06:40:00+09:00"
status: accepted
---

## 背景

SettingsForm は保存開始時の view をスナップショットし、
成功時に config signal 全体を updated で置換していた。
保存 in-flight 中の編集は成功レスポンスで捨てられた。
editor の autosave_last_text は doc_id 変更時と手動保存時に更新されず、
別文書 / 手動保存後の本文が自動保存でスキップされることがあった。

## 決定

- SettingsForm に `saving: RwSignal<bool>` を追加する。
- on_save 先頭で再入を拒否し、saving=true を設定する。
- 成功・失敗・config=None の全経路で saving=false に戻す。
- text/number は既存 readonly 契約を維持しつつ、
  `prop:disabled=move || saving.get()` を併記する。
- 全 on:input / on:change ハンドラ先頭で saving 中は更新を返す。
- checkbox / select / 保存ボタンは saving 中 disabled にする。
- editor は doc_id 変更時に autosave_last_text をクリアし、
  手動保存成功時に保存済み本文を設定する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| saving signal + handler guard（採用） | DOM 属性に依存せず確実に遮断できる |
| disabled 属性のみ | autofill / IME で発火する入力を止められない |
| dirty フラグで再保存 | 実装が大きく E2E 契約を変えるため見送り |

## 検証（Runtime Proof）

- `cargo fmt --all -- --check`: rc=0。
- `cargo check -p openrustbooks-web --target wasm32-unknown-unknown --locked`: rc=0。
- `npx tsc -p tsconfig.json`: rc=0。
- `npm run test:wdio`: 1 passing / 1 total（既存 spec）。
- `./scripts/verify.sh`: 全ゲート成功、E2E **256 passed / 0 failed**、WDIO gate rc=0。

## 未決定

- saving 中の disabled / 再入防止 / 失敗復帰の専用回帰テスト。
- dirty フラグによる保存後再保存の UX。
