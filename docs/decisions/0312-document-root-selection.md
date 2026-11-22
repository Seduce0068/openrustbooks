---
type: Decision
title: "ADR-0312: save_document の root 選択を read_only fail-closed 化"
description: "全候補 root を検査し、入れ子の read-only root を save_document から書き換えられないようにする決定。"
tags:
  - ADR
  - desktop
  - security
generated:
  by: dsh
  at: "2026-11-22T10:53:00+09:00"
status: accepted
---

## 背景

resolve_within_roots は最初に一致した root を返していたため、
writable 親 + read-only 子の入れ子構成で子 root の read_only が無視された。
fs.rs の resolve_target_storage は全候補検査・最深一致を行っており、
文書保存だけが旧方式のままだった。

## 決定

- resolve_within_roots を全候補方式へ変更する。
- 候補に read_only root が一つでもあれば
  `READ_ONLY_ROOT_MESSAGE` で拒否する。
- 最深 root（component 数）を選択し、解決済み path を IO へ使う。
- canonical target が別の設定 root に属する場合も候補へ追加し、
  symlink 経由の read-only 迂回を塞ぐ。
- save_document_impl の既存 read_only チェックは二重ガードとして維持する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 全候補検査 + 最深選択（採用） | fs コマンドと契約を統一できる |
| 先頭一致のまま | 設定順で read-only が迂回される |
| resolve を削除 | containment を失う |

## 検証（Runtime Proof）

- 新規回帰 test 8 件（入れ子両順 / 相対 + 絶対 / root 外 / symlink / command レベル）。
- desktop test: **136 passed / 0 failed**。
- `cargo fmt --all -- --check`: rc=0。
- `cargo check --all-targets --locked`: rc=0。
- `cargo clippy --all-targets --locked -- -D warnings`: rc=0。
- `./scripts/verify.sh`: 全ゲート成功、E2E **256 passed / 0 failed**、WDIO gate rc=0。

## 未決定

- resolve_within_roots と fs.rs::resolve_target_storage の完全な共通化。
- command レベル emit 不変 test の更なる一般化。
