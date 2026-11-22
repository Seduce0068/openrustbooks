---
type: Decision
title: "ADR-0321: read_directory の全 root 解決"
description: "ディレクトリ一覧を先頭 storage 固定ではなく、書き込み系と同じ最深一致 root へ解決する決定。"
tags:
  - ADR
  - desktop
  - storage
generated:
  by: dsh
  at: "2026-11-22T14:06:00+09:00"
status: accepted
---

## 背景

read_directory は storages.first() 固定で一覧取得していた。
複数 local_roots 構成で 2 番目以降の root を絶対パス指定すると
"Path escapes storage root" で拒否され、相対パスは先頭 root に解決された。

## 決定

- 共通ヘルパー（root canonical 化 / `..` 拒否 / 候補収集 / 最深選択）を抽出する。
- `resolve_read_target` を追加し、read_only 拒否条件のみ外す。
- read_directory は resolve_read_target で解決した storage / path を list() へ渡す。
- 絶対 / 相対 / symlink 経由の canonical target と、別設定 root の候補化は
  書き込み系と同じ意味論にする。
- root 自身（`.` / 空文字 / 絶対 root / symlink 経由）は一覧対象として受理する。
- 書き込み系の `resolve_target_storage` は read_only fail-closed を維持する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| read 専用 resolver（採用） | 読み取り許可と書き込み拒否を分離できる |
| 書き込み系 resolver を流用 | read_only root を一覧できない |
| 先頭固定継続 | 複数 root 構成が壊れる |

## 検証（Runtime Proof）

- 新規 test 7 件（2 番目 root / 入れ子両順 / 絶対 + 相対 / read_only root /
  root 自身 / root 外 + `..` / symlink）。
- desktop test: **164 passed / 0 failed**。
- `cargo fmt --all -- --check`: rc=0。
- `cargo check --all-targets --locked`: rc=0。
- `cargo clippy --all-targets --locked -- -D warnings`: rc=0。
- `./scripts/verify.sh`: 全ゲート成功、E2E **256 passed / 0 failed**、WDIO gate rc=0。

## 未決定

- 同深度の複数 root に対する相対パスの選択（現状は設定順先頭）。
