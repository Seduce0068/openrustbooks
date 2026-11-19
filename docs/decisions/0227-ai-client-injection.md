---
type: Decision
title: "ADR-0227: AI クライアント注入点の設計受容"
description: "core 内のモック注入を維持し、公開注入点を追加しない決定と将来の最小設計を記録する。"
tags:
  - ADR
  - core
  - ai
  - testing
generated:
  by: dsh
  at: "2026-11-19T16:20:00+09:00"
status: accepted
---

## 背景

- `AiMarkdownPipeline::new()` は `client::create_client(ai_config).await?` を直接呼び、
  `Arc<dyn AiClient + Send + Sync>` を生成する（`core/src/ai_markdown/mod.rs:81`）。
- `#[cfg(test)] mod tests` の `CountingAiClient`（`mod.rs:601`）は
  `complete` / `stream_complete` / `count_tokens` / `model_name` の 4 メソッドを実装し
  （`impl client::AiClient`、`mod.rs:607`）、`pipeline_with_mock()`（`mod.rs:642`）が
  `AiMarkdownPipeline { client: mock, ... }` と非公開フィールドを直接構築して注入する。
  ADR-0133:36-39 が `#[cfg(test)]` 内のテスト契約として記録済み。
- `AiClient` は `#[async_trait]` で object-safe で、
  `Arc<dyn AiClient + Send + Sync>` へのブランケット impl が既存（`client.rs:22,45-47`）。
- `AiMarkdownPipeline`（`mod.rs:33-34`）とその impl（`mod.rs:68`）は
  `#[cfg(not(target_arch = "wasm32"))]` 下にあり、将来の `with_client` も同じ cfg 配下で
  native 専用となる。
- ネットワーク／FS なしで決定的にテストできており、公開注入点を追加する利得は限定的。
  一方で公開注入点は `validate_base_url`（`client.rs:114`）と予約ヘッダ拒否（ADR-0221）の
  fail-closed 境界を迂回し得る。

## 決定

- 現状を設計受容とし、**公開注入点は追加しない**。本番コード・E2E は変更しない。
- 将来クロスクレート要件（他クレートからのクライアント差し込み）が生じた場合のみ、
  同期メソッド
  `with_client(client: Arc<dyn AiClient + Send + Sync>, linter, prompt_registry, config) -> Self`
  を追加し、`new()` は生成したクライアントを渡して委譲する。
- 追加時の公開範囲はまず `pub(crate)` から検討し、
  必要な根拠が揃うまで `pub` へ昇格しない。
- 既存テストのフィールド直接構築は core 内の `#[cfg(test)]` に閉じ、
  公開 API の契約としては扱わない。

## 信頼境界

- 注入された `Arc<dyn AiClient + Send + Sync>` は**信頼済み**コードとして扱う。
  `validate_base_url` による送信先 allowlist（ADR-0101）は `create_client()` 生成経路に加え、
  desktop の `validate_ai_base_urls()`（`desktop/src/commands/config.rs:537`）でも
  保存・再初期化前に適用される。予約ヘッダ拒否（ADR-0221）も生成経路の検証であり、
  注入経路はこれらを迂回し得る。
- クライアント注入は IPC を通らないため IPC DTO 境界（ADR-0102 / 0186）の対象外である。
  この経路を IPC コマンドや wasm 公開 API へ公開すれば、DTO 正規化と機微フィールド制御を
  迂回し得るため公開しない。
- したがって公開範囲は core 内（まず `pub(crate)`）に限定するのが安全である。

## 根拠

- ADR-0133: `#[cfg(test)]` 内の 4 メソッドモックとフィールド直接構築による注入が
  テスト契約として記録済み。
- 既存 unit test 成功: `cargo test -p openrustbooks-core` で core **267 passed**
  （ADR-0221 の実績）。
- `client.rs` は `#[cfg(not(target_arch = "wasm32"))]` 下の native 専用（ADR-0003）で、
  wasm 側の `AiClient` は未実装（未決定）。
- fail-closed 境界（ADR-0101 / 0102 / 0186 / 0221）を公開 API で迂回させない。
- 新規依存なし、unsafe なし、コード変更なし。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 設計受容・公開注入点なし（採用） | 既存テストが決定的で利得が限定的、境界迂回リスクを避けられる |
| 即時に `pub fn with_client(Arc<dyn AiClient + Send + Sync>, ...)` を追加 | 信頼境界を越え、allowlist / 予約ヘッダ拒否を迂回し得るため却下 |
| `pub(crate)` で先に追加 | 将来要件が出た時点の最小ステップとして保留（今回は追加しない） |

## 影響

- 本番コード・テスト・E2E・依存は不変（E2E 期待総数 254 は不変）。
- `new()` は引き続き `create_client()` を直接呼び、fail-closed 検証を維持する。
- 将来 `with_client` を追加しても、`new()` からの委譲に限定すれば
  公開面を増やさずクロスクレート要件へ対応できる。
- markdownlint 0 issues（`/tmp/mdlint.jsonc`）。

## 未決定

- 公開範囲（`pub` vs `pub(crate)`）。
- 将来 `with_client` を採用する場合の最終的な引数順と所有権（値 vs 参照）。
- ADR-0133（`#[cfg(test)]` 内テスト契約）の更新要否（維持する限り不要だが、公開時に再確認）。
- `AiMarkdownService` への注入点の有無（`core/src/ai_markdown/pipeline.rs:16`）。
- wasm 側の `AiClient`（ADR-0003 継続、現状は `WasmAiMarkdownService` が別経路）。
