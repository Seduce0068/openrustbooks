---
type: Decision
title: "ADR-0188: atomic write の一時ファイル名一意化"
description: "atomic 書き込みの tmp 名に PID とカウンタを付与し、並行書き込みの衝突を防ぐ決定を記録する。"
tags:
  - ADR
  - core
  - data-integrity
generated:
  by: dsh
  at: "2026-11-19T06:05:00+09:00"
status: accepted
---

## 背景

`LocalStorage::write()` の atomic 分岐は
`full_path.with_extension("tmp")` という固定名を使っていた（R6 PR-M06）。
同一ファイルへの並行書き込みや既存の `.tmp` ファイルと衝突し、
片方のデータが他方の rename で上書きされる可能性があった。

## 決定

- `static ATOMIC_WRITE_ID: AtomicUsize`（Relaxed `fetch_add`）を追加する
  （native `local` モジュール内、wasm には非影響）。
- tmp 名を `<元ファイル名>.<pid>.<counter>.tmp`
  （例: `note.txt.12345.0.tmp`）にする。
  - 元ファイルと同一ディレクトリに置き、rename の atomicity を維持。
  - 既存の `.tmp` を上書きしない（stale ファイルの所有権を奪わない）。
- `OpenOptions::new().write(true).create_new(true).open()` を使い、
  `AlreadyExists` ならカウンタを進めて最大 128 回再試行する。
- `write_all()` → `flush()` → `drop(file)` → `rename()` の順で実行し、
  書き込み／rename 失敗時は tmp を best-effort で削除する。
- `!options.overwrite && exists` ガード（ADR-0173）と
  `create_parents` は維持。
- unit test 4 件追加（一意性、並行作成、stale tmp 非上書き、
  並行 write で内容混在なし＆残 tmp なし）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| PID + counter + create_new（採用） | 同一／他プロセスの衝突を実用上防ぐ |
| 固定 tmp 名のまま | 並行書き込みでデータ混在のリスク |
| 乱数（rand 等） | 新規依存追加が必要 |

## 影響

- 同一プロセス内は counter、他プロセスは PID で衝突を回避。
- 共有 FS で PID 空間が重なる环境下でも create_new の再試行で回避（推測）。
- 旧固定名の `.tmp` は今後も生成されず、既存の stale ファイルは触らない。
- `sync_all()` は未使用（durability と性能のトレードオフは未決定）。
- 新規依存なし、unsafe なし。

## 検証（Runtime Proof）

- `cargo test -p openrustbooks-core --locked`: core **228**
  （新規 4 件、旧 224 から拡充）。
- `cargo clippy --all-targets --locked -- -D warnings`、
  wasm32 core check、`./scripts/verify.sh`: 成功。
- E2E **249 項目すべて成功**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- `sync_all()` の導入（クラッシュ時の durability 保証）。
- 起動時の残 tmp クリーンアップ方針（kill -9 / 電源断時）。
- 長いファイル名で ENAMETOOLONG になる場合のフォールバック。
- `full_path.exists()` と `rename()` 間の TOCTOU 強化
  （overwrite=false でも final を上書きし得る）。
- tmp ファイルのパーミッション（umask 依存、0600 にするか）。
- counter の 2^64 周回（実用上非問題の明記）。
