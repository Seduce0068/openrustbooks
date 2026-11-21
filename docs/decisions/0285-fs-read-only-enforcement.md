---
type: Decision
title: "ADR-0285: desktop fs コマンドの read_only 強制"
description: "local_roots[].read_only を save_document だけでなく fs コマンドにも適用し、入れ子 root と symlink の迂回を塞ぐ決定。"
tags:
  - ADR
  - security
  - desktop
generated:
  by: dsh
  at: "2026-11-22T02:07:00+09:00"
status: accepted
---

## 背景

create_file / delete_file / rename_file は LocalStorage を生成する際に
local_roots[].read_only を参照しておらず、save_document のガードだけが
read-only root を守っていた。描画側 JS から fs コマンドを直接 invoke すると、
read-only と設定された root を書き換えられる。
さらに writable 親 root と read-only 子 root が入れ子の場合、
未作成パスを設定順先頭 root に解決すると子 root を迂回でき、
symlink 経由でも同様に迂回できた。

## 決定

- `READ_ONLY_ROOT_MESSAGE` を core の共有定数にする。
- `LocalStorage` が `read_only` を保持し、write / create_dir / delete / move_item
  の入口で fail-closed に拒否する。read_only root は initialize() で
  ディレクトリを作成しない。
- desktop の `resolve_target_storage` は以下を満たす:
  - 既存パスは canonical target を使う。
  - canonical target が別の設定 root に属する場合もその root を候補に含める。
  - 候補に read_only root が一つでもあれば固定文言で拒否する。
  - 最深 root を選択し、解決済み絶対パスを実際の IO に使う。
- `create_file` / `delete_file` / `rename_file` は解決済み target を使う。
- command レベル test で、拒否時にファイル・親ディレクトリ・emit が
  変化しないことを検証する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| root 解決時に read_only を検査（採用） | 全 fs コマンドへ一貫して適用できる |
| save_document のみ検査 | 迂回可能で却下 |
| root指定を IPC に追加 | API 変更が必要で今回は見送り |

## 影響

- 未作成の相対パスは、設定内に read_only root が一つでもあると
  fail-closed で拒否される。writable root が存在しても相対パスでは
  一意に特定できないためである。
- read_directory は従来どおり先頭 root を使用する（読み取りは許可）。
- read-only root を指す symlink 自身の delete / rename も拒否される
  （安全側の fail-closed）。
- `desktop/Cargo.toml` の dev-dependencies に Tauri `test` feature を追加し、
  `mock_app()` で command レベル test を実行する。

## 検証（Runtime Proof）

- `cargo test -p openrustbooks-desktop --all-targets --locked`:
  **103 passed / 0 failed**。
- `cargo test --all-targets --locked`: 296 + 103 + 2 passed / 0 failed。
- `cargo fmt --all -- --check`: rc=0。
- `cargo clippy --all-targets --locked -- -D warnings`: rc=0。
- `./scripts/verify.sh`: 全ゲート成功、E2E **256 passed / 0 failed**、
  WDIO gate rc=0。

## 未決定

- 相対パスの新規作成で対象 root を明示する IPC 拡張。
- read_directory のマルチ root 対応。
- symlink 自身の delete / rename を許容するか。
