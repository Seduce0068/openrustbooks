---
type: Decision
title: "ADR-0216: ストリーミング未接続の設計受容"
description: "ストリーミング API の本番接続を見送り、設定表示を未配線として正直化する決定を記録する。"
tags:
  - ADR
  - ai
  - web
generated:
  by: dsh
  at: "2026-11-19T13:07:00+09:00"
status: accepted
---

## 背景

`AiClient::stream_complete()` / `TokenStream::next_token()` は実装済みだが、
本番呼び出しは 0 件だった（R6 TD-M18）。
`complete()` は `_streaming` を無視して常に `stream: false` でリクエストし、
`stream_generate()` は `generate()` の全文を 1 回だけ `on_token` へ渡す
疑似ストリームだった。一方 UI は `config.ai.streaming = true` を
「有効」と表示しており、利用者に動作中の誤認を与えた。
wasm の AI service は fail-closed で、トークン配信先も存在しない。

## 決定

- ストリーミングの本番接続は見送り、**設計受容**とする。
- UI の AI 情報表示を
  `ストリーミング: 未配線` の固定文言へ変更する
  （`cfg.ai.streaming` の参照と `SettingsAiView.streaming` は削除）。
  serde は未知フィールドを無視するため config パースは不変。
- `stream_generate()` の doc / inline コメントに
  「疑似ストリーム（完了本文を 1 回だけ通知、トークン単位ではない）」と明記する。
  挙動は変更しない。
- 将来接続する場合の前提条件を未決定として明示する（下記）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 設計受容＋正直化（採用） | 配信先 UI / IPC 契約が未定義で、接続しても利用されない |
| 今すぐ Tauri イベント接続 | lint retry の破棄トークン表示契約が未定で誤表示リスク |
| 設定 `streaming` を削除 | config 互換を壊すため本次は表示のみ変更 |

## 影響

- 利用者は AI 設定で「ストリーミング: 未配線」を見る。
- `config.ai.streaming` 値自体は設定ファイルに残るが、
  動作には影響しない（死設定のまま、未決定として継続）。
- 既存の AI 生成・lint retry 契約は不変。
- 新規依存なし、unsafe なし、E2E 253 維持。

## 検証（Runtime Proof）

- E2E に旧文言（`ストリーミング: 有効` / `無効`）の依存なし
  （`tools/` / `scripts/` grep 一致 0）。
- `cargo fmt --all -- --check`、
  `cargo test --all-targets --locked`（core 261 + desktop 79 + web 2）、
  `cargo clippy --all-targets --locked -- -D warnings`、
  wasm32 core / web check: すべて成功。
- `./scripts/verify.sh`: 全ゲート成功、E2E **253 項目すべて成功**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- 将来接続する場合の lint retry 表示契約
  （全試行 / 最終試行のみ / attempt 明示、ADR-0121 を継続）。
- Tauri 側の IPC（`AppHandle::emit` vs `ipc::Channel`、
  request_id・順序・キャンセル・エラー通知、バッチング）。
- 完了判定: `next_token() == None` 固定（ADR-0197）で
  OpenAI `finish_reason` の正常／異常をどう扱うか（ADR-0189 継続）。
- `tokens_used` の算出方法（再カウント vs stream 側公開）。
- timeout を total deadline から connect / read 分割へ変更するか
  （ADR-0189 未決定）。
- `config.ai.streaming` の配線または削除（todo R1-ai 死設定群）。
- ブラウザ版 AI 呼び出し方式と API キー（ADR-0003 継続）。
- 自動リトライ時の送信確認ダイアログの扱い。
