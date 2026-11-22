---
type: Decision
title: "ADR-0345: EPUB XHTML サニタイズの入力上限"
description: "サニタイザへ渡す前に入力長とタグ深度を制限し、深いネストによる wasm フリーズを防ぐ決定。"
tags:
  - ADR
  - EPUB
  - availability
generated:
  by: dsh
  at: "2026-11-23T03:42:00+09:00"
status: accepted
---

## 背景

`sanitize_epub_html` は入力長/タグ深度を制限せず ammonia へ渡していた。
1 章に深いネストや巨大テキストを含む悪意ある EPUB を開くと、
wasm 側で数秒以上のフリーズとメモリ消費が起きた。

## 決定

- `MAX_SANITIZE_INPUT_BYTES = 4 MiB` / `MAX_SANITIZE_TAG_DEPTH = 128` を導入する。
- 上限超過時は入力を含まない固定の空文字を返す（fail-closed）。
- API シグネチャは維持し、web 側の呼び出しと E2E check は変更しない。
- 深度スキャンはタグ名付きスタックと quote 状態追跡を使う:
  - 未対応の終了タグ（`<div></x>`）では深度を減らさない。
  - 属性値内の `<` / `>` / 引用符、本文の `1 < 2` はタグ開始と数えない。
  - void 要素は ASCII 大文字小文字を区別せずスタックへ積まない。

## 検証（Runtime Proof）

- core test: **364 passed / 0 failed**。
- 追加 test: 4 MiB 超 / 129 段 / 10000 段拒否、128 段境界成功、
  `1 < 2`・属性内特殊文字の誤拒否なし、未対応終了タグ拒否、
  大文字 void タグ（`<BR>` / `<IMG>` / `<Hr/>`）の誤爆なし。
- `cargo fmt --all -- --check`: rc=0。
- `cargo clippy --all-targets --locked -- -D warnings`: rc=0。
- wasm check / `ci-wasm-test.sh`: rc=0。
- `./scripts/verify.sh`: 全ゲート成功、E2E **263 passed / 0 failed**、WDIO gate rc=0。

## 未決定

- 上限拒否を web の status へ固定文言で通知するか（現状は空文字で無言）。
