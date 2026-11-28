---
type: Decision
title: "ADR-0362: FileTree から文書編集への段階 2 接続"
description: "open_document と get_document_content を FileTree / Editor へ接続し、UUID 検証・下書き優先・入力中の上書き防止を確定する決定。"
tags:
  - ADR
  - ipc
  - web
generated:
  by: dsh (lead)
  at: "2026-11-28T19:19:41+09:00"
status: accepted
---

## 背景

ADR-0359 で read_directory の一覧表示のみを接続していた。ファイル選択から文書編集へ
進む経路が未接続だったため、段階 2 として open_document / get_document_content を接続する。

## 決定

- FileTree 行はディレクトリを除き、root 相対の論理キー（名前のみ）で `open_document` を呼ぶ。
- 生 path、`..`、ユーザー名、制御文字は論理キー生成時に fail-closed で拒否する。
- 戻り値 `DocumentView.id` は UUID 検証後に `/editor/<id>` へ遷移する。直接 URL の id も Editor 側で検証し、不正値は IPC 引数と DOM へ流さない。
- Editor は IPC 本文よりローカル下書きを優先し、差分がある場合のみ固定日本語文言を status へ出す。
- IPC 往復中にユーザーが入力した場合は本文を適用しない（取得開始時の本文との比較）。これにより lost update を防ぐ。
- `check-capability-scope.sh` の許可集合は 7 command（設定 4 + read_directory + document 2）で、document 側の登録も fail-closed で検査する。

## 検証（Runtime Proof）

- `cargo fmt --all -- --check` / `cargo check --all-targets --locked` / `cargo test --all-targets --locked`（40 passed）/
  `cargo clippy --all-targets --locked -- -D warnings` / wasm32 check / capability gate: rc=0。
- `bash scripts/verify.sh`: rc=0（コミット前に再確認する）。

## 未決定

- `save_document` の UI 接続（下書きからディスク保存へ昇格）と、複数 storage root の明示選択。

## 参照

- ADR-0350: 生 path を出さない DTO 契約
- ADR-0359: 段階 1 の read_directory 接続
- ADR-0179: 800ms デバウンスと世代ガード
