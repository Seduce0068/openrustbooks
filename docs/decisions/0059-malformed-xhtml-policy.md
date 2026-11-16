---
type: Decision
title: "ADR-0059: 非整形式 XHTML の扱い"
description: "OPF は fail-closed、章本文はサニタイズに委譲する実際の挙動を確認し記録する。"
tags:
  - ADR
  - epub
  - rust
  - security
generated:
  by: dsh
  at: "2026-11-17T06:45:00+09:00"
status: accepted
---

## 背景

ADR-0005 の未決定「非整形式の XHTML を許容するかどうかの判断（現状は fail-closed）」。
調査の結果、`EpubBook::chapter_html` は内容をそのまま返し XML 検証をせず、
Web 側の `sanitize_epub_html`（ammonia）が HTML としてサニタイズしていた。
つまり ADR の「fail-closed」という記述は OPF / nav には当てはまるが、
章本文には当てはまらない。

## 決定（現状の確認）

- OPF（メタデータ・spine・manifest）および nav は XML として解析し、
  構文エラーは `CoreError::Archive` として拒否する（fail-closed）。
- 章本文（XHTML）は XML 検証せずそのまま返し、表示側のサニタイズ
  （許可リスト方式の ammonia）に委ねる。非整形式でも表示できる。
- この方針をテストで固定する。
  - `malformed_chapter_xhtml_is_passed_through`: 閉じタグ欠落の章が
    `chapter_html` でそのまま返る。
  - `malformed_opf_is_rejected`: 構文エラーの OPF で `open` が失敗する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 現状の挙動を確認・固定（採用） | 既存 EPUB を壊さず、実際の安全性を明示できる |
| 章本文も XML 厳密検証に変更 | 非整形式な実在 EPUB を開けなくなる（回帰リスク） |
| 未決定のまま残す | ADR と実装の不一致が続く |

## 影響

- ADR-0005 の未決定を「決定済み（現状の確認）」として解消する。
- サニタイズの許可リストが最終防御であることが明示される。
- プロダクションコードの挙動は変わらない（テスト追加のみ）。

## 検証

- `cargo test -p openrustbooks-core --lib --locked malformed` は 2 件成功。
- `cargo test --all-targets --locked` は 123 件成功（121 → 123）。
- ヘッドレス Chrome 157 と `tools/verify-web-reader.py` の E2E 176 項目が成功。
- 6 ゲート（fmt / check / test / clippy / wasm32 × 2）と `trunk build` が成功。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- `remote-resources` プロパティを宣言する EPUB の扱い（ADR-0005 から継続）。
  現状は宣言を無視するが、リモート資源へのアクセスはサニタイズと
  Blob URL 解決の多層防御で塞がれている。
- `IntersectionObserver` による画像の遅延読み込み（ADR-0034 から継続）。
- EPUB 内 CSS、縦書き・ページ送り（ADR-0012 から継続）。
