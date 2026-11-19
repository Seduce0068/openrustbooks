---
type: Decision
title: "ADR-0240: 正規化後同名 ZIP エントリの拒否"
description: "parser differential を防ぐため、正規化後に同名となる ZIP エントリを含むアーカイブを fail-closed で拒否する決定を記録する。"
tags:
  - ADR
  - archive
  - security
generated:
  by: dsh
  at: "2026-11-19T20:14:00+09:00"
status: accepted
---

## 背景

`ArchiveReader` は `index.entry(path).or_insert(position)` で
正規化後同名のエントリを先着保持していた（R6 R1-archive#4）。
core 内では `is_epub()` も `read_entry()` も同じ先着を参照するため整合するが、
`entries()` は重複を露出しており、
外部 consumer が「後勝ち」を選ぶと parser differential が生じ得た。
`mimetype` / `META-INF/container.xml` / OPF の重複は特に危険。

## 決定

- `from_owned()` の走査ループで
  `index.insert(path.clone(), position).is_some()` の場合に
  `CoreError::validation("正規化後に同名のエントリがあります: {path}")`
  を返す。
- `new()` / `with_limits()` / `from_owned()` / `EpubBook` の各生成経路と
  `WasmArchive::new()` はすべて `from_owned()` に収束するため、
  1 箇所の変更で全経路に適用される。
- テストは「異なる raw 名・正規化後同名」の組で生成する:
  - `./a.txt` と `a.txt`
  - `a//b.txt` と `a/b.txt`
  - `a\b.txt` と `a/b.txt`
  - `a/./b.txt` と `a/b.txt`
  - `a.txt/` と `a.txt`（完全一致の proxy）
- EPUB では `mimetype` と `./mimetype` の重複が
  `CoreError::Validation` で拒否されることを確認する
  （`UnsupportedFormat` ではない）。
- `ArchiveEntry` / 公開 API / `Send + Sync` は変更しない。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 全体拒否（採用） | ADR-0004 の「逸脱エントリはアーカイブ全体を拒否」と整合 |
| 先着保持（現状） | parser differential のリスクが残る |
| 後勝ち | 検証と消費で選択が変わり得るため却下 |

zip 9.1.3 の `ZipWriter` は同一 raw 名を拒否し、
Reader は `IndexMap<Box<[u8]>, ...>` により raw 名が同一なら
1 エントリへ畳み込む（`by_index` の
`enforce_header_consistency` は既定 false）。
したがって完全一致 raw 名の重複は今回の走査には現れず、
外部ツールが作った場合は最後のエントリが黙って採用され得る。
この **完全一致 raw 名の検出不能は未対応** として、
`ZipArchive::has_duplicate_names()` の追加呼び出しを未決定に残す。
テストは正規化等価な raw 名で代替し、コードコメントに理由を記載した。

## 影響

- 重 duplicate を含む既存 EPUB / ZIP は読み込み不可になる（互換性の受容）。
- 正規化後に別名のエントリは従来どおり読み込める。
- 通常 fixture / EPUB はすべて正規化後一意で、回帰なし。
- E2E 254 維持（VC-014 は core unit のみ）。

## 検証（Runtime Proof）

- unit test 5 件追加（archive 3 / epub 1 / wasm 1）。
- `cargo test --all-targets --locked`: core **274** / desktop 79 / web 2。
- `wasm-pack test --node --locked`: **15 passed**。
- `cargo clippy --all-targets --locked -- -D warnings`、
  wasm32 core / web check、`./scripts/verify.sh`: すべて成功。
- E2E **254 項目すべて成功**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- 累計実展開上限（`max_total_size` は宣言値の合計で、
  実展開バイトのセッション予算は未実装）。
- 明示的な XML 深さ上限（現在はテストと `MAX_TOC_ENTRIES` のみ）。
- OPF manifest の `id` 重複（`or_insert` で先着保持）。
- `is_image_media_type()` の空 media-type 許容を継続するか
  （ADR-0008 の判断を維持する前提）。
- 完全一致 raw 名の検出（`ZipArchive::has_duplicate_names()` の追加呼び出し）。
  現状は Reader が 1 エントリへ畳み込むため検出できない。
- 重複エントリを生成する実世界ツールの調査。
