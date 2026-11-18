---
type: Decision
title: "ADR-0173: WriteOptions.overwrite の実効化"
description: "overwrite=false のとき既存ファイルへの書き込みを拒否し、黙って上書きされる問題を解消する決定を記録する。"
tags:
  - ADR
  - core
  - security
generated:
  by: dsh
  at: "2026-11-19T01:55:00+09:00"
status: accepted
---

## 背景

`LocalStorage::write()` は `WriteOptions.overwrite` を参照せず、
常に `tokio_fs::write` で既存ファイルを上書きしていた。
`desktop/src/commands/fs.rs::create_file()` は
`WriteOptions::default()`（overwrite=false）を渡すため、
「新規作成」操作が既存データを破壊し得た（R6: PR-H01 / TD-H01）。

## 決定

- `resolve_path()` 直後、`create_parents` と `atomic` 分岐より前に
  `if !options.overwrite && full_path.exists()` を判定する。
- 拒否時は `CoreError::Storage` を返す。
- エラー文言にはファイル内容も呼び出し元の生パスも載せず、
  正規化済み root からの相対パスのみを表示する
  （`"File already exists: <relative>"`、正規化不能なら固定文言
  `"File already exists"`）。
- `create_file()` は `WriteOptions::default()` のまま。
  新規作成のみを許可する契約が今回のガードで実現される。
- 新規／既存 × overwrite true/false × atomic true/false の
  分岐を unit test で固定する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| `exists()` で事前拒否（採用） | 最小変更で既存破壊を防げる |
| `OpenOptions::create_new` | OS 原子性は高いが atomic 分岐と两套になる（残課題） |
| 呼び出し側で確認 | Tauri 経由の全経路を保証できない |

## 影響

- 既存ファイルへ `overwrite=false` で書き込むと失敗する（意図した挙動変更）。
- `overwrite=true` は従来どおり上書きを許可。
- TOCTOU（判定直後の並行作成）と固定 tmp 名の衝突は残る
  （単一プロセス前提のベストエフォート、TODO TD-M06 として追跡）。
- 新規依存なし、unsafe なし。

## 検証（Runtime Proof）

- 新規 unit tests:
  - 既存拒否（非 atomic / atomic）
  - 上書き成功（非 atomic / atomic）
  - 新規作成成功（`create_parents` 含む）
  - エラー文言に元内容・生パスを含まない
- `cargo test --all-targets --locked`: core **208** + desktop 56 + web 2。
- wasm-test 10、`cargo audit` 脆弱性 0 件。
- `./scripts/verify.sh` rc=0、E2E **246 項目すべて成功**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- TOCTOU を排除するための `create_new(true)` ベースの実装
  （非 atomic 分岐、temp 名の一意化と合わせて検討）。
- `move_item()` の overwrite 規約との統一（remove → rename の非原子性）。
- 既存ファイル時の UI 確認／リネーム導線
  （`create_file` は失敗として表示される）。
