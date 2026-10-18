---
type: Decision
title: "ADR-0008: EPUBの表紙画像と章の表示名を取得する"
description: "表紙の宣言の解決順と、目次を優先する章タイトルの解決、欠落と破損の扱いを記録する。"
tags:
  - ADR
  - epub
  - core
  - navigation
generated:
  by: opencode
  at: "2026-10-18T13:52:51+09:00"
status: accepted
---

## 背景

`docs/requirements.md` の REQ-002 は EPUB の閲覧を求める。ADR-0005 は本文と章の
読み出しを実装し、画像の扱いを未決定として残した。ADR-0006 は「章タイトルを
`EpubChapter` に埋め込む」案を却下し、UI が `toc()` と `chapters()` を `href` で
突き合わせる方針を示したが、章タイトルの自動取得は未決定のままだった。
ADR-0007 の後、章の XHTML とリソースの読み出し（`chapter_html` /
`chapter_resource`）を追加した際に、表紙と章タイトルは範囲外とされた。

表紙の指定方法は版によって異なる。

| 版 | 情報源 | 場所 |
| --- | --- | --- |
| EPUB3 | `properties` の `cover-image` | manifest の `item` |
| EPUB2 | `<meta name="cover">` の `content` | metadata の `meta` |

EPUB3 の本でも `meta` が残っている例があり、優先順位を決める必要がある。

## 決定

`EpubCoverImage { id, href, media_type, bytes }`、`EpubBook::cover_image()`、
`EpubBook::chapter_title(index)` を公開する。どちらも読み取り専用であり、
新規依存は無い。

### 表紙画像

- `properties` に `cover-image` を含む item（EPUB3）を優先し、無ければ
  `<meta name="cover">` の `content` が指す `id` の item（EPUB2）を使う。
  EPUB3 の宣言がある場合は EPUB2 へは落ちない。
- `id` は文書順の先勝ち。同じ `id` を持つ `item` が複数ある場合は最初の 1 件を
  使う。
- `media-type` が空（未宣言）の item は受け入れ、`image/` 以外と明示されて
  いる item は表紙にしない（表紙ページの XHTML を返さないため）。
- `href` は manifest と同じ `resolve_href` で解決する。エントリがアーカイブに
  無い場合は `None`（表紙は補助情報であり、無いことで本文の読み出しを
  妨げない）。`%XX` の復号フォールバックは章と同じ規則を使う。
- 欠落以外の読み出し失敗（暗号化・非対応圧縮・上限超過）は
  `CoreError::Archive` / `CoreError::Validation` として報告する。黙って
  `None` にすると破損を隠すため。

### 章の表示名

- 目次で `href` が一致する最初のエントリの表示名を優先し、一致する
  エントリが無ければ章の XHTML の `<head>` の `<title>` を使う。両方
  無ければ `None`。
- `<body>` に入った後に現れる `<title>`（インライン SVG のアクセシブル名
  など）は章の表示名ではないため対象にしない。
- 実体参照は定義済みの 5 つと数値参照だけを文字に戻し、前後の空白と連続する
  空白をまとめる（本文抽出と同じ規則）。
- 範囲外の `index` は `CoreError::DocumentNotFound`。目次または章の
  読み出し・解析の失敗はそのまま報告し、`<title>` へは落とさない。目次が
  壊れている本で、本のタイトルを章名として返さないため。
- 呼び出しごとに目次と章を読み直す（ADR-0005 / ADR-0006 の非キャッシュ方針）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 目次優先 + `<title>` フォールバック（採用） | 目次は表示名で、`href` で章と対応付く |
| `<title>` を先に読む | 本のタイトルが章名になる本が多い |
| 読み出し失敗も `None` にする | 破損を「表紙なし」として隠す |
| 目次失敗でも `<title>` を使う | 壊れた目次を黙って回避してしまう |
| href とバイト列を別の API にする | 呼び出しが 2 回になり状態が変わり得る |
| `open` 時に表紙を読む | 開くだけで画像をメモリに載せてしまう |
| 未宣言の `media-type` を拒否 | 不完全な OPF を排除してしまう |
| `EpubChapter` にタイトルを埋め込む | ADR-0006 で却下済み |

## 影響

- UI は表紙と章名を core の API だけで表示できる。
- `chapter_title()` は呼び出しごとに目次を解析し、必要なら章を読み直す。
  章一覧の表示では `toc()` の利用を推奨する。
- `landmarks` / `page-list` は ADR-0009 で対応した。アンカー単位の移動、
  サニタイズ、リソース URL の書き換え、画像のデコード・縮小は未対応。

## 検証

- 5 ゲートが緑。`cargo test --all-targets` は 92 件（前回 76 件 + 16 件）。
- 追加した 16 件は、EPUB3 / EPUB2 の表紙、EPUB3 の優先、宣言なし、`id` 不明、
  エントリ欠落、非画像、`%XX`、読み出し失敗の報告、目次の表示名、
  `<title>` フォールバック、正規化、`<head>` 限定、空の `<title>`、範囲外、
  壊れた XML を確認する。
- 読み出し失敗の報告は、表紙エントリの圧縮方式だけを非対応の値に書き換えた
  ZIP を使い、一覧の走査は通るが読み出しだけが失敗する状況で確認した。
- 設計レビューを 1 ラウンド実施した。採用: `<head>` 限定の `<title>` と、
  欠落以外の読み出し失敗の報告。却下: 全失敗を `None` にする案と、目次失敗時に
  `<title>` へ落とす案（どちらも上記の理由で不採用）。

## 未決定

- `landmarks` / `page-list` の活用。
- アンカー（`#fragment`）単位の移動。
- サニタイズ、リソース URL の書き換え、画像のデコード・縮小。
- UI（WebView）への接続と、表紙一覧の表示。
