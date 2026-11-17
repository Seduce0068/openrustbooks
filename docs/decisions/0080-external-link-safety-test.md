---
type: Decision
title: "ADR-0080: 外部リンク安全性の E2E 検証"
description: "本文内の外部リンクが noopener noreferrer 付きで保持されることを E2E で検証する決定を記録する。"
tags:
  - ADR
  - epub
  - web
  - security
generated:
  by: dsh
  at: "2026-11-17T13:35:00+09:00"
status: accepted
---

## 背景

ADR-0013 で「外部リンクは opener を渡さずに開く」と決定し、
`window.open(..., "_blank", "noopener,noreferrer")` を実装していたが、
E2E で検証していなかった。

## 決定

- サンプル CH1 に `https://example.com` へのリンクを追加する。
- E2E で次を確認する。
  - サニタイズ後も外部リンクの href が残る。
  - `rel="noopener noreferrer"` が付く（ammonia の `link_rel`）。
  - `javascript:` リンクの href は除去される（既存テストを維持）。
- プロダクションコードは変更しない。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| サンプルに外部リンクを追加（採用） | 実際のサニタイズ結果を検証できる |
| サニタイザの単体テストだけ | wasm 側の DOM 展开了確認できない |
| 検証なし | ADR-0013 の決定が回帰から守られない |

## 影響

- 外部リンクの安全性が E2E で継続的に確認できる。
- 既存のサンプルにリンクが 1 つ増えるが、id 付きのため他のテストには影響しない。
- 追加依存は無い。

## 検証

- ヘッドレス Chrome 157 と `tools/verify-web-reader.py` の E2E 214 項目が
  5 回連続で成功。
- 外部リンクの href 保持、`noopener noreferrer` 付与、
  `javascript:` リンクの除去を確認した。
- `cargo test --all-targets --locked` は 125 件成功。
- 6 ゲート（fmt / check / test / clippy / wasm32 × 2）と `trunk build` が成功。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- 実際にリンクをクリックして新しいタブが開くことの検証（E2E の対象外）。
- `IntersectionObserver` による画像の遅延読み込み（ADR-0034 から継続）。
- EPUB 内 CSS、縦書き・ページ送り（ADR-0012 から継続）。
