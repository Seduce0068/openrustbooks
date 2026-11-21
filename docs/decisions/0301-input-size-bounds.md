---
type: Decision
title: "ADR-0301: ユーザー入力サイズの fail-closed 境界"
description: "open / import 前に 256 MiB 上限を検査し、読み取り後にも実バイト長を再確認する決定。"
tags:
  - ADR
  - security
  - web
generated:
  by: dsh
  at: "2026-11-22T06:55:00+09:00"
status: accepted
---

## 背景

desktop `open_document` はファイルを無制限に `read_to_string` し、
web の JSON import は `file.text()` 前にサイズを検査していなかった。
巨大入力はメモリ消費／UI フリーズの原因になり得る。

## 決定

- 上限は core の `DEFAULT_MAX_READ_BYTES`（256 MiB）を単一情報源とする。
- desktop は `metadata.len()` で読み取り前に拒否し、
  `read_to_string` 後にも実バイト長を再検査する。
- web は `file.size()` で `file.text()` / `array_buffer()` 前に拒否し、
  読み取り後にも実バイト長を再検査する。
- EPUB / JSON import の上限も同じ core 定数から導出する。
- エラー文言にパス / 実サイズ / ファイル名は出さない。
- 既存 EPUB 拒否文言の「上限 256 MiB」表示は維持する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 事前 + 事後の二重検査（採用） | TOCTOU でのサイズ拡大も拒否できる |
| 事前のみ | 検査後に拡大した場合を許す |
| 事後のみ | 読み込み自体のメモリ消費を防げない |

## 検証（Runtime Proof）

- core 327 / desktop 126 / web 6 tests、すべて成功。
- `cargo fmt --all -- --check`: rc=0。
- `cargo check --all-targets --locked`: rc=0。
- `cargo clippy --all-targets --locked -- -D warnings`: rc=0。
- `cargo check -p openrustbooks-core --target wasm32-unknown-unknown --locked`: rc=0。
- `cargo check -p openrustbooks-web --target wasm32-unknown-unknown --locked`: rc=0。
- `npx tsc --noEmit`: rc=0。
- `./scripts/verify.sh`: 全ゲート成功、E2E **256 passed / 0 failed**、WDIO gate rc=0。

## 未決定

- サイズ超過を専用 IPC code で UI に区別して伝えるか。
- EPUB 拒否文言の上限値表示を固定文言へ変更するか。
