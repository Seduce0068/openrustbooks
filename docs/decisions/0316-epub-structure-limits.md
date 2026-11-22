---
type: Decision
title: "ADR-0316: EPUB 構造要素数の上限"
description: "OPF / NCX の構造要素蓄積時に上限を適用し、細工 EPUB によるメモリ枯渇を fail-closed で拒否する決定。"
tags:
  - ADR
  - EPUB
  - security
generated:
  by: dsh
  at: "2026-11-22T12:11:00+09:00"
status: accepted
---

## 背景

ZIP エントリ数とサイズには上限があったが、
1 つの OPF / NCX 文書内の `<item>` / `<itemref>` / メタデータキャプチャ /
`<navPoint>` は無制限だった。64 MiB の文書に数百万要素を詰めることで、
アーカイブ展開予算内でもヒープを枯渇させられる。

## 決定

- `MAX_STRUCTURE_ENTRIES = 10_000` を単一定数として定義する。
- `check_structure_limit(count, subject)` を追加し、
  manifest / spine / captures / collected / NCX navPoint / 目次エントリの
  全蓄積経路で同じチェックを行う。
- 上限超過は `CoreError::Validation` で拒否する。
- エラー文言には要素名と上限値のみを含め、
  id / href / テキスト値は出さない。
- 既存の目次エラー文言・重複 manifest id 拒否・fragment 復号・
  外部スキーム判定・check 名・E2E 件数は維持する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 蓄積前の単一共通チェック（採用） | 全経路で漏れなく防止できる |
| 目次のみ既存上限を維持 | manifest / NCX の枯渇を許す |
| 上限値を固定文言で出さない | 入力由来値ではないため現状維持 |

## 検証（Runtime Proof）

- 新規 test 6 件（manifest / spine / captures / collected / NCX / 正常 EPUB 回帰）。
- core test: **352 passed / 0 failed**。
- `cargo fmt --all -- --check`: rc=0。
- `cargo check --all-targets --locked`: rc=0。
- `cargo clippy --all-targets --locked -- -D warnings`: rc=0。
- wasm core / web check: rc=0。
- `./scripts/verify.sh`: 全ゲート成功、E2E **256 passed / 0 failed**、WDIO gate rc=0。

## 未決定

- 正当な大型 EPUB の互換性判断（10,000 件を超える書籍の受容）。
- ArchiveLimits との定数統合。
