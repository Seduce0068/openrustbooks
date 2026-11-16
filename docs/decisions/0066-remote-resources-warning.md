---
type: Decision
title: "ADR-0066: remote-resources 宣言の警告"
description: "remote-resources を宣言する EPUB を開いたとき、利用者に警告を表示する決定を記録する。"
tags:
  - ADR
  - epub
  - web
generated:
  by: dsh
  at: "2026-11-17T08:30:00+09:00"
status: accepted
---

## 背景

ADR-0060 の未決定「`remote-resources` 宣言を根拠に、利用者へ警告を表示するか」。
宣言を持つ EPUB は受け付ける方針だが、外部資源を参照する可能性があることを
利用者に知らせる。

## 決定

- `EpubBook::has_remote_resources()` を追加し、manifest item の `properties` に
  `remote-resources` を含む場合に true を返す。
- wasm バインディング経由で Web から参照できるようにする。
- 本を開いたとき、宣言があれば状態メッセージ
  `This EPUB declares remote resources.` を表示する。
- 宣言が無い場合はメッセージを出さない。
- 警告は表示のみで、読み込みや表示を阻止しない（ADR-0060 の方針維持）。
- 章描画で status がクリアされるため、`render_chapter` の後に警告を設定する。
- 巨大画像警告（ADR-0064）と両方該当する場合は、位置適用完了時に出る
  巨大画像警告が優先される。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 警告を表示（採用） | 利用者がネットワークアクセスへの意識を持てる |
| 読み込みを拒否 | ADR-0060 の方針（受け付ける）に反する |
| 警告なし | 宣言の存在が利用者に伝わらない |

## 影響

- `remote-resources` を宣言する本を開くと警告が出る。
- 通常の本（宣言なし）では警告は出ない。
- 追加依存は無く、既存の DOM / wasm-bindgen の範囲で実装している。

## 検証

- ヘッドレス Chrome 157 と `tools/verify-web-reader.py` の E2E 189 項目が
  5 回連続で成功。既存サンプル（宣言なし）では警告が出ないこと、
  `remote-resources` を宣言する EPUB で警告が出ることを確認した。
- `cargo test --all-targets --locked` は 125 件成功。
- 6 ゲート（fmt / check / test / clippy / wasm32 × 2）と `trunk build` が成功。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- 警告の表示時間・消し方（現在は次の操作まで残る）。
- `IntersectionObserver` による画像の遅延読み込み（ADR-0034 から継続）。
- EPUB 内 CSS、縦書き・ページ送り（ADR-0012 から継続）。
