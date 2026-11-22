---
type: Decision
title: "ADR-0324: TableAlignment の fail-closed 検証"
description: "未知の alignment を Left へ黙ってフォールバックせず、TableFormat と同じ validation で拒否する決定。"
tags:
  - ADR
  - desktop
  - AI
generated:
  by: dsh
  at: "2026-11-22T21:20:00+09:00"
status: accepted
---

## 背景

parse_table_format は未知値を CONFIG_VALIDATION で拒否していたが、
parse_table_alignment は未知値を TableAlignment::Left へ黙ってフォールバックしていた。
クライアントの誤記が要求と異なる表を生成しても通知されなかった。

## 決定

- `parse_table_alignment` を `Result<Option<Vec<TableAlignment>>, String>` に変更する。
- 未知値は `CoreError::validation` → `ipc_error` の固定文言で拒否する。
- None は `Ok(None)`、left / center / right は大小文字無視で従来どおり成功する。
- `convert_to_table_impl` で parse を confirm 前に移動し、
  parse_table_format と同じ入力検証タイミングへ揃える。
- 単体 test で未知値拒否、既知値成功、confirm 前に拒否されることを固定する。
- ADR-0319 の「alignment は confirm 後」という記述は本 ADR が更新する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| Result 化して拒否（採用） | format と方針が揃い誤記を通知できる |
| Left フォールバック継続 | 要求と異なる結果が黙って通る |
| デシリアライズ型で検証 | 引数型変更が大きい |

## 付随修正

- wasm-pack は `wasm-bindgen` と同じディレクトリに
  `wasm-bindgen-test-runner` を期待するため、
  `scripts/ci-wasm-test.sh` で Cargo.lock と同版（0.2.129）の
  runner を wasm-pack キャッシュから PATH へ追加する。
- これにより `./scripts/verify.sh` の wasm-test ゲートが環境依存で失敗しなくなる。

## 検証（Runtime Proof）

- desktop test: **167 passed / 0 failed**。
- `cargo fmt --all -- --check`: rc=0。
- `cargo check --all-targets --locked`: rc=0。
- `cargo clippy --all-targets --locked -- -D warnings`: rc=0。
- `./scripts/verify.sh`: 全ゲート成功、wasm-test rc=0、
  E2E **256 passed / 0 failed**、WDIO gate rc=0。

## 未決定

- alignment 値を型レベル Deserialize で検証するか。
