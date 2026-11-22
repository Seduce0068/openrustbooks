---
type: Decision
title: "ADR-0337: save_document 競合時のディスク復元"
description: "CAS 競合時に atomic_write 済みのディスクを元内容へ戻し、state とディスクの不整合を防ぐ決定。"
tags:
  - ADR
  - desktop
  - concurrency
generated:
  by: dsh
  at: "2026-11-23T02:15:00+09:00"
status: accepted
---

## 背景

save_document は atomic_write 後に generation / full_text を再確認し、
競合なら Conflict を返していたが、ディスクには保存要求の本文が残っていた。
state は割り込み編集を保持するため、再読込や再起動後に最新編集が失われた。

## 決定

- atomic_write 直前に元バイト列を退避する。
  - read 前後で `DEFAULT_MAX_READ_BYTES` を検査する。
  - NotFound は新規ファイルとして Absent 扱い。
- 競合経路（document 消失 / generation・本文不一致 / commit_saved_document 失敗）
  で退避内容を atomic_write により元路径へ復元する。
  - Absent の場合は作成済みファイルを削除する。
- `restore_disk_on_conflict()` は `CoreResult<()>` を返し、
  失敗時は Conflict とは異なる固定文言（`CoreError::storage`）で IPC へ返す。
- 成功時は退避を破棄し、metadata / mark_saved / document-saved emit は従来どおり。
- ADR-0318 の「競合時にディスクは上書き済み」という記述は本 ADR が更新する。

## 検証（Runtime Proof）

- desktop test: **184 passed / 0 failed**。
- 注入 test:
  - `FAIL_NEXT_ATOMIC_WRITE`: 書き込み失敗で退避・state・emit 不変。
  - `FAIL_NEXT_BACKUP_METADATA`: 退避失敗で atomic_write 前に拒否。
  - `FAIL_NEXT_RESTORE_WRITE`: 復元失敗を別固定文言で返す。
- `cargo fmt --all -- --check`: rc=0。
- `cargo check --all-targets --locked`: rc=0。
- `cargo clippy --all-targets --locked -- -D warnings`: rc=0。
- `./scripts/verify.sh`: 全ゲート成功、E2E **263 passed / 0 failed**、WDIO gate rc=0。

## 未決定

- 退避 `tokio::fs::read` 本体の失敗注入（metadata 失敗とは同じく ? で早期 return）。
- 成功時に退避バイト列を emit 前に明示 dropするか。
