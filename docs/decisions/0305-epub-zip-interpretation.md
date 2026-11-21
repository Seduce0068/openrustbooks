---
type: Decision
title: "ADR-0305: EPUB / ZIP の解釈統一"
description: "BOM 入り mimetype、fragment 復号、重複 manifest id、外部スキーム判定を core / web で統一する決定。"
tags:
  - ADR
  - EPUB
  - security
generated:
  by: dsh
  at: "2026-11-22T08:28:00+09:00"
status: accepted
---

## 背景

core と web で EPUB / ZIP の解釈に差があった。
BOM 入り mimetype の誤判定、fragment の未復号、
manifest id 重複の黙認、%XX 復号失敗の Validation 飲み込み、
web の外部スキーム前方一致判定が対象。

## 決定

- `is_epub()` は先頭 BOM を 1 個だけ除去して比較する。
  連続 BOM は 2 個目が残るため不正とする。
- `xmlutil::split_fragment()` を `CoreResult` 化し、
  nav / NCX / `ChapterLink` の全経路で percent decode と不正 % の
  Validation を一箇所へ集約する。
- 既存 ADR-0007 の「目次 fragment は生の綴り保持」は本 ADR が supersede し、
  復号済み fragment を保持する。
- manifest id 重複は `CoreError::Validation` で拒否する（id 値は文言に出さない）。
- リテラル `%` を含むファイル名は生名一致を先に試すフォールバックで保持する。
- web は core と同規則の `has_external_scheme()` / fragment 復号を使い、
  http(s) / mailto のみ `window.open` を許可する。
- core の判定関数を wasm 経由で web test から直接比較する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| core 規則を単一情報源化（採用） | 重複規則の drift を防げる |
| web で独自判定を維持 | 既存の不一致が残る |
| リテラル % を全拒否 | 既存 EPUB 互換性を壊すため却下 |

## 検証（Runtime Proof）

- core 341 / web 9 tests、すべて成功。
- `cargo fmt --all -- --check`: rc=0。
- `cargo check --all-targets --locked`: rc=0。
- `cargo clippy --all-targets --locked -- -D warnings`: rc=0。
- wasm core / web check: rc=0。
- `npx tsc --noEmit`: rc=0。
- `scripts/check-e2e-count-consistency.sh`: rc=0（Checker(expected=256)）。
- `./scripts/verify.sh`: 全ゲート成功、E2E **256 passed / 0 failed**、WDIO gate rc=0。

## 未決定

- 外部リンクの allowlist を設定可能にするか。
- 压缩包 / OPF のすべての URI _attributes を厳格検証するか。
