---
type: Decision
title: "ADR-0363: save_document の desktop UI 接続"
description: "エディタの下書きを desktop の save_document で昇格させ、保存後の競合と下書き削除結果を固定文言で扱う決定。"
tags:
  - ADR
  - ipc
  - web
generated:
  by: dsh (lead)
  at: "2026-11-28T23:14:31+09:00"
status: accepted
---

## 背景

ADR-0362 で文書を開く経路を接続したが、保存は localStorage 下書きのままだった。
段階 3 として desktop の save_document を接続し、下書きをディスクへ昇格できるようにする。

## 決定

- 保存ボタンは `.editor-toolbar` の外に置き、ツールバー子要素 5 と E2E 273 件を維持する。
- ブラウザでは disabled かつ desktop only と明示し、クリック時にも runtime を再確認する。
- IPC 引数は `{id, content}` のみ。有効 UUID 以外（生 path、`..`、placeholder を含む）は渡さない。
- クリック時に保留中の自動保存を解除し世代を進める。IPC 完了時に世代・doc_id・本文を再確認し、変化していれば下書き削除と status 更新を行わない。
- 成功時は下書きを削除し、削除失敗も固定日本語文言で伝える。code / message / path は status へ出さない。
- capability 静的ゲートの許可集合は 8 command（設定 4 + read_directory + document 3）へ拡張する。

## 検証（Runtime Proof）

- `cargo fmt --all -- --check` / `cargo check --all-targets --locked` / `cargo test --all-targets --locked`（43 passed）/
  `cargo clippy --all-targets --locked -- -D warnings` / wasm32 check / capability gate: rc=0。
- `bash scripts/verify.sh`: rc=0（コミット前に再確認する）。

## 未決定

- 保存後の `document-saved` イベントを Editor が購読して他画面へ反映する経路。

## 参照

- ADR-0127 / 0179 / 0359 / 0362: UI 接続段階と下書き契約
