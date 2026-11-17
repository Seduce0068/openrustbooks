---
type: Decision
title: "ADR-0081: 外部リンクのポップアップ呼び出し検証"
description: "外部リンクのクリックが noopener/noreferrer 付きの window.open 呼び出しになることを E2E で検証する決定を記録する。"
tags:
  - ADR
  - epub
  - web
  - security
generated:
  by: dsh
  at: "2026-11-17T13:50:00+09:00"
status: accepted
---

## 背景

ADR-0080 では外部リンクの href と `rel="noopener noreferrer"` を検証したが、
実際にクリックして `window.open(..., "_blank", "noopener,noreferrer")` が
呼ばれることは未検証だった。

## 決定

- E2E でクリック前に `window.open` をスタブし、呼び出し引数を記録する。
- 外部リンクをクリックし、次を確認する。
  - `window.open` が 1 回呼ばれる。
  - URL が `https://example.com`、target が `_blank`、features に
    `noopener` と `noreferrer` が含まれる。
- 実際にはタブを開かないため、外部ネットワークアクセスは発生しない。
- 検証後はリロードしてスタブを元に戻す。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| `window.open` をスタブ（採用） | 副作用なしで引数を直接検証できる |
| CDP の `Page.windowOpen` を捕捉 | ハーネスのイベント処理変更が必要で複雑 |
| 実際にタブを開く | 外部ネットワークアクセスが発生し不安定 |

## 影響

- ADR-0013 の決定がクリック動作まで含めて回帰から守られる。
- プロダクションコードの変更は無い。
- 追加依存は無い。

## 検証

- ヘッドレス Chrome 157 と `tools/verify-web-reader.py` の E2E 216 項目が
  5 回連続で成功。
- `window.open` が 1 回呼ばれ、URL / target / features が期待どおりである
  ことを確認した。
- `cargo test --all-targets --locked` は 125 件成功。
- 6 ゲート（fmt / check / test / clippy / wasm32 × 2）と `trunk build` が成功。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- `mailto:` リンクのクリック動作の検証。
- `IntersectionObserver` による画像の遅延読み込み（ADR-0034 から継続）。
- EPUB 内 CSS、縦書き・ページ送り（ADR-0012 から継続）。
