---
type: Decision
title: "ADR-0298: 文書保存の atomic 化"
description: "save_document を temp + rename 方式へ変更し、書き込み失敗時に既存ファイルを保持する決定。"
tags:
  - ADR
  - desktop
  - document
generated:
  by: dsh
  at: "2026-11-22T06:03:00+09:00"
status: accepted
---

## 背景

save_document は `tokio::fs::write` で既存ファイルを truncate してから
書き込んでいた。書き込み途中エラーで元ファイルが空／部分内容のまま残り、
利用者の文書が失われる危険があった。

## 決定

- 同一ディレクトリに `<file>.<pid>.<counter>.tmp` を `create_new` で作成する。
- `write_all` → `flush` → `sync_all` → `rename` の順で保存する。
- 失敗時は一時ファイルを削除し、元ファイル / metadata / mark_saved / emit を
  一切変更しない。
- Windows 等で rename が既存ファイルを置換できない場合は、
  直接書き込みへフォールバックせず fail-closed とする。
- read_only / containment / front matter 原文保全は維持する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| temp + rename（採用） | 失敗時に元ファイルを保持できる |
| 直接 write の継続 | 部分書き込みでデータ損失するため却下 |
| 既存 core::io::write_atomic を公開 | 今回は desktop 境界を広げないため見送り |

## 検証（Runtime Proof）

- desktop test: **121 passed / 0 failed**（新規 6 件含む）。
- `cargo fmt --all -- --check`: rc=0。
- `cargo check --all-targets --locked`: rc=0。
- `cargo clippy --all-targets --locked -- -D warnings`: rc=0。
- `./scripts/verify.sh`: 全ゲート成功、E2E **256 passed / 0 failed**、WDIO gate rc=0。

## 未決定

- rename 後の親ディレクトリ fsync によるクラッシュ耐久性。
- 一時ファイル削除失敗のログ出力と起動時掃除。
- 親ディレクトリ未作成パスへの保存意味論の統一。
- 往復テストを save_document_impl 経由へ一本化するか。
