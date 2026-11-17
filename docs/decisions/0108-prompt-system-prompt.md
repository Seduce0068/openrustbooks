---
type: Decision
title: "ADR-0108: 内蔵プロンプトのデシリアライズ修復"
description: "内蔵 .hbs テンプレートに必須フィールドを追加し、AI サービスの初期化失敗を解消する決定を記録する。"
tags:
  - ADR
  - ai
  - bugfix
generated:
  by: dsh
  at: "2026-11-18T04:25:00+09:00"
status: accepted
---

## 背景

`PromptTemplate` は `system_prompt: String` と `user_prompt_template: String` を
必須とするが、`core/prompts/*.hbs` 全 7 ファイルの front matter には
`system_prompt` が無く、さらに `user_prompt_template` も無かった。
加えて `frontmatter.hbs` / `explain_code.hbs` の `prompt_type` が
serde の snake_case バリアントと不一致（`"frontmatter"` / `"explaincode"`）だった。
そのため `PromptRegistry::with_builtins()` が失敗し、native の
`AiMarkdownService::new()` は設定に関わらず常にエラーだった
（ADR-0107 の未決定、service-refresh-implementer の指摘）。

## 決定

- 全 7 テンプレートの front matter に次を追加・修正する。
  - `system_prompt`: `client.rs` の固定 system message と同等の意味
    （generate は同一文、他はタスク別の亜種）。
  - `user_prompt_template = ""`: `parse_template()` が本文で上書きするため
    空文字プレースホルダ。
  - `prompt_type = "front_matter"` / `"explain_code"` に修正。
- Handlebars 本文は変更しない。
- `prompt.rs` にテストを追加する。
  - `with_builtins()` が成功し 7 種すべて registered。
  - 各 `system_prompt` が非空。
  - `render()` の先頭に `system_prompt` が 1 回だけ含まれる。
  - AI 有効 config で `AiMarkdownService::new()` が成功（HTTP 要求なし）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| front matter を修正（採用） | デシリアライズを直し、既存テンプレート本文を維持できる |
| `PromptTemplate` 側を Option に変更 | API が曖昧になり、system 指示の欠落を許してしまう |
| テンプレートを削除 | 要件 REQ-010 のプロンプト管理を失う |

## 影響

- `AiMarkdownService::new()` が成功し、ADR-0107 の fail-closed refresh が
  有効時に service を生成できるようになる。
- `render()` はテンプレートの `system_prompt` をユーザーメッセージ先頭に
  プレーンテキストとして挿入する。
- 新規依存は無い。

## 検証

- 追加テスト 4 件。
- `cargo test --all-targets --locked`: core 155 + desktop 28、すべて成功。
- `./scripts/verify.sh` を実行し、6 ゲート・trunk build・WASM パス検査・
  E2E 228 項目がすべて成功した。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 解決済み

- ~~OpenAI では `client.rs` の固定 system message と、テンプレートの
  `system_prompt`（`render()` が user 本文先頭に挿入）が意味的に二重になる。~~
  → ADR-0109 で固定 system message を削除し、system 指示をテンプレートに
  一元化（解消済み）。
- ~~`render()` + `client.complete()` をまたぐ境界テストが無く、
  既存の `render_starts_with_system_prompt` は API リクエスト全体の
  重複を検出できない。~~
  → ADR-0109 で境界テスト 3 件を追加（messages に system role が無いこと、
  render 結果先頭にテンプレートの `system_prompt` が含まれること）し解消済み。

## 未決定（system-prompt-auditor の指摘）

- Ollama API の `system` パラメータ対応は未確認（リポジトリ外の仕様）。
