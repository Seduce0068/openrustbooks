---
type: Decision
title: "ADR-0042: wasm32 clippy 警告の解消"
description: "wasm32 ターゲットの clippy 警告をゼロにし、コードをクリーンに保つ決定を記録する。"
tags:
  - ADR
  - rust
  - wasm
  - tooling
generated:
  by: dsh
  at: "2026-11-17T01:05:00+09:00"
status: accepted
---

## 背景

native の 6 ゲートは `-D warnings` で通っていたが、wasm32 ターゲットの
`cargo clippy` には警告が残っていた。最近の変更で 2 件（初期化外のフィールド
代入、冗長な signal 再定義）が増えたため、まとめて解消した。

## 決定

### 解消した警告

- `WasmConfig` に `Default` を実装し、`new()` は `Self::default()` に委譲する。
- `load_settings()` の初回デフォルトを、代入ではなく構造体更新構文
  (`ReaderSettings { follow_system: .., ..Default::default() }`) で作る。
- `import_positions()` の `set_item(&key, ...)` を `set_item(key, ...)` にする。
- 冗長な `let set_system_dark = set_system_dark;` を削除する。
- `render_chapter` の複雑な型に `RenderChapter` エイリアスを導入する。
- `Copy` なクロージャへの不要な `.clone()` を削除する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 警告をすべて解消（採用） | 将来の変更で警告が埋もれないようにする |
| native のゲートだけ通せばよい | wasm32 固有の問題を見逃す |
| `#[allow]` で抑制 | 実際の改善にならない |

## 影響

- 挙動は一切変わらない（警告解消と型エイリアスのみ）。
- wasm32 の `cargo clippy` が警告 0 件になる。
- 依存クレート（`openrustbooks-core` 由来の警告を除く）は対象外。

## 検証

- `cargo clippy -p openrustbooks-web --target wasm32-unknown-unknown --locked`
  が警告 0 件で完了。
- ヘッドレス Chrome 157 と `tools/verify-web-reader.py` の E2E 147 項目が
  5 回連続で成功。
- `cargo test --all-targets --locked` は 119 件成功。
- 6 ゲート（fmt / check / test / clippy / wasm32 × 2）と `trunk build` が成功。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- `openrustbooks-core` の残る警告（`WasmConfig` 以外）の扱い。
- `cargo clippy -- -D warnings` を wasm32 にも適用するか。
- `IntersectionObserver` による画像の遅延読み込み（ADR-0034 から継続）。
- EPUB 内 CSS、縦書き・ページ送り（ADR-0012 から継続）。
