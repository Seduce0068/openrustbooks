---
type: Decision
title: "ADR-0146: E2E スクロール読み書きの軸非依存化"
description: "E2E の DOM スクロール読み書きを writing-mode に応じて軸を選ぶヘルパーへ集約する決定を記録する。"
tags:
  - ADR
  - testing
  - e2e
generated:
  by: dsh
  at: "2026-11-18T16:05:00+09:00"
status: accepted
---

## 背景

ADR-0145 の最初のステップとして、縦書き導入前に
E2E のスクロール読み書きを軸非依存ヘルパーへ集約する必要があった。
`tools/verify-web-reader.py` には `scrollTop` が 49 箇所あったが、
内訳は DOM プロパティアクセス 14 箇所と JSON キー／メッセージ／コメント等だった。

## 決定

- `tools/verify-web-reader.py` に次を追加:
  - `_scroll_axis_js(target)`（内部）: computed `writing-mode` が
    `vertical-*` / `sideways-*` なら `scrollLeft`、それ以外 `scrollTop`。
  - `scroll_axis_expr()` / `scroll_read_expr()` / `scroll_set_expr()` /
    `scroll_reset_expr()` / `scroll_offset_expr()` / `scroll_delta_expr()`。
- DOM の `scrollTop` 直接参照 14 箇所と `rect.top` 差 2 箇所を
  ヘルパー呼び出しへ置換した（合計 16 サイト）。
- スクロール順序計測の injected JS も軸非依存化し、
  横書きでは従来どおり `top` キーに `scrollTop` を記録する。
- **保存 JSON のキー `scrollTop` は変更しない**。
  - `sorted(saved.keys()) == ["chapterIndex", "fragment", "savedAt", "scrollTop"]` を維持。
  - `scrollLeft` キーは追加しない。
  - `scrollTop` は後方互換の識別子であり、DOM プロパティではない
    （縦書き時は軸非依存オフセットを格納する意味になる、**要検証**）。
- チェック名、期待総数 **229**、閾値（2px / 50px / 30px）は据え置き。
- アプリ本体（`web/src/lib.rs` 等）は変更しない。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| ヘルパー集約（採用） | 横書きの挙動を保ちつつ縦書きの準備ができる |
| 全 49 箇所を書き換え | JSON キーまで壊れる |
| アプリの保存形式を先に変更 | E2E 229 項目と履歴・エクスポートが一斉に壊れる |

## 影響

- 横書きでは computed writing-mode が `horizontal-tb` のため、
  ヘルパーは従来どおり `scrollTop` を返す（fail-closed）。
- `.reader-sidebar` 等の他要素は置換対象外（注入リスナは per-target 判定）。
- 新規依存なし、unsafe なし。

## 検証

- `python3 -m py_compile tools/verify-web-reader.py` 成功。
- `grep -E '\.scrollTop\b'` は 0 件（DOM 直接参照なし）。
  JSON キー `scrollTop` と期待総数 229 は残存。
- E2E 1 回 229/0、`./scripts/verify.sh` rc=0（6 ゲート + audit + wasm-test すべて成功）。
- 10 回連続ベースライン: **10/10 すべて 229/0**（ADR-0140 の
  2px / 50px / 30px 閾値を含む位置復元系も維持）。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定（axis-e2e-auditor の指摘）

- `vertical-rl` の `scrollLeft` 符号と `> 0` 比較の扱い（実機未検証）。
- `writing-mode` 宣言の正規位置（`.reader-content` vs `.reader-html`）と継承の扱い。
- computed style 取得の flush が scroll-order 計測の 50px 窓へ与える影響（推測）。
- `offsetTop/offsetLeft` と `offsetParent` の関係、`rect.top/left` 差分の符号（推測）。
- アプリ側 position 保存の軸非依存化とスキーマ移行（ADR-0016 / 0036 / 0076）。
- CSS の `overflow-y` を論理プロパティへ置換する時期（ADR-0145 から継続）。
- 20 回連続実行、5 ブラウザ実機検証（ADR-0140 / 0145 から継続）。
