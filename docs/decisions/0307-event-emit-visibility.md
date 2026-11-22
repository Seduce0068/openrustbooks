---
type: Decision
title: "ADR-0307: イベント発行失敗と import 混在状態の可視化"
description: "emit 失敗を警告ログ化し、import 失敗件数と navigation parse 失敗を status で告知する決定。"
tags:
  - ADR
  - desktop
  - web
generated:
  by: dsh
  at: "2026-11-22T09:20:00+09:00"
status: accepted
---

## 背景

desktop の全 `app.emit` / `window.emit` は `let _ = ...` で Result を破棄し、
発行失敗が完全に黙って捨てられていた。
web の import は成功件数のみを status に出し、
set_item 失敗や nav / landmarks / page-list の parse 失敗は
空配列 / 固定成功文言で隠れていた。

## 決定

- 全 emit の失敗を `log::warn!("<event> event failed")` で記録する。
  Tauri 2.12.3 の `tauri::Error` に code() が無いため固定文言のみとし、
  ペイロード / パス / 本文 / id はログへ出さない。
- import の `save_failed: bool` を失敗件数へ格上げし、
  成功文言は E2E 契約のため維持したまま、
  失敗時のみ「K 件失敗しました。」を末尾へ追加する。
- 目次 / landmarks / page-list の parse 失敗を検出し、
  固定日本語文言を status へ設定する。
  情報源なしの空配列は正常扱いとし、既存の remote resource 警告を優先する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 固定文言で可視化（採用） | 秘密情報を出さず失敗に気付ける |
| 生エラーを表示 | path / body / key の漏洩リスクがある |
| 成功文言を置換 | E2E 256 件の正規表現を壊すため却下 |

## 検証（Runtime Proof）

- 478 tests（core 341 / desktop 128 / web 9）、すべて成功。
- `cargo fmt --all -- --check`: rc=0。
- `cargo check --all-targets --locked`: rc=0。
- `cargo clippy --all-targets --locked -- -D warnings`: rc=0。
- wasm core / web check: rc=0。
- `npx tsc --noEmit`: rc=0。
- `./scripts/verify.sh`: 全ゲート成功、E2E **256 passed / 0 failed**、WDIO gate rc=0。

## 未決定

- emit 失敗を UI status へ伝播するか。
- parse 失敗ごとの詳細文言（固定文言の粒度）。
