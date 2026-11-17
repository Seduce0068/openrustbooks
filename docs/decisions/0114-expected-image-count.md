---
type: Decision
title: "ADR-0114: 期待画像数の算出"
description: "expected_images を src を持つ画像の数にし、src 削除画像による無駄な 2 秒待ちを解消する決定を記録する。"
tags:
  - ADR
  - web
  - bugfix
generated:
  by: dsh
  at: "2026-11-18T05:50:00+09:00"
status: accepted
---

## 背景

`expected_images` は章 HTML の `html.matches("<img").count()` で算出していたが、
`resolve_images()` で解決失敗して `src` を削除された画像もカウントしていた。
一方 `images_settled()` は src なし画像をスキップするため
`with_source >= expected` が成立せず、2 秒 deadline まで待ち続けていた
（ADR-0113 の未決定）。

## 決定

- `count_images_with_src()` を追加し、`expected_images` を
  「src 属性を持つ `<img>` の数」に変更する。
- 文字列解析を採用する（Leptos の DOM 反映タイミングに依存しないため）。
  - タグ名・属性名は ASCII 大小文字無視。
  - `=` 前後の空白、`'`/`"` クォート、クォート内の `>` を正しく扱う。
  - **属性スキャンでもクォート状態を追跡し、alt/title の値に含まれる
    " src " を誤検出しない**（監査で見つかった実バグを修正）。
- `images_settled()` の判定（`complete() && naturalWidth > 0`）と
  `with_source >= expected` は変更しない。
- `media-src` 方式（resolve_images の成功 URL 数）は今回採用しない。
  expected を HTML 内の src 付き img に直接結び付ける方が回帰テストを
  書きやすいため（将来の切替は可能）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 文字列スキャナ（採用） | DOM 反映タイミング非依存、テスト可能 |
| `set_content` 後の DOM クエリ | Leptos の Effect タイミング依存で旧章を読むリスク |
| `resolve_images()` の URLs.len() | 確実だが内部実装に結合する（将来検討） |

## 影響

- src 削除画像を含む章でも 2 秒待たずに位置復元へ進む。
- src あり画像が 1 枚でも未完了なら待ち続ける（過小見積もりしない）。
- web クレートに 2 件の単体テストを追加（native 実行可能）。
- 既存の 193 テスト、E2E 228 項目は変更なし。

## 検証

- 回帰テスト: `<img alt="image src here">` が 0 件、
  `<img src="x">` が 1 件、大文字／空白／クォート内 `>` 等を検証。
- `cargo fmt --all` / wasm check / wasm clippy（`-D warnings`）成功。
- `./scripts/verify.sh` を実行し、6 ゲート・trunk build・WASM パス検査・
  E2E 228 項目がすべて成功した。
- 全単体テスト core 165 + desktop 28 + web 2。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- `resolve_images()` の成功 URL 数への切り替え（将来の堅牢化）。
- `expected = 0` の章で DOM 反映が間に合わない場合の保証（ADR-0112 の
  「可視／位置に影響する画像」定義と統合時に検討）。
- 破損画像の error イベント即完了扱い（ADR-0113 から継続）。
