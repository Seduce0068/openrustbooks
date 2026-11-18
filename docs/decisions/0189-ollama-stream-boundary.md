---
type: Decision
title: "ADR-0189: Ollama ストリームの行境界とタイムアウト"
description: "Ollama の JSONL ストリームをバッファリングし、HTTP タイムアウトとエラーを fail-closed で扱う決定を記録する。"
tags:
  - ADR
  - core
  - ai
generated:
  by: dsh
  at: "2026-11-19T06:22:00+09:00"
status: accepted
---

## 背景

`OllamaTokenStream::next_token()` はチャンクごとに `text.lines()` をパースしていたため、
JSON 行がチャンク境界で分断されると不正として捨てられ、
1 チャンクに複数行入ると最初の response だけを返して残りを失った（R6 PR-M01）。
また `reqwest::Client::new()` に timeout が無く、
無応答の Ollama に対して Tauri コマンドが戻らなかった（TD-M04）。

## 決定

- `OllamaTokenStream` に `pending: Vec<u8>` / `done: bool` /
  `responses: VecDeque<String>` を追加する。
- チャンクのバイトを `pending` に累積し、`\n` で終わった完全行だけをパースする。
  未完了行は次チャンクへ残す（UTF-8 途中バイトも保持）。
- EOF で改行の無い残り行も最後の行としてパースする。
- `parse_line()` を `CoreResult` 化し、次を明示エラーにする:
  - `json["error"]` が非 null。
  - 不正 JSON。
  - 不正 UTF-8。
  - エラー文言は固定文字列のみで、本文・URL は埋め込まない。
- `done` 行の空 `response` はキューへ入れない。
- `is_done()` は `self.done && self.responses.is_empty()` とし、
  キューが残っている間は完了を返さない。
- `MAX_PENDING_BYTES = 1 MiB` を設け、超過で fail-closed。
- `complete()` / `stream_complete()` の `.send()` に `.error_for_status()` を追加。
- timeout は `create_client(&AiConfig)` から
  `OllamaClient::new(&config.ollama, config.timeout_secs)` へ渡す。
  `OllamaConfig` への新規フィールド追加は行わない（DTO / merge 契約を維持）。
- OpenAI ストリームは変更しない。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| バイト累積＋行キュー（採用） | 分断行・複数行・UTF-8 分割を同時に解消 |
| チャンクごと lines()（旧挙動） | トークン欠落の原因 |
| OllamaConfig へ timeout 追加 | DTO / ADR-0131 / 0132 / 0186 へ波及するため却下 |

## 影響

- reqwest の `timeout()` はリクエスト全体の total deadline であり、
  長時間の生成を 60 秒（AiConfig 既定）で切る可能性がある（未決定）。
- ネットワーク／HTTP エラーの `reqwest::Error` Display には
  URL が含まれ得るが、これは既存挙動で本次は変更していない。
- ストリーミングはまだパイプラインへ接続されておらず（TD-M18）、
  E2E 249 件は本変更を実質的に検証しない。
  unit test が唯一の実証（未決定）。
- 新規依存なし、unsafe なし。

## 検証（Runtime Proof）

- `ollama_*` unit test 14 件:
  分割 JSON、複数行、done + response、UTF-8 2 箇所分割、EOF 行、
  ストリームエラー、不正 JSON、error フィールド、不正 UTF-8、
  pending 上限、空 response、空行、timeout 配線。
- `cargo test --all-targets --locked`: core **241** + desktop 62 + web 2。
- `cargo clippy --all-targets --locked -- -D warnings`、
  `./scripts/verify.sh`: 成功。E2E **249 項目すべて成功**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- total deadline から `connect_timeout` + `read_timeout` への変更。
- 実 Ollama との結合テスト（現状は unit のみ）。
- ネットワークエラー文言の URL マスキング。
- ストリーミングを `AiMarkdownService::stream_generate()` へ接続するか（TD-M18）。
