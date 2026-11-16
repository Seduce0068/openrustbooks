---
type: Decision
title: "ADR-0045: SVG の XML 処理命令除去"
description: "SVG から XML 処理命令を除去し、外部 CSS 参照による情報漏洩を防ぐ決定を記録する。"
tags:
  - ADR
  - epub
  - web
  - security
generated:
  by: dsh
  at: "2026-11-17T01:48:00+09:00"
status: accepted
---

## 背景

ADR-0044 で SVG の外部スキーム参照を塞いだが、XML 処理命令
（`<?xml-stylesheet href="https://..."?>`）は残っていた。Blob URL を直接開くと、
処理命令が外部 CSS を読み込み、読者の IP や閲覧時刻が漏れる可能性がある。

## 決定

- SVG サニタイズ時に、パース済み Document の子ノードを走査し、
  `PROCESSING_INSTRUCTION_NODE` を削除する。
- DOM API（`child_nodes()` / `remove_child()`）を使い、新規依存・新規
  web-sys feature を追加しない。
- ADR-0043 / ADR-0044 のサニタイズ（script / foreignObject / SMIL / on* /
  外部スキーム除去）はそのまま維持する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| DOM API で処理命令ノードを削除（採用） | 構造的に確実、依存追加なし |
| 文字列置換で `<?xml-stylesheet...` を消す | コメント内等との誤判定があり危険 |
| 何もしない | Blob URL 直接オープンで外部 CSS が読み込まれる |

## 影響

- SVG を含む EPUB で、Blob URL を直接開いても XML 処理命令による外部 CSS
  読み込みが発生しない。
- 通常の SVG 画像は従来どおり表示される。
- 追加依存は無く、既存の `web-sys` 機能の範囲で実装している。

## 検証

- ヘッドレス Chrome 157 と `tools/verify-web-reader.py` の E2E 157 項目が
  5 回中 4 回成功。残り 1 回は既存のフラグメントスクロールテストが
  タイミングで失敗したが、今回の XML 処理命令の変更とは無関係
  （後述のとおり別途調査中）。`xml-stylesheet` と外部 CSS の URL が
  Blob の内容から除去されることを確認した。
- `cargo test --all-targets --locked` は 119 件成功。
- 6 ゲート（fmt / check / test / clippy / wasm32 × 2）と `trunk build` が成功。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- フラグメント付きリンクのスクロールが稀に失敗する問題（E2E フレーク）の
  根本的な原因特定と修正。
- CSS の `url()` による外部参照（EPUB 内 CSS の安全な適用と併せて検討）。
- `IntersectionObserver` による画像の遅延読み込み（ADR-0034 から継続）。
- EPUB 内 CSS、縦書き・ページ送り（ADR-0012 から継続）。
