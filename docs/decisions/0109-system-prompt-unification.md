---
type: Decision
title: "ADR-0109: system 指示のテンプレート一元化"
description: "OpenAI の固定 system message を削除し、system 指示をプロンプトテンプレートに一元化する決定を記録する。"
tags:
  - ADR
  - ai
  - bugfix
generated:
  by: dsh
  at: "2026-11-18T04:35:00+09:00"
status: accepted
---

## 背景

`client.rs` の `OpenAiClient::complete()` / `stream_complete()` は固定 system message
"You are a Markdown expert..." を system role で送っていた。一方
`prompt.rs::render()` はテンプレートの `system_prompt` をユーザーメッセージ先頭に
挿入するため、意味的に二重だった（ADR-0108 の未決定、
system-prompt-auditor / ai-behavior-auditor の指摘）。
Ollama には `client.rs` に system 経路が無く、テンプレート側が唯一の指示経路。

## 決定

- `OpenAiClient::complete()` / `stream_complete()` から固定 system message を削除し、
  共通ヘルパー `build_openai_request()` に集約する。
- `messages` は user message 1 件のみにする。
- Ollama 側のペイロード、`AiClient` トレイト、`complete()` のシグネチャ、
  テンプレート本文、`render()` は変更しない。
- 境界テストを追加する（messages に system role が無いこと、user content が
  prompt そのもの.render 結果の先頭にテンプレートの system_prompt が含まれること）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 固定文を削除しテンプレートに一元化（採用） | 重複を解消し、Ollama と方針を統一できる |
| テンプレートの `system_prompt` を空にする | Ollama の指示経路が消える |
| `render()` を `{system, user}` 構造に変更 | トレイト／両クライアント／wasm に波及し大きい |

## 影響

- OpenAI の `messages` は user 1 件になる（role=system は消える）。
- 指示内容自体はテンプレートの `system_prompt` として user 本文先頭に残るため失われない。
- generate では同一文の完全重複が解消され、他タスクではタスク別指示のみになる。
- Ollama の挙動は不変。新規依存は無い。

## 検証

- 追加テスト 3 件（user 1 件のみ、prompt がそのまま入る、render 結果に
  テンプレートの system_prompt が含まれる）。
- `cargo test --all-targets --locked`: core 158 + desktop 28、すべて成功。
- `./scripts/verify.sh` を実行し、6 ゲート・trunk build・WASM パス検査・
  E2E 228 項目がすべて成功した。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定（【推測】として記録）

- system role を優先するよう調整されたモデルでは、同一文言を user 本文先頭に
  置くと指示従順度が下がる可能性がある。本リポジトリに実測データは無く仮説。
  軽減策として rumdl lint ゲート（ADR-0002）が残る。実機で出力品質を
  実測して再評価する。
- Ollama API の `system` パラメータ対応は未確認（リポジトリ外仕様）。
  Ollama ペイロードを変更しないため今回の影響はなし。
- `render()` を `{system, user}` 構造に変更する中長期案（トレイト変更を伴う）。
