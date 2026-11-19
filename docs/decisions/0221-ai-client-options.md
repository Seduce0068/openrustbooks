---
type: Decision
title: "ADR-0221: AI クライアント設定の配線"
description: "OpenAI extra_headers と Ollama options を安全にクライアントへ渡す決定を記録する。"
tags:
  - ADR
  - core
  - ai
generated:
  by: dsh
  at: "2026-11-19T14:43:00+09:00"
status: accepted
---

## 背景

`OpenAiConfig.extra_headers` / `OllamaConfig.options` は config.toml から読まれ、
IPC DTO と Debug ではマスクされていた（ADR-0102 / 0186）。
しかしクライアント生成時に無視されており、
ADR-0102 の「config.toml の headers は AI リクエストへ送信される」という
契約と実装が一致していなかった（R6 TD-M20）。

## 決定

- OpenAI: `extra_headers` をキー順（BTreeMap 相当）でソートし、
  `OpenAIConfig::with_header()` 経由で全リクエストへ付与する。
- ヘッダ名／値の不正は固定文言
  `OpenAI extra header is invalid` で fail-closed とし、
  キー・値をエラー文言へ出さない。
- 予約ヘッダ `authorization` / `openai-organization` /
  `openai-project` / `openai-beta` は大小文字無視で拒否し、
  `OpenAI extra header is reserved` を返す。
  `async-openai 0.42.2` は custom header が既存ヘッダより優先されるため、
  `with_api_key()` / `with_org_id()` の上書きを防ぐため。
- Ollama: `options` を `serde_json::Map` で保持し、
  `build_ollama_request()` で固定値
  （`num_predict = max_tokens`、`temperature = 0.3`、`top_p = 0.9`）
  の上から config 値をマージする。
- Ollama の `num_predict` / `model` / `stream` は config 側から上書き不可とし、
  呼び出し側の `max_tokens` とトップレベル値を優先する。
  `temperature` / `top_p` は config 側の上書きを許可する（仕様変更）。
- `complete()` / `stream_complete()` は同じ request builder を共有する。
- 既存の allowlist / `merge_secrets` / DTO マスク / wasm fail-closed 契約は維持する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 安全な配線（採用） | config.toml の設定を契約どおり使いつつ権限境界を守れる |
| 丸ごと無視（現状維持） | ADR-0102 と矛盾し proxy / gateway 利用を壊す |
| 予約ヘッダも許可 | API key / organization が上書きされるため却下 |

## 影響

- config.toml に正しい `extra_headers` / `options` を書いた利用者は
  実際に送信される（ADR-0102 の契約どおり）。
- `temperature` / `top_p` を config で変更できるようになった。
- `client.rs` は native 専用で wasm には影響しない（ADR-0003）。
- リトライは同一 `Client` を再利用するため、
  headers は各試行へ自動的に再適用される。
- 新規依存なし、unsafe なし、E2E 253 維持。

## 検証（Runtime Proof）

- unit test 7 件追加:
  - 妥当なヘッダ受理、不正キー／値の拒否（非漏洩）。
  - 予約ヘッダの大小文字無視拒否。
  - Ollama マージ（カスタム値反映・予約キー非上書き）。
  - `OllamaClient::new()` が options を保持。
- `cargo test -p openrustbooks-core`: core **267 passed**（旧 260 + 7）。
- `cargo clippy --all-targets --locked -- -D warnings`、
  wasm32 core / web check、`./scripts/verify.sh`: すべて成功。
- E2E **253 項目すべて成功**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- reqwest 既定の redirect 追従で extra_headers が別 host へ送られる可能性
  （ADR-0218 の未決定を継続）。
- Ollama options の型検証（現状は JSON のまま、不正型は Ollama 側の 400 に依存）。
- 予約ヘッダの拡張（`OpenAI-*` 以外の制御ヘッダ）。
- ブラウザ版 AI 実装時に同一の header / options 契約を適用するか
  （ADR-0003 / 0117 継続）。
