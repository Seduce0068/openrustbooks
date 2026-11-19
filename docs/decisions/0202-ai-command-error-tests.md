---
type: Decision
title: "ADR-0202: AI コマンド異常系のテスト固定"
description: "confirmation / format / alignment の分岐を純粋関数へ抽出し、既存挙動を unit test で固定する決定を記録する。"
tags:
  - ADR
  - desktop
  - testing
generated:
  by: dsh
  at: "2026-11-19T09:24:00+09:00"
status: accepted
---

## 背景

`desktop/src/commands/ai.rs` の `confirm_send()` は
`Ok(true)` / `Ok(false)` / `Err(_)` の 3 分岐を持つが unit test が無く、
`convert_to_table()` の不正 format と未知 alignment の
Left フォールバックも未検証だった（R6 TD-L06）。

## 決定

- `interpret_confirmation(operation, Result<bool, RecvError>) -> Result<(), String>`
  を追加し、3 分岐のエラー文言を完全一致で固定する。
  - `Ok(false)`: `"{operation} がキャンセルされました"`
  - `Err(_)`: `"{operation} の確認ダイアログが応答しませんでした"`
  - 実 `RecvError` は `drop(tx)` + `rx.await` で生成してテストする。
- `parse_table_format(&str) -> Result<TableFormat, String>` を追加する。
  - `to_lowercase()` で csv / json / tsv を解釈（既存挙動）。
  - 不正値は `Invalid format. Use csv, json, or tsv` 固定。
- `parse_table_alignment(Option<Vec<String>>) -> Option<Vec<TableAlignment>>` を追加する。
  - `to_lowercase()` で left / center / right、未知値は Left。
  - `None` は `None` のまま維持（`Some(vec![])` に潰さない）。
- 4 つの送信コマンドは `confirm_send(...).await?` を維持し、
  `convert_to_table()` は format 解釈（confirm 前）／alignment 解釈（confirm 後）の
  元の呼び出し順を維持する。
- `TableFormat` / `TableAlignment` は `PartialEq` 未実装のため、
  テストでは `matches!` で検証する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 純粋関数を抽出してテスト（採用） | AppHandle 不要で 3 分岐を直接検証できる |
| Tauri 実機 E2E で確認 | E2E は web reader 対象で desktop IPC 文言をカバーしない |
| テストを追加しない | 文言変更の回帰を検出できない |

## 影響

- ADR-0104 の同意ダイアログ契約（JS consent 不受取、
  `show()` + oneshot、IPC 引数不変、lint はダイアログ無し）は維持。
- エラー文言は operation 名のみで、本文・URL・フィールド値は含まない。
  `build_summary()` の 80 文字プレビューはダイアログ用で従来どおり。
- 旧 `match rx.await` は command 側に残らず、
  `rx.await` は `confirm_send()` の 1 箇所のみ。
- 新規依存なし、unsafe なし、E2E 251 維持。

## 検証（Runtime Proof）

- 追加テスト 10 件:
  - confirmation 3 分岐（assert_eq 完全一致）。
  - format csv / json / TsV（大小混在）/ 不正。
  - alignment left / center / right / 大小混在＋未知 Left / None。
- `cargo test -p openrustbooks-desktop`: **72 passed**（旧 62 + 10）。
- `cargo clippy -p openrustbooks-desktop --all-targets --locked -- -D warnings`: rc=0。
- `./scripts/verify.sh`: 全ゲート成功、E2E **251 項目すべて成功**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- `Some(vec![])` の alignment テスト追加要否（実装は None と区別して維持）。
- `operation` を enum 化して文言の埋め込みを型で制限するか。
- Tauri 実機経路でのエラー文言アサーション（現状は構造的に担保）。
- ADR-0104 の既存未決定（全文プレビュー等）を継続。
