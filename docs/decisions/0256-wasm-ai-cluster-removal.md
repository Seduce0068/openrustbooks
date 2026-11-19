---
type: Decision
title: "ADR-0256: Wasm AI cluster の削除"
description: "未使用の fail-closed スタブ 3 型と自己言及 test を削除し、ADR-0156 の AI 部分のみ supersede する決定を記録する。"
tags:
  - ADR
  - wasm
  - ai
  - api
generated:
  by: dsh
  at: "2026-11-20T01:35:22+09:00"
status: accepted
---

## 背景

ADR-0156 で fail-closed 化した `WasmAiMarkdownService` /
`WasmPipelineResult` / `WasmDiffResult` は、4 メソッドすべてが固定エラーを
返すスタブのままである。ADR-0156 は「JS 側の型互換のため残置する」としていた
が、追跡対象の web / E2E / wasm test には参照が無く、根拠は失効している
（R6 R2-crates#3）。

- 追跡ソース（`web/src` / `tools/verify-web-reader.py` / wasm test）の参照は 0 件。
  `web/dist` の生成 JS は stale 成果物で追跡対象外（ADR-0243 / 0244 と同じ扱い）。
- ADR-0156 の残置根拠は「将来接続された際に JS の型が壊れないようにする」だったが、
  型を生成する API 自体が未使用であり、接続計画も無い。
- `WasmDiffResult` は生成箇所が無く、スタブ struct が残るだけである。
- 唯一の wasm test `ai_methods_fail_closed` は、API 自体の固定エラーを検証する
  自己言及 test で、削除対象に含める。

## 決定

- `WasmAiMarkdownService` / `WasmPipelineResult` / `WasmDiffResult` の
  doc コメント、wasm 属性、struct / enum / impl を属性ごと削除する。
- wasm test `ai_methods_fail_closed` を削除する。
- lint cluster（`WasmMarkdownLinter` / `WasmLintIssue` / `WasmLintFixResult`）と
  `assert_js_error` helper は維持する（ADR-0160 で実装済み）。
- ADR-0156 のうち **AI 部分のみ** を本 ADR で supersede する。
  lint 部分は ADR-0160 で実装済みのため維持し、status は `accepted` のままとする
  （プロジェクト規約: ADR の status は `accepted` を用いる）。
- 他 ADR の「AI cluster 削除は別案件」という記述は、本 ADR で完了として更新する。

## 根拠

| 根拠 | 内容 |
| --- | --- |
| 参照 0 件 | 追跡ソース（web / E2E / wasm test）に 3 型の利用が無い |
| 型互換の根拠消滅 | ADR-0156 の「JS 型互換のため残置」は、接続される API が無いため意味を失った |
| 未構築の型 | `WasmDiffResult` はどの経路からも生成されない |
| fail-closed より強い | 削除は公開面そのものを無くし、誤用・誤接続の余地を残さない |

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 残置（ADR-0156 のまま） | 未使用スタブを公開し続け、型互換の根拠も失効しているため却下 |
| 削除（採用） | 公開面を実際に減らし、自己言及 test も消える |
| JS export のみ非公開化 | `wasm-bindgen` は Rust の `pub` と独立して export を生成するため不十分（ADR-0243 / 0244 と同じ理由） |

## 影響

- `core/src/wasm.rs` から 3 型・impl・test・import を削除する。
- wasm-bindgen-test は **15 → 14 件**（`ai_methods_fail_closed` の 1 件を削除）。
- core test は **284 件維持**（wasm-bindgen-test は native target の test 数に含まれない）。
- E2E は **255 件維持**。追跡ソースに変更が無いため期待総数は不変。
- 公開 wasm API から AI 経路が消え、ブラウザ側には AI を呼び出す経路が存在しなくなる
  （ADR-0232 を更新）。
- 外部 JS 利用者の有無は未決定で、いた場合は破壊的変更になり得る
  （リポジトリ内に利用証拠なし）。

## 検証（Runtime Proof）

- `grep`: 追跡ソース（`core/src` / `web/src`）の 3 型と `ai_methods_fail_closed` の参照は 0 件。
- `#[wasm_bindgen_test]` 属性は 14 件（削除前 15 件）。
- `cargo test -p openrustbooks-core --locked`: **284 passed / 0 failed**（削除前後で不変）。
- 全ゲート（`cargo fmt --all -- --check` / `cargo check --all-targets --locked` /
  `cargo clippy --all-targets --locked -- -D warnings` / wasm32 core / web check /
  `wasm-pack test --node --locked` / `./scripts/verify.sh`）の結果は親エージェントの
  Runtime Proof に従う（E2E 255 維持を見込む）。
- CI は billing issue で実行不可（ローカル検証で代替、従来どおり）。

## 未決定

- 外部 JS 利用者の有無（リポジトリ内に証拠なし）。
- deprecation shim の要否。
- ADR-0156 の `status` 変更（本プロジェクト規約では ADR の status は `accepted` を維持）。
- バンドルサイズの before / after 実測（ADR-0160 / 0242 / 0243 / 0244 / 0246 を継続）。 比較可能 before は [ADR-0267](0267-bundle-size-baseline.md) に記録済み。

## 参照

- [ADR-0156: Wasm AI/lint スタブの fail-closed 化](0156-wasm-stub-failclosed.md)
- [ADR-0160: 実 Wasm lint](0160-real-wasm-lint.md)
- [ADR-0227: AI クライアント注入点の設計受容](0227-ai-client-injection.md)
- [ADR-0232: 初版リリーススコープ](0232-initial-release-scope.md)
- [ADR-0243: 未使用 Wasm API の削除](0243-unused-wasm-api-removal.md)
- [ADR-0244: WasmDocument の削除](0244-wasm-document-removal.md)
- [ADR-0246: workspace 依存の第一段統一](0246-workspace-dependency-unification.md)
