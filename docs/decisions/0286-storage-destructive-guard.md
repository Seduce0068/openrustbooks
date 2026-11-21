---
type: Decision
title: "ADR-0286: storage root の破壊的操作と move 上書きの fail-closed 化"
description: "storage root 自身の削除・改名を拒否し、move_item の上書き競合で既存データを保持する決定。"
tags:
  - ADR
  - storage
  - security
generated:
  by: dsh
  at: "2026-11-22T02:30:00+09:00"
status: accepted
---

## 背景

`resolve_path('.')` は canonical root を返すため、
`delete_file(path='.', recursive=true)` が root 配下全体を削除できた。
`move_item` は overwrite=false でも POSIX `rename` が既存宛先を暗黙上書きし、
overwrite=true は「宛先削除 → rename」で途中失敗時に元と宛先の両方を失う危険があった。

## 決定

- `STORAGE_ROOT_OPERATION_MESSAGE` / `MOVE_DESTINATION_EXISTS_MESSAGE` /
  `MOVE_ROLLBACK_FAILED_MESSAGE` をパスレス固定文言として追加する。
- `delete` と `move_item` は、解決先が canonical root と等しい場合に拒否する。
- `create_dir(root, recursive=true)` は root 直下への新規作成を壊さないため、
  idempotent に成功させる（拒否対象は delete / move に限定）。
- `move_item(overwrite=false)` は rename 直前に宛先の存在を再確認し、
  既存なら固定文言で拒否して元・宛先を保持する。
- `move_item(overwrite=true)` は退避先を同種別で予約し、
  「退避 → rename → 成功時削除 / 失敗時復元」へ変更する。
- desktop の `resolve_target_storage` も root 自身を専用固定文言で拒否する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| root 拒否 + 退避付き move（採用） | 既存データを保持しつつ安全に上書きできる |
| create_dir(root) も拒否 | root 直下の新規ファイル作成が壊れるため却下 |
| unsafe FFI で renameat2(NOREPLACE) | プロジェクト規約で unsafe が禁止のため却下 |

## 影響

- overwrite=false の TOCTOU は std に safe API が無いため、
  「rename 直前の再確認」が上限。完全な原子的 no-replace は未決定。
- root 自身への symlink 別名も canonical 比較で拒否される。
- 新規固定文言にはパス・設定値・秘密情報を含めない。
- 既存の io error 文字列にパスが出る契約は今回の範囲外。

## 検証（Runtime Proof）

- `cargo test -p openrustbooks-core --locked`: **302 passed / 0 failed**。
- `cargo test -p openrustbooks-desktop --locked`: **107 passed / 0 failed**。
- `cargo fmt --all -- --check`: rc=0。
- `cargo clippy --all-targets --locked -- -D warnings`: rc=0。
- `./scripts/verify.sh`: 全ゲート成功、E2E **256 passed / 0 failed**、WDIO gate rc=0。

## 未決定

- safe crate による `renameat2(RENAME_NOREPLACE)` / `RENAME_EXCHANGE` の利用。
- 既存 io error 文字列からのパス削除（ADR-0103 との整合）。
- root 自身への create_dir を明示的に拒否する API を別途設けるか。
