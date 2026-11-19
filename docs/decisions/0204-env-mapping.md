---
type: Decision
title: "ADR-0204: 環境変数マッピングの明示化"
description: "figment の split 依存をやめ、明示マップと生値検証で ADR-0122 の契約を維持する決定を記録する。"
tags:
  - ADR
  - core
  - config
generated:
  by: dsh
  at: "2026-11-19T09:52:00+09:00"
status: accepted
---

## 背景

`AppConfig::load()` は `Env::prefixed("OPENRUSTBOOKS_").split("_")` を使っていた。
figment は全 `_` を `.` へ置換するため
`OPENRUSTBOOKS_AI_MAX_RETRIES` が `ai.max.retries` になり、
`extract()` では届かない。ADR-0122 では `apply_env_max_retries()` による
extract 後の手動上書きをワークアラウンドとして採用し、
恒久方式を未決定としていた（R6 TD-M08b）。

## 決定

- `Env::prefixed("OPENRUSTBOOKS_").map(map_env_key)` へ変更し、
  既存環境変数名を明示マップする。
  - `AI_MAX_RETRIES` → `ai.max_retries`
  - 既存 single-word env 12 件（GENERAL_LANGUAGE / THEME / TELEMETRY、
    AI_ENABLED / OPENAI_MODEL / OPENAI_ORGANIZATION / OLLAMA_MODEL /
    STREAMING、LINT_ENABLED、EDITOR_MINIMAP、READER_THEME / HYPHENATION）も維持。
  - 未知の env は小文字化して serde に無視させる。
  - 秘密フィールド（api_key / client_secret / refresh_token）はマップしない
    （ADR-0098 の前提を維持）。
- **素の `map()` 単独は採用しない**。実測で
  `{"ai":{"max_retries":"abc"}}` の `extract()` は
  `InvalidType(Str("abc"), "usize")` となり起動失敗する。
- `include_env_key()` + `Env::filter()` を追加し、merge 前に
  `OPENRUSTBOOKS_AI_MAX_RETRIES` の生値を `parse::<usize>()` で検証する。
  - 有効値のみ env provider へ載せる。
  - 不正値はキーごと除外し、toml / default を維持する（ADR-0122 の契約）。
- `apply_env_max_retries()` は削除せず、
  `extract()` 後の防御的フォールバックとして役割を限定する。
- 優先順位 `env > toml > default` は維持する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 明示マップ＋生値検証フィルタ（採用） | 既存名を守りつつ手動ワークアラウンドを減らせる |
| `map()` 単独 | 不正値で `extract()` が Err になり起動失敗（実測） |
| 現行 `split()` ＋手動上書き維持 | ADR-0122 の未決定が残る |
| `figment::Jail` で環境を隔離 | core に `test` feature が無く利用不可 |

## 影響

- 環境変数名は **アンダースコア 1 つ**
  （`OPENRUSTBOOKS_AI_MAX_RETRIES`）を厳守。
- `OPENRUSTBOOKS_AI_MAX_RETRIES=abc` は無視され、
  toml 値（例 5）または既定値 3 が維持される（起動成功）。
- `default_provider` / `timeout_secs` / `base_url` 等多語フィールドは
  ADR-0122 の対象外のため今回は未対応（従来どおり無視）。
- 新規依存なし、unsafe なし、E2E 251 維持。

## 検証（Runtime Proof）

- 追加テスト 7 件:
  - マップ一覧／未知 env／秘密フィールド非マップ。
  - 不正 max_retries の filter 選別。
  - filter が無いと `extract()` が Err になる根拠。
  - `7 → 7`、`abc` + toml 5 → 5、`abc` → default 3。
  - single-word env 互換。
- 既存 `apply_env_max_retries` 3 件も維持。
- `cargo test -p openrustbooks-core --lib`: core **255 passed**（旧 248 + 7）。
- `cargo clippy --all-targets --locked -- -D warnings`、wasm check、
  `./scripts/verify.sh`: すべて成功。E2E **251 項目すべて成功**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- `default_provider` / `timeout_secs` / `base_url` 等、
  ネストが深い env のマッピング要否。
- env 由来値を `save()` で config.toml へ永続化するか
  （ADR-0122:48-49 を継続）。
- `OPENRUSTBOOKS_AI__MAX_RETRIES`（2 アンダースコア）表記は
  GOAL の誤記として扱ったが、互換展開の要否は無い（未使用）。
