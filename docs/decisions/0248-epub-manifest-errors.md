---
type: Decision
title: "ADR-0248: EPUB manifest 欠落エラーの統一"
description: "manifest item の id / href 欠落を固定文言の Archive エラーとして fail-closed に報告する決定を記録する。"
tags:
  - ADR
  - epub
  - error
generated:
  by: dsh
  at: "2026-11-19T22:22:00+09:00"
status: accepted
---

## 背景

`push_manifest_item()` は `id` 欠落時に `return Ok(())` で item を黙って
スキップしていた一方、`href` 欠落は `{id}` を埋め込んだ
`CoreError::archive()` で拒否していた（R6 R2-epub#3）。
ADR-0005 / 0059 / 0240 は不正な OPF を黙って読み飛ばさない方針を
定めており、ADR-0247 は本件を未決定として残していた。

## 決定

- `id` 欠落時に
  `CoreError::archive("manifest の item に id がありません")`
  を返す。
- `href` 欠落文言から `{id}` を削除し、
  `CoreError::archive("manifest の item に href がありません")`
  へ統一する。
- いずれも固定文言で、id 値・href 値・ファイル名を埋め込まない。
- `Start` / `Empty` の両 XML イベントは同じ `push_manifest_item()` を通るため、
  両経路へ適用される。
- `id=""` は属性欠落ではないため本次では受理する（未決定）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 欠落を fail-closed で報告（採用） | ADR-0005 / 0059 / 0240 の「黙って読み飛ばさない」と整合 |
| 現状のスキップ維持 | 後段の idref 解決で別エラーになり原因が隠れる |
| 全ての OPF エラーを値非露出化 | 本次は manifest item の 2 文言に限定（他は未決定） |

## 影響

- id 欠落 item を含む EPUB は読み込み不可になる（互換性の受容）。
- リポジトリ内の fixture / E2E の OPF はすべて id 付きで回帰なし。
- エラー文言に機微値を含まないため、Web 側の固定文言変換に依存せず
  core 単体でも値を出さない（ADR-0212 / 0247 を强化）。
- E2E 255 維持。

## 検証（Runtime Proof）

- unit test 2 件追加:
  - `id` 欠落 OPF が固定文言の `CoreError::Archive` になる。
  - `href` 欠落 OPF が固定文言の `CoreError::Archive` になる。
  - いずれも id / href / ファイル名を文言に含まない。
- `cargo test --all-targets --locked`: core **278** / desktop 79 / web 2。
- `wasm-pack test --node --locked`: 15 passed。
- `cargo clippy --all-targets --locked -- -D warnings`、
  wasm32 core / web check、`./scripts/verify.sh`: すべて成功。
- E2E **255 項目すべて成功**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- 実世界 EPUB コーパスにおける id 欠落 item の有無と互換性判断。
- `id=""` を欠落として扱うか（現状は受理）。
- `<manifest>` 外の `<item>` にも本検証を適用するか
  （現状は OPF 内の全 `item` 要素に適用）。
- `resolve_reference` / `build_chapters` / `parse_opf` の
  他エラー文言への redaction 横展開（ADR-0212 / 0247 を継続）。
