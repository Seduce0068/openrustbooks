---
type: Decision
title: "ADR-0190: read 上限の fail-closed 化"
description: "StorageBackend::read の要求サイズに上限を設け、超過時は黙って切り詰めずエラーにする決定を記録する。"
tags:
  - ADR
  - core
  - data-integrity
generated:
  by: dsh
  at: "2026-11-19T06:37:00+09:00"
status: accepted
---

## 背景

`ReadOptions.limit` は `u64` のままで `file.take(limit)` に渡され、
`StorageCapabilities.max_file_size` と関連付けられていなかった（R6 PR-M04）。
`limit=None` ではファイル全体を Vec に読み込み、
レスポンス直列化も含めてメモリを圧迫し得た。

## 決定

- `DEFAULT_MAX_READ_BYTES: u64 = 256 * 1024 * 1024` を追加する（暫定値、未計測）。
- pure helper `resolve_read_limit(requested, max_file_size)` を追加:
  - `effective = max_file_size.unwrap_or(DEFAULT_MAX_READ_BYTES)` を先に確定。
  - `requested = limit.unwrap_or(effective)`。
  - `requested > effective` なら `CoreError::validation` を返す（**clamp しない**）。
  - cap が既定値より大きい場合は cap を尊重する。
- `limit=None` でファイル全体を読む場合も、上限を超える場合は
  1 バイト probe で検出して `CoreError::validation` を返す。
  - 黙って部分読みすると read → edit → write でデータ損失につながるため。
- offset と limit は加算せず独立に扱う。
  offset がファイル末尾を超える場合は空 Vec（既存挙動を維持）。
- `read_text()` の lossy UTF-8 契約は維持（UTF-8 境界で切れる可能性は未決定）。
- unit test 5 件追加（None / cap Some(小) / cap Some(>DEFAULT) /
  limit=u64::MAX / offset 末尾超え / probe 検出 / read_text 伝播）。
- E2E は 249 件維持。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 上限超過を明示エラー（採用） | archive.rs と同種の fail-closed、部分読みによる損失を防ぐ |
| clamp して返す | 呼び出し元が切断を検知できずデータ損失リスク |
| 無制限のまま | メモリ圧迫リスク |

## 影響

- 256 MiB 超のファイルを `read()` / `read_text()` で読むと失敗する。
- エラー文言にはサイズ値のみを含め、内容・生パスは載せない。
- 他経路（`desktop/src/commands/document.rs:145` の
  `tokio::fs::read_to_string` 直接呼び、
  `core/src/wasm.rs:84` の `EpubBook::open_owned` 経由）は本次対象外。
  EPUB エントリは `ArchiveLimits::max_entry_size`（64 MiB）で別途管理。
- 新規依存なし、unsafe なし。

## 検証（Runtime Proof）

- `cargo test --all-targets --locked`: core **246** + desktop 62 + web 2。
- `cargo clippy --all-targets --locked -- -D warnings`、
  `./scripts/verify.sh`: 成功。E2E **249 項目すべて成功**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- 256 MiB の根拠（未計測、暫定）。
- UTF-8 境界で切れた場合の `read_text()` の扱い（現状は U+FFFD 混入）。
- `desktop/src/commands/document.rs` / `core/src/wasm.rs` の他経路へ
  同じ上限を適用するか。
- `read()` の `limit` を IPC / Tauri コマンドへ公開するか。
