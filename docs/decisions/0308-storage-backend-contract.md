---
type: Decision
title: "ADR-0308: StorageBackend の競合安全性と未実装契約"
description: "overwrite の原子的新規作成、resolve_path 再確認、watch 未実装契約、退避後始末を明確化する決定。"
tags:
  - ADR
  - storage
  - concurrency
generated:
  by: dsh
  at: "2026-11-22T09:32:00+09:00"
status: accepted
---

## 背景

StorageBackend の write(overwrite=false) は exists チェック後に
truncate/rename するため、競合で既存ファイルを上書きできた。
canonicalize 後に open/rename までの間にパスが差し替わる可能性もある。
watch は stub のまま trait に残り、契約が不明だった。

## 決定

- write(overwrite=false) は `OpenOptions::create_new(true)` で原子的に新規作成する。
- atomic + overwrite=false は一時ファイルを `hard_link` で配置し、
  rename による暗黙上書きを回避する（同一 filesystem 前提）。
- `ensure_path_unchanged()` を追加し、
  read / write / move の open / rename 直前に resolve_path を再実行して
  不一致なら `STORAGE_PATH_CHANGED_MESSAGE` で拒否する。
- move(overwrite=false) の check→rename 窓は
  std に RENAME_NOREPLACE が無く unsafe FFI が禁止のため、
  二重チェックを `ensure_destination_absent()` に集約して維持する。
- watch は ADR-0178 に従い trait に残すが、
  doc comment に「全 backend 未実装・can_watch=false なら呼ぶな」を明記する。
- move の退避先削除は `remove_path` に統一する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| create_new / hard_link + 再解決（採用） | safe API の範囲で競合を減らせる |
| unsafe FFI で renameat2 | プロジェクト規約で禁止 |
| watch を削除 | ADR-0178 の型残存決定に反する |

## 検証（Runtime Proof）

- core 346 / desktop 128 / web 9 tests、すべて成功。
- `cargo fmt --all -- --check`: rc=0。
- `cargo check --all-targets --locked`: rc=0。
- `cargo clippy --all-targets --locked -- -D warnings`: rc=0。
- wasm core check: rc=0。
- `./scripts/verify.sh`: 全ゲート成功、E2E **256 passed / 0 failed**、WDIO gate rc=0。

## 未決定

- O_NOFOLLOW / openat2(RESOLVE_BENEATH) 相当の safe API が standard に追加された場合の移行。
- hard_link 非対応 filesystem の扱い（現状は fail-closed）。
- watch を別 trait へ分離するか（ADR-0178 の再評価）。
