---
type: Decision
title: "ADR-0162: Markdown Preview のレンダリング"
description: "エディタ Preview で comrak による Markdown レンダリングと二段構えのサニタイズを行う決定を記録する。"
tags:
  - ADR
  - web
  - security
generated:
  by: dsh
  at: "2026-11-18T22:25:00+09:00"
status: accepted
---

## 背景

ADR-0161 で textarea と Preview を接続したが、Preview は
`sanitize_epub_html()` のみで `# Heading` が見出しにならなかった。
core は `comrak 0.59` と `pulldown-cmark 0.13` を宣言済みだが未使用だった。

## 決定

- **comrak 0.59** を採用（`default-features = false` の既存宣言を利用）。
  - CLI / syntect-onig / bon を除外、追加依存なし、Cargo.lock 変更なし。
  - `Options::default()` に `strikethrough` / `table` / `autolink` のみ有効化。
  - pulldown-cmark は未使用のため既存宣言を維持していたが、
    ADR-0163 で直接依存を削除済み（推移依存は残存）。
- `core/src/markdown.rs` に純粋関数 `render_markdown(md) -> String` を追加:
  1. `comrak::markdown_to_html()`（生 HTML は既定 `render.r#unsafe=false` で出力しない）
  2. 生成 HTML を `sanitize_epub_html()`（ammonia 許可リスト）へ通す
  3. 結果を `inner_html` へ渡す
- **逆順（sanitize → Markdown 変換）は禁止**（変換時に生 HTML が再生成され XSS になる）。
- Wasm API `render_markdown_wasm()` を追加し、Preview で利用。
  EPUB 表示側の `sanitize_epub_html()` は維持。
- 生 HTML は Preview で表示されない仕様（Markdown 記法を用いる前提）。
- E2E を 1 件追加（Markdown 見出しが `<h1>` として描画され、
  `<script>` が DOM に残らないこと）。期待総数 237 → **238**
  （check 221 + equal 17）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| comrak featureless（採用） | 純粋 API、wasm 対応、追加依存なし |
| pulldown-cmark | 生 HTML 素通しで防御層が変わる、未使用宣言を残す |
| 生 HTML を通す | XSS リスク |

## 影響

- 新規依存なし、`unsafe` なし。
- `a.href` のスキーム制限と `rel="noopener noreferrer"` は
  `sanitize_epub_html()` の既存契約を継承。
- プロトコル相対 URL（`//evil.com`）の扱いは未検証（**推測**）。
- 巨大入力のデバウンス／入力長上限は未実装（ADR-0161 から継続）。

## 検証（Runtime Proof）

- 新規 native テスト 6 件（heading / link rel / script 除去 /
  javascript: 除去 / table / strikethrough+autolink）。
- `cargo test --all-targets --locked`: core **202** + desktop 56 + web 2。
- wasm32 core check、wasm-test 10、`./scripts/verify.sh`: すべて成功。
- E2E **238 項目すべて成功**
  （「Markdown の見出しが Preview に描画され script が除去される」も ok）。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- ~~pulldown-cmark の削除~~ ADR-0163 で直接依存を削除（推移依存は残存）。
- GFM 拡張の範囲（tasklist / 数式等）の要否。
- `a.href` のプロトコル相対 URL 方針、`target` 許可時の `noopener` 戦略。
- 巨大入力のデバウンス、IME / Undo の実機検証（ADR-0161 から継続）。
