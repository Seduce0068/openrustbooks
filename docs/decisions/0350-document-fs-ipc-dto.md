---
type: Decision
title: "ADR-0350: document / fs IPC の生パス除外"
description: "Document / FileEntry を IPC 境界用 View へ変換し、絶対パスとパス由来 id を JS へ出さない決定。"
tags:
  - ADR
  - IPC
  - security
generated:
  by: dsh
  at: "2026-11-23T04:25:00+09:00"
status: accepted
---

## 背景

設定 DTO は ADR-0103 でパスを除外していたが、
document / fs の返り値とイベントは `DocumentMetadata.path` / `FileEntry.path`、
さらに `open_document` の id（path 由来）をそのまま JS 境界へ渡していた。

## 決定

- desktop commands に `DocumentView` / `FileEntryView` を追加し、
  path / url / extra を除外する。
- `open_document` / `list_documents` の返り値と
  `document-opened` / `document-saved` / `document-changed` の emit を View 化する。
- `open_document` の id を `DocumentId::new()`（不透明 UUID）へ変更し、
  id からの経由でも path が漏れないようにする。
- `read_directory` / `create_file` / `rename_file` の返り値を `FileEntryView` 化する。
- `file-created` は FileEntryView、`file-deleted` / `file-renamed` は basename のみを emit する。
- core 内部の struct と保存先解決 / read_only 契約は維持する。

## 検証（Runtime Proof）

- 新規 4 test:
  - `document_view_json_never_exposes_path_or_host_details`
  - `open_document_ipc_boundary_never_exposes_raw_path`
  - `file_entry_view_json_never_exposes_raw_path`
  - `fs_ipc_views_and_events_never_expose_host_path`
- `cargo test --all-targets --locked`: core 364 / desktop 189 / web 31 = **584 passed / 0 failed**。
- `cargo fmt --all -- --check`: rc=0。
- `cargo check --all-targets --locked`: rc=0。
- `cargo clippy --all-targets --locked -- -D warnings`: rc=0。
- `./scripts/verify.sh`: 全ゲート成功、E2E **263 passed / 0 failed**、WDIO gate rc=0。

## 未決定

- frontend を document / fs IPC へ接続する際の View スキーマ確定（現状は未配線）。
