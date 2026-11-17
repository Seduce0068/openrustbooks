---
type: Decision
title: "ADR-0101: AI 送信先 base_url の許可リスト"
description: "AI の送信先 URL を既定ホストと loopback のみに制限し、JS からの任意送信先差し替えを防ぐ決定を記録する。"
tags:
  - ADR
  - security
  - ai
generated:
  by: dsh
  at: "2026-11-18T03:05:00+09:00"
status: accepted
---

## 背景

`OpenAiConfig.base_url` / `OllamaConfig.base_url` は単なる String で、
Tauri IPC の `update_config` / `update_ai_config` から JS が任意 URL
（例: `http://attacker.example`）に変更して有効化できた。Rust 側 HTTP は
WebView の CSP 対象外のため、同意なく本文が外部へ送られるリスクがあった
（ADR-0099 / 0100 の未決定、baseurl-auditor の指摘）。

## 決定

- `core/src/ai_markdown/client.rs` に公開 API を追加する。
  - `ProviderKind { OpenAi, Ollama }`
  - `validate_base_url(&str, ProviderKind) -> CoreResult<()>`
- 検証は既存依存 `url::Url` で parse し、次を確認する。
  - userinfo（`username` / `password`）は拒否。
  - scheme + `host_str()` + port の完全一致（大小文字無視、`ends_with` 等は不使用）。
  - OpenAI: `https` + `api.openai.com` + port なし / 443。
  - Ollama: `http` + loopback（`localhost` / `127.0.0.1` / `[::1]`）+ port 11434。
  - path は検査しない。
- `OpenAiClient::new` / `OllamaClient::new` の先頭で検証し、
  `create_client` 経由で全経路に適用する。
- `desktop/src/commands/config.rs` の `update_config` / `update_ai_config` でも
  merge 後に同期的に検証し、不正なら command が Err を返す。
  `let _ = init_ai_service().await;` でエラーを飲み込まない。
- エラーメッセージには userinfo を含み得る URL 全体を出さず、host と理由のみを出す。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| ハードコード許可リスト（採用） | JS から書き換えられない。IPC が `AppConfig` 全体を受け取るため、設定に置くと無意味 |
| `https` 必須のみ | `https://attacker.example` を防げない |
| 警告ログのみ | 送信先の差し替えを防止できない |

## 影響

- **互換性コスト**: Azure OpenAI / OpenRouter / LiteLLM / 社内プロキシ等の
  カスタムエンドポイント、既定以外の port/host の Ollama は拒否される。
- 既存 `config.toml` にカスタム base_url が保存されている場合、起動時に
  service が作られず AI 機能が使えなくなる（エラーは起動ログと command 結果に出る）。
- カスタム opt-in は、JS から変更できない config.toml / 環境変数専用フラグ等を
  設計した上で別 ADR で解禁する。
- 新規依存は無い。WASM スタブには影響しない。

## 検証

- core に 10 件のテストを追加（既定許可、大小文字/443、http/https 攻撃ホスト拒否、
  loopback 3 形式、ftp/file 拒否、userinfo 拒否、エラーメッセージ非漏洩、
  非既定 port 拒否、プロバイダ不一致、不正 URL）。
- `cargo test --all-targets --locked`: core 146 + desktop 10、すべて成功。
- `./scripts/verify.sh` を実行し、6 ゲート・trunk build・WASM パス検査・
  E2E 228 項目がすべて成功した。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- カスタムエンドポイントの安全な opt-in 経路（config.toml 直接編集専用フラグ等）。
- 送信プレビュー＋ネイティブの同意ダイアログ。
- WASM に fetch ベースの AI クライアントを実装する際の同リスト適用。
- `IntersectionObserver` による画像の遅延読み込み（ADR-0034 から継続）。
EOF
