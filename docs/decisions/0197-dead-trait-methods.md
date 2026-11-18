---
type: Decision
title: "ADR-0197: 未使用 trait メソッドの削除"
description: "TokenStream::is_done と StorageBackend::available_space を削除し、未実装 API への誤依存を防ぐ決定を記録する。"
tags:
  - ADR
  - core
  - maintenance
generated:
  by: dsh
  at: "2026-11-19T08:22:00+09:00"
status: accepted
---

## 背景

- `TokenStream::is_done()` は OpenAI 実装が常に `false` を返すだけだった。
  Ollama 実装は ADR-0189 で `self.done && self.responses.is_empty()` へ改善されたが、
  参照は unit test のみで production 呼び出し側が存在しなかった（R6 TD-L02）。
- `StorageBackend::available_space()` は `Ok(None)` を返すだけで、
  呼び出し側が存在しなかった（R6 TD-L03）。

## 決定

- `TokenStream` trait から `is_done()` を削除し、
  OpenAI / Ollama 両実装も削除する。
- ADR-0189 の回帰検知（`done` 行を読んでもキュー残りがある間は
  `None` を返さない）は、`next_token()` の
  `Some("A") → Some("B") → Some("C") → None → None` シーケンス検証へ吸収する。
- `StorageBackend` trait から `available_space()` と
  `LocalStorage` の `Ok(None)` 実装を削除する。
- 残存参照（`is_done` / `available_space`）は Rust ソースで 0 件にする。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 両方削除（採用） | 未実装／未使用 API への誤依存を防ぐ |
| `is_done` をテスト専用 API として残す | 公開 trait をテストが駆動する設計になる |
| `available_space` を残す | 呼び出し側ゼロのまま誤用リスクが残る |

## 影響

- ADR-0189:36-37 の `is_done()` セマンティクス決定は
  本 ADR で **supersede** される（キュー優先配信の挙動自体は維持）。
- `next_token()` の完了契約（キュー消化後に `None`）は変更しない。
- Ollama unit test 13 件はすべて存続し、
  `next_token()` シーケンスで同種の回帰を検証する。
- 新規依存なし、unsafe なし。

## 検証（Runtime Proof）

- `grep -rn 'is_done\|available_space' --include='*.rs' core/src desktop/src web/src`
  → 一致 0 件。
- `cargo test --all-targets --locked`: core 248 + desktop 62 + web 2。
- wasm-pack test --node: 10 件成功。
- `cargo clippy --all-targets --locked -- -D warnings`、
  `./scripts/verify.sh`: 成功。E2E **250 項目すべて成功**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- ストリーミングを `AiMarkdownService::stream_generate()` へ接続する際に
  完了判定 API が再び必要になるか（ADR-0189 / TD-M18 を継続、推測）。
- OpenAI の `finish_reason` 完了判定を公開 API 化するか。
- 将来 `StorageBackend` に容量情報が必要になった場合の再導入経路。
