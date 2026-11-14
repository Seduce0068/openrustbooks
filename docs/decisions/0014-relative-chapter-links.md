---
type: Decision
title: "ADR-0014: 本文内の相対リンクから章へ移動する"
description: "相対リンクの解決を core に置き、Web の本文内リンクから章間移動とアンカー移動を行う決定を記録する。"
tags:
  - ADR
  - epub
  - web
  - wasm
generated:
  by: dsh
  at: "2026-11-15T01:48:00+09:00"
status: accepted
---

## 背景

ADR-0012 は目次からの章移動と、本文内の `#fragment` リンクのスクロールを
実装した。しかし `ch2.xhtml` のような相対リンクによる章間移動は未対応で、
クリックしても何も起きなかった（ADR-0012・ADR-0013 の未決定）。本文から
別の章の注釈・図・節へ移動する本では、目次に戻らないと読めない状態だった。

core には章の `href` を基準にした `resolve_reference` と spine の章一覧が
あり、`%XX` の復号も既存の `percent_decode` を使える。解決規則を UI 側で
作り直すと、パス脱出や `%XX` の扱いが二重実装になる。

## 決定

### 解決は core に置く

`EpubBook::resolve_chapter_link(index, href)` を追加し、`ChapterLink` を返す。

- `split_fragment` でパスとフラグメントに分ける。
- パスが空なら、フラグメントがあれば同じ章、無ければ `None`。
- スキーム付き URL・先頭 `/` は `None`（外部・絶対参照へは移動しない）。
- `resolve_reference(章の所属ディレクトリ, パス, "章リンクの href")` で解決し、
  エラー（空・絶対・スキーム・ルート越え）は `None`。
- 解決したパスを spine の章の `href` と比較する。`%XX` は復号した形でも
  比較する。一致する章が無ければ `None`。
- 範囲外の `index` は `DocumentNotFound`。

`None` は「移動しない」を意味し、外部や本の外へは移動しない（fail-closed）。

### Wasm

`WasmEpubBook::resolve_chapter_link(index, href)` は
`{ chapterIndex, fragment }` か `null` を返す。

### Web

本文内のリンクのクリックを次の順で扱う。

1. `#fragment` だけのリンクは同じ章のフラグメントへスクロールする（従来どおり）。
2. `http` / `https` / `mailto` は新しいタブで開く（従来どおり、
   `noopener,noreferrer`）。
3. それ以外は `resolve_chapter_link` で解決する。同じ章ならフラグメントへ
   スクロールし、別の章ならその章を表示してフラグメントへスクロールする。
4. 解決できないリンクは移動せず、状態メッセージを表示する。

章を切り替えたときは本文のスクロール位置を先頭へ戻す。フラグメント指定時は
その位置へスクロールする。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| core で解決する（採用） | パス解決の規則を 1 か所に保ち、`%XX` とルート越えをテストできる |
| Web で文字列比較する | 解決規則が二重実装になり、脱出の検証も漏れやすい |
| iframe に任せてブラウザに移動させる | サニタイズと Blob URL の方針と衝突する（ADR-0012） |
| 章間リンクを無効のまま | 本文内の注釈・図への導線が無く、読書の連続性が損なわれる |

## 影響

- 本文内の相対リンクで章を移動できる。外部・本の外へは移動しない。
- 目次・前後移動を含め、章を切り替えたときに本文の先頭へ戻るようになる。
- spine に無いパス（注釈先など）は移動せず状態メッセージを表示する。
- `linear="no"` の章も spine にあれば移動先になる（現状の一覧と同じ）。
- 公開 API が 1 つ増える（`ChapterLink` と `resolve_chapter_link`）。

## 検証

- `core` の `resolve_chapter_link` 6 テスト。別章、フラグメント、フラグメント
  のみ、サブディレクトリからの親参照、`%XX` 復号、本の外・spine 外・範囲外。
- `cargo test --all-targets --locked` は 119 件成功。
- `cargo fmt --all -- --check`、`cargo check --all-targets --locked`、
  `cargo clippy --all-targets --locked -- -D warnings` が成功。
- `cargo check -p openrustbooks-core --target wasm32-unknown-unknown --locked` と
  `cargo check -p openrustbooks-web --target wasm32-unknown-unknown --locked` が成功。
- `trunk build --offline --dist <一時出力先>` が成功。
- CI（`Seduce0068/workflows/rust-gates.yml@v1`）が成功する。

## 未決定

- 戻る・進むの履歴と、読書位置（章・スクロール位置）の保存。
- リンク先が `linear="no"` の章のときの扱い（現状は移動する）。
- 注釈・脚注から本文へ戻る導線。
- EPUB 内 CSS、縦書き・ページ送り（ADR-0012・ADR-0013 から継続）。
