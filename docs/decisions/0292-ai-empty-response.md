---
type: Decision
title: "ADR-0292: AI provider の空レスポンスを失敗扱いにする"
description: "choices / content / response が欠落した応答を空文字成功とせず、固定文言の provider エラーへ変換する決定。"
tags:
  - ADR
  - AI
  - security
generated:
  by: dsh
  at: "2026-11-22T04:25:00+09:00"
status: accepted
---

## 背景

OpenAI 経路は choices が空 / message.content が None の場合に
`unwrap_or_default()` で空文字を返していた。
Ollama 経路も `result["response"].as_str().unwrap_or("")` で空文字を返していた。
その結果、lint_passed=true の空文書が「生成成功」として扱われる危険があった。

## 決定

- `openai_response_content()` / `ollama_response_content()` を追加する。
- choices 欠落 / content None / response 欠落 / 非文字列 / 空文字を
  固定文言 `AI provider returned an empty response` の
  `CoreError::ai_provider` へ変換する。
- host / body / secret はエラー文言へ含めない（ADR-0287 に整合）。
- `complete()` のみを対象とし、`stream_complete()` は既存動作を維持する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 空レスポンスを provider error 化（採用） | 空文書の成功保存を防げる |
| 空文字を正当成功とする | lint を通過した空文書が保存されるため却下 |
| 生レスポンスをエラーへ含める | secret / body 露出のため却下 |

## 検証（Runtime Proof）

- 新規 test 8 件:
  - OpenAI: choices 欠落 / content None / 空文字 / 正常。
  - Ollama: response 欠落 / 非文字列 / 空文字 / 正常。
- `cargo test -p openrustbooks-core --locked`: **312 passed / 0 failed**。
- `cargo fmt --all -- --check`: rc=0。
- `cargo clippy -p openrustbooks-core --all-targets --locked -- -D warnings`: rc=0。
- `./scripts/verify.sh`: 全ゲート成功、E2E **256 passed / 0 failed**、WDIO gate rc=0。

## 未決定

- `stream_complete()` の空ストリームを明示的に失敗扱いにするか。
- 空白のみの本文を「実質的な空」として拒否するか。
