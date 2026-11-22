---
type: Decision
title: "ADR-0317: バックアップ JSON の実用上限と構造制限"
description: "8 MiB のバイト上限と parse 前の構造走査で、バックアップ import によるヒープ枯渇を防ぐ決定。"
tags:
  - ADR
  - web
  - security
generated:
  by: dsh
  at: "2026-11-22T12:36:00+09:00"
status: accepted
---

## 背景

バックアップ JSON の上限は core の read 上限（256 MiB）から導出されていた。
`serde_json::from_str::<Value>` は元テキストに加えて巨大なツリーを確保するため、
ブラウザタブのヒープ枯渇が懸念された。

## 決定

- `IMPORT_JSON_MAX_BYTES` を 8 MiB の実用値へ変更する。
- `import_positions()` の parse 前に `serde` Visitor による構造走査を行い、
  トップレベル 64 キー / セクション 10,000 キー / settings 64 キー /
  文字列 2 MiB / 深さ 16 / ノード総数 20,000 を拒否する。
- 構造制限超過のみ固定文言 `IMPORT_JSON_TOO_LARGE_MESSAGE` を返す。
- 構文エラーは従来の「JSON の解析に失敗しました」を維持する。
- 未知 version のエラー文言から値を削除する。
- version / merge / settings_preserved / failed 件数の既存契約は維持する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 実用上限 + parse 前走査（採用） | Value 展開前に拒否できる |
| 256 MiB のまま | ヒープ枯渇リスクが残る |
| parse 後チェック | 確保後の拒否では効果が薄い |

## 検証（Runtime Proof）

- web test: **17 passed / 0 failed**（新規 8 件 + 構文分類 1 件）。
- `cargo check -p openrustbooks-web --target wasm32-unknown-unknown --locked`: rc=0。
- `cargo clippy -p openrustbooks-web --all-targets --locked -- -D warnings`: rc=0。
- `npx tsc --noEmit`: rc=0。
- `./scripts/verify.sh`: 全ゲート成功、E2E **256 passed / 0 failed**、WDIO gate rc=0。

## 未決定

- 上限値のユーザー設定化。
- ストリーミング（-import の完全な incremental 化）。
