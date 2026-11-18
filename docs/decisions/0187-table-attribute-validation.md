---
type: Decision
title: "ADR-0187: table 数値属性の fail-closed 検証"
description: "EPUB サニタイズで table 関連の数値属性を検証し、不正値を削除する決定を記録する。"
tags:
  - ADR
  - core
  - security
generated:
  by: dsh
  at: "2026-11-19T05:53:00+09:00"
status: accepted
---

## 背景

`img` の `width` / `height` は `is_valid_dimension()` で検証される一方、
`td` / `th` の `colspan` / `rowspan`、`ol` の `start`、
`li` の `value`、`col` / `colgroup` の `span` は任意文字列を通していた。
極端な値でレイアウト計算コストやメモリが増える可能性があった
（R6: PR-M02）。

## 決定

- `MAX_TABLE_NUMBER: u32 = 1000` を追加する（img の 16384 とは共有しない）。
- `is_valid_table_number()` で
  ASCII 数字・1〜4 桁・`1..=1000` のみを許可する。
- `sanitize_epub_html()` の `attribute_filter` で
  次の 8 組み合わせを検証し、不正値は属性だけを削除する:
  - `td` / `th`: `colspan`, `rowspan`
  - `ol`: `start`
  - `li`: `value`
  - `col` / `colgroup`: `span`
- 非数値属性（`summary` / `scope` / `headers` / `abbr` / `type` /
  `reversed`）は従来どおり維持する。
- E2E は 249 件を維持（unit テストで担保）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 属性単位で削除（採用） | 本文は残しつつ不正なレイアウト指定だけ防ぐ |
| 要素ごと削除 | 正規の本の本文まで失う |
| 上限を img と共有 | サービス目的が異なるため却下 |

## 影響

- HTML 仕様上 `ol start` / `li value` の負数や `colspan=0` は有効だが、
  hardening として拒否する（仕様からの意図的な逸脱）。
- 先頭ゼロは 4 桁以内なら維持（`"0003"` は許可、`"00001"` は拒否）。
- 1 万桁の属性値は html5ever の解析コスト自体は防げず、
  レンダリング時のレイアウトコストを防ぐ。
- 新規依存なし、unsafe なし。

## 検証（Runtime Proof）

- unit test 4 件追加:
  - 正常値が残る。
  - 0 / -1 / 1001 / 全角 / `3px` / 1 万桁が 8 通りすべて削除される。
  - 非数値属性が維持される。
  - 境界（1 / 1000 / 0003 / 0 / 1001 / 00001）が正しい。
- `cargo test --all-targets --locked`: core **224** + desktop 62 + web 2。
- `cargo clippy --all-targets --locked -- -D warnings`、
  `./scripts/verify.sh`: 成功。E2E **249 項目すべて成功**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- `sanitize_svg_xml()` 側の `width` / `height` 等への同等検証。
- 負数 / 0 を許容するか（仕様準拠とのトレードオフ）。
- 上限 1000 の根拠（未計測、推測）。
- E2E への table 属性ケース追加の要否。
- エンティティデコード後の値の扱い。
