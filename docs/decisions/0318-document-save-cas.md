---
type: Decision
title: "ADR-0318: save_document の世代 CAS"
description: "保存と並行編集を世代 CAS で検出し、古い snapshot による state の書き戻しを防ぐ決定。"
tags:
  - ADR
  - desktop
  - concurrency
generated:
  by: dsh
  at: "2026-11-22T13:01:00+09:00"
status: accepted
---

## 背景

save_document_impl は get_document で clone し、atomic_write().await 中に
set_document_content が新しい本文を格納しても、完了後に古い snapshot を
state.add_document へ書き戻し mark_saved するため最新本文が失われた。

## 決定

- AppState に文書単位の世代カウンタを追加する。
- 保存は開始時に document + generation を取得し、
  atomic_write().await 後に generation / full_text を再確認する。
- 競合なら固定文言の Conflict を返し、
  古い snapshot の state 書き戻し・mark_saved・emit を行わない。
- `commit_saved_document` は `documents.contains_key()` と generation を
  同じ write guard 内で確認し、成功時にのみ文書差し替え・世代前進・
  unsaved クリアを行う。
- `close_document_atomic` は削除成功時に世代を前進させ、
  再確認後に force close が入っても文書が復活しないようにする。
- ロック順は documents → generations → unsaved に統一する。

## 競合時のディスク契約

atomic_write は CAS より前に実行されるため、
競合 detection 時点でディスクには保存要求側の本文が書き込まれている。
state 上は割り込み編集（unsaved=true）が権威であり、
両者は一時的に対象文書について乖離する。
現状はこの挙動を受容し、UI 側で Conflict を再保存の促しとして扱う。
ディスクの完全復元は未実装（未決定）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 世代 CAS（採用） | state のデータ破壊を確実に防げる |
| 保存ロックで直列化 | await を長く保持しUXを阻害する |
| 何もしない | 最新本文が失われる |

## 検証（Runtime Proof）

- desktop test: **157 passed / 0 failed**。
- 競合 test: Barrier で await 後に割り込み編集 / force close を注入。
- 失敗・read_only・symlink 拒否で generation 不変を確認。
- `cargo fmt --all -- --check`: rc=0。
- `cargo check --all-targets --locked`: rc=0。
- `cargo clippy --all-targets --locked -- -D warnings`: rc=0。
- `./scripts/verify.sh`: 全ゲート成功、E2E **256 passed / 0 failed**、WDIO gate rc=0。

## 未決定

- 競合時にディスク内容を元へ復元する仕組み。
- Conflict を UI で専用文言へ変換するか（現状は "Configuration conflict"）。
