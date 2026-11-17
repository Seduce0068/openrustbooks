---
type: Decision
title: "ADR-0009: EPUB3 nav の landmarks と page-list を読む"
description: "EPUB3 nav の landmarks と page-list を既存の目次項目型で返す決定を記録する。"
tags:
  - ADR
  - epub
  - core
  - navigation
generated:
  by: opencode
  at: "2026-10-18T14:04:05+09:00"
status: accepted
---

## 背景

ADR-0006 は目次を `EpubBook::toc()` として実装したが、EPUB3 nav の
`landmarks` / `page-list` は未対応とした。ADR-0008 でも残課題としていた。
これらは目次と同じ nav 文書内のナビゲーションリストであり、ページ・章への
移動を UI から行うために取得 API が必要。

## 決定

`EpubBook::landmarks()` と `EpubBook::page_list()` を公開し、既存の
`EpubTocEntry { title, href, fragment, depth }` の平坦なリストを返す。

- manifest の `properties` に `nav` を含む item の XHTML だけを対象とする。
  NCX に landmarks / page-list の代替を求めない。
- `epub:type` 属性（接頭辞付き属性）のトークンが `landmarks` または
  `page-list` と一致する `<nav>` を文書順で探す。該当 nav が無ければ空を返す。
  `toc()` のように最初の `<nav>` へはフォールバックしない。
- 各リストは既存の nav エントリ収集処理を使う。`href` 解決、フラグメント保持、
  深さの計算、エントリ数上限、XML エラーとパス逸脱の扱いは `toc()` と共通。
- nav エントリがアーカイブに無い場合は空を返す。解析・検証エラーは伝播する。
- 内部キャッシュは追加せず、呼び出しごとに nav を読み直す。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| `EpubTocEntry` を共用（採用） | title、href、fragment、depth が同じ表現であり、型の重複を避ける |
| 専用構造体を作る | 現時点で異なるフィールドや不変条件が無い |
| 指定 type が無いとき最初の nav を返す | landmarks / page-list を目次など別用途の nav と誤認するため不採用 |
| NCX から補完する | 対応する標準形式・意味を追加せず、EPUB3 nav に限定する |

## 影響

- UI は landmarks とページ一覧を、目次と同じ href・fragment の規則で扱える。
- API は `CoreResult<Vec<EpubTocEntry>>` とし、情報源や該当 nav が無い場合は
  空を返す。
- アンカーへの実際の移動、ページ番号とレンダリングページの対応付け、UI 表示は
  引き続き未対応。

## 検証

- 5 ゲートが緑。`cargo test --all-targets` は 95 件。
- nav 内に複数の list type がある場合の選択、href・fragment・depth の保持、
  該当 nav が無い場合の空リスト、パス逸脱の拒否をテストした。

## 解決済み

- `page-list` の UI: ADR-0028 でページ番号入力による移動を実装。
- landmarks の UI: ADR-0027 でドロップダウンによる移動を実装。

## 未決定

- 現在表示中のページ番号の自動表示（ADR-0028 から継続）。
