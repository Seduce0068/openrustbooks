---
type: Decision
title: "ADR-0334: set_document_content の原子化"
description: "本文更新・世代前進・unsaved 設定を単一クリティカルセクションで行い、close との競合を防ぐ決定。"
tags:
  - ADR
  - desktop
  - concurrency
generated:
  by: dsh
  at: "2026-11-23T01:41:00+09:00"
status: accepted
---

## 背景

set_document_content_impl は get_document → add_document → mark_unsaved を
別クリティカルセクションで実行していた。
add_document が documents guard を解放した隙間に force=false の close が走ると、
未保存の編集が削除されるか孤立フラグが残った。

## 決定

- AppState に `update_document_content_atomic(id, content)` を追加する。
- documents → generations → unsaved の順で 3 つの write guard を同時保持し、
  本文更新・世代前進（saturating_add）・unsaved=true を不可分に行う。
- 未登録文書は固定文言の DocumentNotFound、編集不能は Validation で拒否する。
- emit は全 guard 解放後に document-changed を 1 回だけ行う。
- 並行する content 更新は世代を両方前進させるが、
  本文は last-write-wins（最後の書き込みが残る）を意図仕様とする。
  両編集のマージは行わない。
- ADR-0313 / ADR-0318 の close / save CAS 契約は維持する。

## 検証（Runtime Proof）

- desktop test: **176 passed / 0 failed**（新規 6 件含む）。
- コマンド経由 close 競合 test: Barrier + channel で
  編集投入到与直後に close を割り込み、unsaved 拒否 / 本文残存 / emit 0 を検証。
- `cargo fmt --all -- --check`: rc=0。
- `cargo check --all-targets --locked`: rc=0。
- `cargo clippy --all-targets --locked -- -D warnings`: rc=0。
- `./scripts/verify.sh`: 全ゲート成功、E2E **263 passed / 0 failed**、WDIO gate rc=0。

## 未決定

- 並行編集の本文をマージして保持する仕組み（現状は last-write-wins）。
