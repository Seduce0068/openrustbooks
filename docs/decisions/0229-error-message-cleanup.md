---
type: Decision
title: "ADR-0229: エラー文言と終了処理の整理"
description: "二重 prefix と JsValue Debug の露出を解消し、desktop の panic を Result ベースのエラー表示へ変更する決定を記録する。"
tags:
  - ADR
  - desktop
  - error
generated:
  by: dsh
  at: "2026-11-19T17:02:00+09:00"
status: accepted
---

## 背景

`desktop/src/main.rs` の `run()` は `()` を返し、
末尾の `.expect("error while running tauri application")` で panic していた（R6 C05）。
また `CoreError` の Display が `"Lint error: "` /
`"Validation error: "` / `"AI provider error: "` といった prefix を持つのに、
内部で同じ prefix を付ける二重付与が `lint.rs` / `client.rs` に残っていた。
`web/src/lib.rs` の invoke エラーは `JsValue` の Debug を
`"invoke failed: {err:?}"` として露出していた。

## 決定

- `run()` を `tauri::Result<()>` に変更し、`expect()` を削除する。
- non-mobile `main()` は `Display` で 1 行表示し、
  終了コード **101** を維持する（既存挙動との互換）。
- mobile `main()` は `log::error!` で記録する
  （`tauri::mobile_entry_point` が戻り値を捨てるため Result は返さない）。
- `lint.rs` の二重 prefix を削除する:
  - `CoreError::lint(format!("{e:?}"))`
  - `CoreError::validation(e.to_string())`
- `client.rs` の AI provider エラー 8 箇所を
  `CoreError::ai_provider(e.to_string())` に統一する。
  固定文言（予約ヘッダ、UTF-8 / JSON 解析、size limit）は維持する。
- `web/src/lib.rs` の invoke / parse エラーを固定文言へ変更する:
  - `デスクトップ版の設定を取得できませんでした`
  - `設定の形式が正しくありません`
- status の error / info 分離は行わず、ADR 未決定として継続する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| Result 化＋prefix 整理（採用） | panic を減らし、文言の重複と生情報露出を防ぐ |
| `fn main() -> Result` | `Error: <Debug>` 表示と mobile マクロ衝突で却下 |
| status を専用 aria-live へ分離 | ADR-0021 / 0226 の契約変更となり別 ADR が必要 |

## 影響

- desktop 起動時エラーは panic ではなく 1 行の Display 出力になる
  （終了コードは 101 を維持）。
- `Lint error: Lint error: ...` や
  `AI provider error: OpenAI API error: ...` の二重表記が消える。
- `JsValue(...)` の Debug が status / IPC へ出なくなる。
- E2E は文言に依存せず、期待総数 254 を維持する。
- 新規依存なし、unsafe なし。

## 検証（Runtime Proof）

- `grep`: `{err:?}` / `{error:?}` / `invoke failed` / `config parse failed` /
  旧二重 prefix は対象 4 ファイルに残存なし。
- `cargo test --all-targets --locked`、
  `cargo clippy --all-targets --locked -- -D warnings`、
  wasm32 core / web check: すべて成功。
- `./scripts/verify.sh`: 全ゲート成功、E2E **254 項目すべて成功**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- status の error / info 分離
  （専用 `aria-live`、通知寿命、二重読み上げ対策。ADR-0021 / 0154 / 0212 / 0226 継続）。
- 終了コード 101 の契約を今後も維持するか、1 へ変更するか。
- mobile でのエラー通知を `log::error!` のままにするか、
  ネイティブ通知へ昇格するか。
- Prefix 方針の完全統一（OpenAI / Ollama の技術ラベルを残すか、
  `CoreError` variant 側へ一本化するか）。
- ADR-0212 の英語固定文言とコードの日本語固定文言の不一致
  （現状は ADR-0226 / 0229 を優先）。
