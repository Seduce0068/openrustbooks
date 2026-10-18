---
type: Decision
title: "ADR-0010: EPUB読み出し API を WebAssembly に公開する"
description: "WasmEpubBook で EPUB の読み取り専用 API を JavaScript から利用する方針を記録する。"
tags:
  - ADR
  - epub
  - wasm
  - core
  - api
generated:
  by: opencode
  at: "2026-10-18T14:19:25+09:00"
status: accepted
---

## 背景

ADR-0003 により共通 core は `wasm32-unknown-unknown` でもビルドする。
EPUB の読み取り API は native と Wasm の両方で利用できるが、既存の
`wasm-bindgen` 面には EPUB の型がなく、ブラウザ UI から呼び出せなかった。

## 決定

Wasm ターゲットに `WasmEpubBook` を公開する。コンストラクターで ZIP の
バイト列を受け取り、内部に `EpubBook` を所有する。次を JavaScript から利用可能にする。

- 章数と章のアーカイブ内パス。
- 章 XHTML・プレーンテキスト・相対リソースの読み出し。
- 章タイトル、目次、landmarks、page-list、表紙画像。

`CoreError` は既存の表示文字列に変換して JavaScript の例外として返す。
リスト・表紙メタデータは既存依存の `serde-wasm-bindgen` で JavaScript 値に変換する。
新規依存、HTML の描画、サニタイズ、URL の書き換えは追加しない。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| Wasm binding に `WasmEpubBook` を追加（採用） | native 実装を共有し、JS から使える |
| web 側に別の EPUB parser を実装 | 解析ロジックとエラー処理が二重化する |
| `chapter_html` を直ちに `inner_html` へ挿入 | サニタイズ方針未決定のため危険。API 公開と描画を分離する |
| 各操作を個別の free function とする | EPUB のアーカイブ状態を毎回開き直すため不採用 |

## 影響

- JS から `new WasmEpubBook(Uint8Array)` を使い、章・ナビゲーション情報を取得できる。
- JS の `chapter_resource` は `Uint8Array` を返す。任意の HTML を描画する機能は含まない。
- Wasm への追加 API は `cfg(target_arch = "wasm32")` に限定され、native の API と
  ビルド境界を変えない。
- UI はサニタイズ、リソース URL の安全な生成、Blob / object URL の寿命管理を
  別途設計してから描画を行う。

## 検証

- `cargo check -p openrustbooks-core --target wasm32-unknown-unknown` と
  `cargo check -p openrustbooks-web --target wasm32-unknown-unknown` が成功。
- `trunk build --offline --dist <一時出力先>` が成功し、生成 JavaScript に
  `WasmEpubBook`、`chapter_resource`、`landmarks` などの binding が存在することを確認。
- Rust の native gates と Wasm build gates が成功した。

## 未決定

- `WasmEpubBook` を Web UI のファイル選択・状態管理へ接続する方法。
- HTML サニタイズ、リソース URL 化、アンカー移動、object URL の破棄。
- 大きな EPUB のメモリ使用と、ブラウザ上の実機検証。
