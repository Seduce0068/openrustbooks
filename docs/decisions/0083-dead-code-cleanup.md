---
type: Decision
title: "ADR-0083: デッドコードと古い契約値の整理"
description: "調査で検出された未使用関数と古い E2E 項目数を整理する決定を記録する。"
tags:
  - ADR
  - maintenance
  - testing
generated:
  by: dsh
  at: "2026-11-17T20:25:00+09:00"
status: accepted
---

## 背景

別セッションの調査レポート `docs/investigation/round-01.md` で次を指摘された。

- `tools/verify-web-reader.py` の `open_jpeg_and_measure()` が未呼び出し
  （ADR-0074 の JPEG サイズ比較テスト削除時の残骸）。
- 契約の E2E 項目数が古い（216 のまま、実際は 219）。

## 決定

- 未使用の `open_jpeg_and_measure()` と、それ専用の
  `jpeg_ch2` / `jpeg_epub` / `wide_jpeg()` を削除（計約 70 行）。
- 現在の E2E 項目数 219 を契約に反映する。
- 調査レポートは別セッションの成果物のため、変更・コミットしない。
- プロダクションコードは変更しない。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| デッドコードを削除（採用） | 将来の誤解を防ぎ、保守性が上がる |
| 残してコメントで説明 | 死んだコードが残り続ける |
| サイズ比較を復活 | ADR-0074 で不安定と判断済み |

## 影響

- E2E ハーネスから未使用コードが消える。
- E2E の項目数と内容は変わらない（219 項目）。

## 検証

- `python3 tools/verify-web-reader.py` が 219 項目すべて成功。
- `cargo test --all-targets --locked` は 125 件成功。
- 6 ゲート（fmt / check / test / clippy / wasm32 × 2）と `trunk build` が成功。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- 調査レポートの他の指摘（Wasm AI プレースホルダ、Tauri 未接続、
  要件 REQ-002 〜 009 の未実装など）は別タスクで検討する。
- `IntersectionObserver` による画像の遅延読み込み（ADR-0034 から継続）。
- EPUB 内 CSS、縦書き・ページ送り（ADR-0012 から継続）。
