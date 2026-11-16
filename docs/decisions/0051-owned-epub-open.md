---
type: Decision
title: "ADR-0051: EPUB バイト列の所有権移譲による二重コピー解消"
description: "wasm で EPUB を開くときのアーカイブバイト列の二重コピーを解消する決定を記録する。"
tags:
  - ADR
  - rust
  - wasm
  - performance
generated:
  by: dsh
  at: "2026-11-17T03:30:00+09:00"
status: accepted
---

## 背景

ADR-0013 の未決定「読み込み時のメモリコピー（`ArrayBuffer`・`Vec`・
`ArchiveReader` の複製）」。調査の結果、`WasmEpubBook::new` が
`Vec<u8>` を受け取った後、`EpubBook::open(&bytes)` →
`ArchiveReader::with_limits(bytes, ...)` が `bytes.to_vec()` でアーカイブ全体を
二重にコピーしていた（256 MiB の EPUB なら 256 MiB の余分なコピー）。

## 決定

- `ArchiveReader::from_owned(bytes: Vec<u8>, limits)` を追加する。
  既存の `with_limits` は `bytes.to_vec()` して `from_owned` に委譲する
  （既存 API の挙動は維持）。
- `EpubBook::open_owned(bytes: Vec<u8>)` を追加し、メタデータ読み出しは
  `from_archive` に共通化する。
- `WasmEpubBook::new` は `open_owned(bytes)` を使い、JS から渡された
  バイト列の所有権をそのままアーカイブに渡す（コピー 1 回分を削減）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 所有権移譲 API を追加（採用） | 既存 API を壊さず、wasm の二重コピーだけを解消できる |
| `with_limits` の引数を `Vec` に変更 | native 側の既存呼び出しすべてに影響する |
| コピーを放置 | 256 MiB の EPUB で余分なメモリを使用する |

## 影響

- wasm で EPUB を開くときのメモリ使用量とコピー時間が削減される。
- native 側の `open` / `with_limits` の挙動は変わらない。
- `chapter_resource` や画像 Blob 生成時のコピーは残る（表示時に
  エントリごとに必要なコピーであり、今回の対象外）。

## 検証

- `core/src/epub.rs` に `open_owned_matches_open` テストを追加し、
  メタデータ・章一覧・本文 HTML が `open` と一致することを確認。
- `cargo test --all-targets --locked` は 121 件成功（119 → 121）。
- ヘッドレス Chrome 157 と `tools/verify-web-reader.py` の E2E 166 項目が
  5 回中 4 回成功。残り 1 回は既知のフラグメントスクロールのフレークで、
  本変更とは無関係（別途調査中）。
- 6 ゲート（fmt / check / test / clippy / wasm32 × 2）と `trunk build` が成功。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- `chapter_resource` が返す `Vec<u8>` から JS の `Uint8Array` へのコピー。
- `IntersectionObserver` による画像の遅延読み込み（ADR-0034 から継続）。
- EPUB 内 CSS、縦書き・ページ送り（ADR-0012 から継続）。
