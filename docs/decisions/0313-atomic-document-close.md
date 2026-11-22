---
type: Decision
title: "ADR-0313: 文書 close の unsaved 検査と削除の原子化"
description: "unsaved チェックと文書削除を単一クリティカルセクションで行い、並行編集の喪失を防ぐ決定。"
tags:
  - ADR
  - desktop
  - concurrency
generated:
  by: dsh
  at: "2026-11-22T11:06:00+09:00"
status: accepted
---

## 背景

close_document は `is_unsaved()` と `remove_document()` を別ロックで呼んでいた。
その間に `set_document_content` が mark_unsaved すると、
force=false でも未保存編集が警告なしに削除された。

## 決定

- AppState に `close_document_atomic(id, force)` を追加する。
- `documents` → `unsaved_changes` の順で両 write guard を同時保持し、
  unsaved 検査から削除までを不可分にする。
- force=false で unsaved なら固定文言の Validation を返し、
  文書・フラグを変更しない。
- force=true は削除し、文書不存在時も孤立フラグを掃除する。
- `document-closed` の emit 条件・イベント名・回数は維持する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 単一クリティカルセクション（採用） | TOCTOU を構造的に排除できる |
| 検査を前にずらす | 競合窓が残る |
| force=true を既定にする | 未保存編集を黙って捨てる |

## 検証（Runtime Proof）

- 競合再現 test: Barrier(2) + channel で
  「旧検査後・削除前」に別スレッドの mark_unsaved を注入し、
  force=false で拒否され文書・フラグが残ることを確認。
- desktop test: **146 passed / 0 failed**。
- `cargo fmt --all -- --check`: rc=0。
- `cargo check --all-targets --locked`: rc=0。
- `cargo clippy --all-targets --locked -- -D warnings`: rc=0。
- `./scripts/verify.sh`: 全ゲート成功、E2E **256 passed / 0 failed**、WDIO gate rc=0。

## 未決定

- IPC assert を code 由来の固定文言へ直接結び付けるか。
- 旧 `remove_document` / `is_unsaved` の production 利用を完全に廃止するか。
