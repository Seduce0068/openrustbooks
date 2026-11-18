---
type: Decision
title: "ADR-0178: watch capability の誠実化"
description: "未実装の watch を利用可能と報告せず、到達不能な IPC も削除する決定を記録する。"
tags:
  - ADR
  - core
  - desktop
generated:
  by: dsh
  at: "2026-11-19T03:20:00+09:00"
status: accepted
---

## 背景

`LocalStorage::new()` は `can_watch: true` を報告していたが、
`watch()` 本体は `"Watch not implemented yet"` を返す未実装スタブだった。
`watch_directory` コマンドも `invoke_handler` に登録されており、
呼び出すと常に失敗する（R6: TD-H04）。

監査で次を確認:
- フロントエンドからの `watch_directory` / `fs-event` 参照は 0 件。
- Tauri capability は `{}` で独自コマンドは ACL 非対象。
- 既存の失败的 API を残すより、宣言と IPC 表面を一致させる方が安全。

## 決定

- `LocalStorage::new()` の `can_watch` を `false` に変更
  （既定値も false）。
- `desktop/src/main.rs` の `invoke_handler` から `watch_directory` を削除。
- `desktop/src/commands/fs.rs` の `watch_directory` 関数自体を削除
  （lint 抑制属性も残さない）。
- `StorageBackend::watch` / `StorageWatchStream` / `StorageEvent` 型は
  将来実装のため残す。
- unit test を 2 件追加:
  - `LocalStorage::new().capabilities().can_watch == false`
  - `watch()` が `CoreError::Storage("Watch not implemented yet")` を返す。
- `watch()` の Err 文言は今回固定するが、将来実装時にテストを更新する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| capability を false にし、IPC も削除（採用） | 宣言・API・実装が一致する |
| スタブ IPC を残す | `can_watch=false` なのに呼べる不整合が残る |
| notify 等で実装 | 新規依存追加で本次範囲外 |

## 影響

- `watch_directory` を直接呼ぶ外部クライアントは
  "command not found" 系エラーになる（**推測**: Tauri の名前解決仕様、
  実機未確認）。
- 型・トレイトは残るため将来実装の妨げにならない。
- 再実装手順: `watch()` 実装 → `can_watch=true` 復帰 →
  コマンド再登録 → テスト更新（未決定）。
- 新規依存なし、unsafe なし。

## 検証（Runtime Proof）

- `cargo test --all-targets --locked`: core **212** + desktop 59 + web 2。
- wasm-test 10、audit 脆弱性 0 件。
- `cargo clippy --all-targets --locked -- -D warnings`、
  `./scripts/verify.sh`: 成功。E2E **247 項目すべて成功**。
- `grep -rn 'watch_directory' desktop/src` は登録しない旨のコメントのみ。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- `watch` を実際に実装するか、 nécessité な依存（notify 等）の選定。
- wasm 側の watch 扱い（native only、io.rs:248）。
- capability.json `{}` の実挙動（**推測**、実ビルド未確認）。
