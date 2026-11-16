---
type: Decision
title: "ADR-0060: remote-resources 宣言の扱い"
description: "remote-resources を宣言する EPUB を受け付け、リモートアクセスは表示側の防御に委ねる決定を記録する。"
tags:
  - ADR
  - epub
  - rust
  - security
generated:
  by: dsh
  at: "2026-11-17T06:55:00+09:00"
status: accepted
---

## 背景

ADR-0059 の未決定「`remote-resources` プロパティを宣言する EPUB の扱い」。
core の OPF パーサーは manifest item の `properties` のうち `nav` と
`cover-image` のみを解釈し、他のトークンは無視する。実際のリモートアクセスは
サニタイズ（外部 img src の除去、SVG 外部参照の除去、CSS の除去）と
Blob URL 解決で塞がれている。

## 決定（現状の確認）

- `properties` に `remote-resources` を含む EPUB も受け付ける。
  宣言は解釈せず無視し、目次・章本文の読み出しは従来どおり行う。
- リモート資源への実際のアクセスは、多層防御で塞ぐ方針を維持する。
  - 章 HTML の `<img src>` はローカル参照のみ保持（ADR-0013）。
  - SVG の外部スキーム参照・XML 処理命令を除去（ADR-0044 / 0045）。
  - EPUB 内 CSS は適用しない（ADR-0012）。
- この挙動をテスト `remote_resources_property_does_not_break_navigation` で固定する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 宣言を無視して受け付ける（採用） | 実害は防御側で塞がれており、実在 EPUB を壊さない |
| 宣言を持つ EPUB を拒否する | `remote-resources` は一般的で、正当な本まで開けなくなる |
| 宣言を解釈して追加検証する | 現状の多層防御で十分で、二重実装になる |

## 影響

- ADR-0005 の未決定「リモート資源を宣言する EPUB を許容するか」を
  「許容する（宣言は無視、実アクセスは防御側で遮断）」として解消する。
- プロダクションコードの挙動は変わらない（テスト追加のみ）。

## 検証

- `cargo test -p openrustbooks-core --lib --locked remote_resources` が成功。
- `cargo test --all-targets --locked` は 124 件成功（123 → 124）。
- ヘッドレス Chrome 157 と `tools/verify-web-reader.py` の E2E 176 項目が成功。
- 6 ゲート（fmt / check / test / clippy / wasm32 × 2）と `trunk build` が成功。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- `remote-resources` 宣言を根拠に、利用者へ警告を表示するか。
- `IntersectionObserver` による画像の遅延読み込み（ADR-0034 から継続）。
- EPUB 内 CSS、縦書き・ページ送り（ADR-0012 から継続）。
