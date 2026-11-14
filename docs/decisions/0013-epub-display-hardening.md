---
type: Decision
title: "ADR-0013: EPUB 表示経路を fail-closed に強化する"
description: "画像参照の URL 正規化、外部リンクの opener 分離、EPUB サイズ上限の決定を記録する。"
tags:
  - ADR
  - epub
  - web
  - wasm
  - security
generated:
  by: dsh
  at: "2026-11-15T01:25:00+09:00"
status: accepted
---

## 背景

ADR-0012 は章 XHTML の許可リストサニタイズと画像の Blob URL 表示を実装し、
`img` の `src` は相対参照だけを許可した。しかし判定は
`core::sanitize::is_local_reference` の単純な文字列比較で、ブラウザが URL を
解釈する前に行う正規化に追随していなかった。

- ブラウザは URL の前後にある C0 制御文字と空白を除き、タブ・LF・CR を
  どこにあっても除き、特殊スキームの URL ではバックスラッシュを `/` として
  扱う。そのため `<img src="\\evil.com/x.png">` はプロトコル相対 URL として
  外部リクエストになるが、旧実装は相対参照として通していた。
- Reader は解決できない `src` を削除するため実害は無かったが、Markdown
  プレビューはサニタイズ結果をそのまま `inner_html` に設定するため、
  外部画像の読み込み経路が残っていた。
- 外部リンクは `window.open(url, "_blank")` で開いており、`noopener` /
  `noreferrer` を明示していなかった。
- EPUB 全体を `File.array_buffer()` でメモリに読み込む前にサイズ上限が
  無かった。`ArchiveLimits::max_total_size` は 256 MiB で、これを超える本は
  core でも開けないが、読み込み前には弾いていなかった。

## 決定

### 画像参照の判定

`is_local_reference` の判定前に、ブラウザの URL 正規化に合わせる
`normalize_reference` を通す。

- 前後の C0 制御文字と空白 (`<= ' '`) を除く。
- 残るタブ・LF・CR を除く。
- バックスラッシュを `/` に読み替える。
- 正規化後に空・先頭 `/`・スキーム付き・制御文字を含む参照を拒否する。
- `images/pic.png`・`a%20b.png` のような相対参照は従来どおり許可する。

### 外部リンク

`window.open` に `noopener,noreferrer` を指定する。サニタイザが付ける
`rel="noopener noreferrer"` と同じ扱いを、クリック処理から開く場合にも
揃える。

### EPUB のサイズ上限

`File.size()` が 256 MiB を超える EPUB は読み込まず、状態メッセージで拒否
する。開いていた本の表示は保持する。上限は `ArchiveLimits::max_total_size`
と同じ値で、これを超える本は core でも読めないため、実質的な機能低下は
無い。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| ブラウザと同じ正規化をしてから判定（採用） | ブラウザの解釈と一致し、正当な相対画像を壊さない |
| バックスラッシュを含む参照を一律に拒否 | ファイル名に含む正当な本を落とす。正規化で十分 |
| サニタイズを DOMPurify に任せる | Rust 側のテストで検証できず、依存も増える（ADR-0012 と同じ理由） |
| サイズ上限を core に追加 | core は展開後のサイズを既に制限している。読み込み前の防御は UI の責務 |
| サイズ上限を設けない | 過大なファイルでタブが落ちる。読み込み前に弾くほうが安全 |

## 影響

- プレビューの外部画像リクエスト経路を塞ぐ。Reader は `chapter_resource` で
  解決できない `src` を削除する既存の多層防御に、正規化の判定が加わる。
- 外部リンクは opener を渡さずに開く。
- 256 MiB を超える EPUB は開かない。これは core の上限と一致する。
- SVG 画像は従来どおり Blob URL で表示する。`<img>` 文脈ではスクリプトは
  実行されないが、blob URL を直接開いた場合の扱いは未決定のまま残る。

## 検証

- `core` のサニタイズ 18 テスト（新規 8）。バックスラッシュ経由の
  プロトコル相対 URL、実体参照で隠したバックスラッシュ、空白入りスキーム、
  先頭の制御文字、非 http スキーム、`srcset`、スクリプト可能なコンテナ、
  バックスラッシュ区切りの相対参照を確認する。
- `cargo test --all-targets --locked` は 113 件成功。
- `cargo fmt --all -- --check`、`cargo check --all-targets --locked`、
  `cargo clippy --all-targets --locked -- -D warnings` が成功。
- `cargo check -p openrustbooks-core --target wasm32-unknown-unknown --locked` と
  `cargo check -p openrustbooks-web --target wasm32-unknown-unknown --locked` が成功。
- `wasm-bindgen-cli 0.2.129` と `trunk 0.22.1` を使った
  `trunk build --offline --dist <一時出力先>` が成功。
- CI は push 時に `onoderakeita/workflows/rust-gates.yml@v1` で検証される。

## 未決定

- SVG 画像の Blob URL を直接開いた場合のスクリプト実行余地。
- 深い入れ子の XHTML によるスタック消費。
- EPUB 内 CSS の安全な適用（`url()` の除去、スコープ付きスタイル）。
- 読み込み時のメモリコピー（`ArrayBuffer`・`Vec`・`ArchiveReader` の複製）。
- 相対リンクによる章間移動と、履歴・読書位置の保存（ADR-0012 から継続）。
