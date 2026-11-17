---
type: Decision
title: "ADR-0006: EPUBの目次（nav / NCX）を平坦な深さ付きリストで返す"
description: "EPUB3 nav と EPUB2 NCX の読み分け、平坦リスト採用の理由、欠落と破損の扱いを記録する。"
tags:
  - ADR
  - epub
  - core
  - navigation
generated:
  by: opencode
  at: "2026-10-17T18:55:00+09:00"
status: accepted
---

## 背景

`docs/requirements.md` のリーダー検証条件は「目次・ページ・章を移動できる」こと
を求める。ADR-0005 では本文と章の読み出しだけを実装し、目次は未決定事項として
残した。EPUB の目次は版によって情報源が異なる。

| 版 | 情報源 | 場所 |
| --- | --- | --- |
| EPUB3 | XHTML の `nav` | manifest の `properties` に `nav` を含む item |
| EPUB2 | NCX | media-type が `application/x-dtbncx+xml` の item |

両方を持つ EPUB も実在するため、優先順位を決める必要がある。

## 決定

`EpubBook::toc()` と `EpubTocEntry { title, href, depth }` を公開し、
目次を平坦なリストとして返す。

- 情報源は EPUB3 nav を優先し、無ければ NCX、どちらも無ければ空を返す。
- `href` は本文と同じ `resolve_href` で解決する（フラグメントは落とす）。
  これにより UI は `toc()[i].href == chapters()[j].href` で章と対応付けられる。
- `depth` は最上位を 0 とする入れ子の深さ。入れ子は再帰ではなく明示的な
  カウンタで数える（深い入力でスタックを消費しない）。
- NCX の `playOrder` は無視し、文書順を正とする（実際の EPUB では
  `playOrder` が欠落・重複している例がある）。
- `href` を持たない見出し（EPUB3 の `<span>`、`<content>` の無い `navPoint`）は
  エントリにしない。
- 情報源のエントリがアーカイブに無い場合は空を返す（目次は補助情報であり、
  本文が読める EPUB を開けなくしない）。
- XML の構文エラーは `CoreError::Archive`、`href` の逸脱は
  `CoreError::Validation`、エントリ数 10000 超は `CoreError::Validation`。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 平坦リスト + `depth`（採用） | UI はインデント表示で足り、再帰も借用も不要でテストしやすい |
| 入れ子 `Vec<children>` | 構造は忠実だが、深さ方向の再帰とスタック消費を持ち込む |
| 章タイトルを `EpubChapter` に埋め込む | 目次の無い EPUB で情報が無く、API の後方互換も崩す。UI で対応付ける方が単純 |
| アンカー（`#fragment`）を保持 | 将来必要になるが、今回の目的（章への移動）には不要で、API を先に固めない |

## 影響

- 目次 UI は `chapters()` と `toc()` を href で突き合わせて実装できる。
- アンカー単位の移動、`nav` の `landmarks` / `page-list`、章タイトルの
  自動取得は未対応。
- `toc()` は呼び出しごとに目次文書を読み直す。内部可変性を使わない方針
  （ADR-0005）を維持した結果で、目次文書は小さく実害は無い。

## 検証

- 5 ゲートが緑。`cargo test --all-targets` は 52 件
  （既存 38 件 + 目次 14 件）。
- 外部クレート視点の受け入れテスト 5 件を一時作成し、すべて pass したのち
  削除した。入れ子 nav の title/href/depth の厳密一致、`epub:type` の無い nav の
  フォールバック、`playOrder` を崩した NCX の文書順、nav と NCX の併存時の
  nav 優先、情報源やエントリの欠落で空、壊れた XML・逸脱 href・10001 件の
  拒否を確認した。
- 実走メモ: この機能はループの 1 ラウンド目でプロセスが OS に停止された
  （Android / Termux のバックグラウンド制限）。残った実装をレビューし、
  上記のゲートと受け入れテストで検証したうえで着地した。次回以降は
  `termux-wake-lock` を取得してから実走する。

## 解決済み

- アンカーの保持と目次からのページ内移動: ADR-0007 / 0046 で実装。
- `landmarks` / `page-list` の活用: ADR-0027 / 0028 で UI を実装。
- UI への接続: ADR-0029 で履歴一覧、ADR-0075 でショートカットヘルプを実装。

## 未決定

- 目次パネルの折りたたみ・検索（現状はドロップダウン形式）。
