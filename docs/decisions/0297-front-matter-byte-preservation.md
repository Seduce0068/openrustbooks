---
type: Decision
title: "ADR-0297: Markdown front matter の元バイト保持"
description: "front matter を構造化値と元テキストの両方で保持し、無編集 open→save でバイト列が変わらないようにする決定。"
tags:
  - ADR
  - document
  - markdown
generated:
  by: dsh
  at: "2026-11-22T05:52:00+09:00"
status: accepted
---

## 背景

extract_front_matter は `lines()` / `join("\n")` で CRLF を LF に変換し、
full_text は serde_norway で YAML を再シリアライズしていた。
そのため編集していない Markdown を開いて保存するだけで、
改行コード・末尾改行・YAML コメント・クォート・キー順が変わった。

## 決定

- DocumentContent::Markdown に `front_matter_raw: Option<String>` を追加する。
- `extract_front_matter_preserving` を追加し、
  区切り線・改行スタイル・末尾改行を含む元テキストを保持する。
- desktop の `open_document` は本番経路でも preserving 版を使う。
- `full_text()` は raw が存在する場合に構造化値を再シリアライズせず、
  raw + body を返す。
- 構造化 front_matter は lint / AI の既存利用のため維持する。
- 不正 YAML / 区切り線のみ / 連続区切り / 空 body / BOM では、
  fail-closed で入力原文を返す。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 元テキストも保持して再利用（採用） | バイト列を完全に保持できる |
| serde_norway で常に再構築 | コメント・書式が失われるため却下 |
| 構造化値のみ保持 | 既存不具合が残るため却下 |

## 検証（Runtime Proof）

- core test: **327 passed / 0 failed**。
- desktop test: **115 passed / 0 failed**。
- `cargo check --all-targets --locked`: rc=0。
- `cargo clippy --all-targets --locked -- -D warnings`: rc=0。
- 往復 test: CRLF / 末尾改行 / YAML コメント / キー順 / クォート /
  末尾改行なし / 不正 YAML / BOM を `assert_eq!` で固定。
- `./scripts/verify.sh`: 全ゲート成功、E2E **256 passed / 0 failed**、WDIO gate rc=0。

## 未決定

- 構造化 front_matter を更新する API を追加する場合、
  raw を無効化／再生成する契約が必要。
- front_matter_raw が無い旧データの移行方針。
- save_document の content が body 専用であることを API 契約へ明記するか。
