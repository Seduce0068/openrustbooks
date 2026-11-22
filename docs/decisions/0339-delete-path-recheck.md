---
type: Decision
title: "ADR-0339: delete 直前のパス再確認"
description: "LocalStorage::delete が削除直前に resolve_path を再実行し、symlink 差し替えを fail-closed で拒否する決定。"
tags:
  - ADR
  - storage
  - security
generated:
  by: dsh
  at: "2026-11-23T02:30:00+09:00"
status: accepted
---

## 背景

read / write / move_item は `ensure_path_unchanged()` で解決直後の差し替えを検出していたが、
`delete` は resolve_path 後すぐに `remove_file` / `remove_dir_all` を実行していた。
symlink 差し替え等で解決先が変わっても検出できなかった。

## 決定

- `LocalStorage::delete` は `resolve_path` と `ensure_root_not_target` の後、
  削除直前に `ensure_path_unchanged(path, &full_path)` を呼ぶ。
- 不一致は `STORAGE_PATH_CHANGED_MESSAGE` の `CoreError::Storage` で拒否する。
- これは ADR-0286 の「解決先再確認」方針を delete 経路へ適用するもの。

## 検証（Runtime Proof）

- `ensure_path_unchanged` の symlink 差し替え test（`recheck_detects_symlink_swap_before_open`）が既存。
- `cargo fmt --all -- --check`: rc=0。
- `./scripts/verify.sh`: 全ゲート成功、E2E **263 passed / 0 failed**、WDIO gate rc=0。

## 未決定

- `O_NOFOLLOW` / `openat2(RESOLVE_BENEATH)` 相当の safe API による完全防止（ADR-0286 と共通）。
