---
type: Decision
title: "ADR-0208: 未保存フラグのクリーンアップ"
description: "文書削除時に unsaved_changes のキーも削除し、force close 後の終了確認誤作動を防ぐ決定を記録する。"
tags:
  - ADR
  - desktop
  - state
generated:
  by: dsh
  at: "2026-11-19T10:43:00+09:00"
status: accepted
---

## 背景

`AppState::remove_document()` は `documents` から文書を削除するが、
`unsaved_changes` のキーを削除していなかった（R6 TD-L11）。
force close 等で未保存フラグ `true` の文書を閉じると
`has_unsaved_changes()` が `true` のままになり、
終了時確認が誤作動し得た。既存テストは `mark_saved()` 後に削除するため
この経路を拾えていなかった。

## 決定

- `remove_document()` が文書削除時に同じ id の `unsaved_changes` エントリも削除する。
- 文書が存在しない場合も孤立した unsaved フラグを削除する。
- 2 つの RwLock は順番に取得・解放し、同時に保持しない
  （`documents.write()` のガードを先にドロップしてから `unsaved_changes.write()` を取得）。
- 戻り値 `Option<Document>` と `close_document()` の
  force / non-force 契約、イベント発行条件は変更しない。
- unit test 2 件を追加:
  - `remove_document_clears_unsaved_flag`
  - `remove_document_clears_orphaned_unsaved_flag`
- 既存 `unsaved_changes_state_transitions` は維持する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| `remove_document()` で削除（採用） | 将来の全呼び出しで確実に効く |
| `close_document()` のみで削除 | 他の削除経路から漏れる |
| 1 ロックへ統合 | 設計変更が大きくデッドロックリスクも生む |

## 影響

- 2 段階のロック取得のため、厳密には非原子
  （「文書は消えたが一瞬フラグが残る」窓は推測で許容）。
- non-force の `close_document()` は `is_unsaved` で早期リターンするため、
  孤立フラグを持つ存在しない文書を non-force で閉じるケースは未対応
  （未決定）。
- 新規依存なし、unsafe なし、E2E 251 維持。

## 検証（Runtime Proof）

- `cargo test -p openrustbooks-desktop`: desktop **74 passed**（旧 72 + 2）。
- `cargo clippy --all-targets --locked -- -D warnings`、
  `./scripts/verify.sh`: 成功。E2E **251 項目すべて成功**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- 2 段階ロックの非原子窓を許容するか、より強い同期が必要か。
- non-force close 時の孤立フラグ扱い
  （現状の拒否を維持／孤立なら文書不存在としてスキップ）。
- poison 時に `unwrap()` で panic した場合の挙動（既存パターンを継続）。
- E2E に desktop の close 経路を追加するか（現状は unit test のみ）。
