# Goal

Web 版 EPUB リーダーの表示経路を fail-closed に強化する。ブラウザの URL
正規化に追随しない画像参照の判定、外部リンクの opener 漏れ、過大な EPUB の
読み込みを塞ぎ、敵対的入力をテストで固定する。

## Scope

### 背景

ADR-0012 は章 XHTML の許可リストサニタイズと画像の Blob URL 表示を実装し、
`img` の `src` は相対参照だけを許可した。しかし判定は
`core::sanitize::is_local_reference` の単純な文字列比較で、ブラウザが URL を
解釈する前に行う正規化（前後の C0 制御文字と空白の除去、タブ・LF・CR の
除去、特殊スキームでのバックスラッシュの `/` 扱い）に追随していない。
そのため `<img src="\\evil.com/x.png">` のような値が残り、Markdown プレビュー
のようにサニタイズ結果をそのまま `inner_html` に設定する経路で外部
リクエストになり得る。また、外部リンクの `window.open` は `noopener` /
`noreferrer` を指定しておらず、EPUB 全体をメモリに読み込む前にサイズ上限も
無い。

### 成果物

- `core/src/sanitize.rs`: 変更（参照の正規化、敵対的入力のテスト追加）
- `web/src/lib.rs`: 変更（`window.open` の features、EPUB のサイズ上限）
- `docs/decisions/0013-epub-display-hardening.md`: 追加（決定と検証の記録）
- `GOAL.md`: 本契約

### 必須事項

1. 画像参照の判定:
   - 判定の前にブラウザの URL 正規化に合わせる。前後の C0 制御文字と空白を
     除き、残るタブ・LF・CR を除き、バックスラッシュを `/` に読み替える。
   - 正規化後に空・先頭 `/`・スキーム付きを拒否する。正規化後に残る
     制御文字も拒否する（fail-closed）。
   - `images/pic.png`・`a%20b.png` のような既存の相対参照は許可したまま。
2. 外部リンク:
   - `window.open` に `noopener,noreferrer` を指定する。
3. サイズ上限:
   - `File.size()` が 256 MiB を超える EPUB は読み込まず、状態メッセージで
     拒否する。開いていた本の表示は保持する。
   - 256 MiB は `ArchiveLimits` の `max_total_size` と同じで、これを超える
     本は core でも読めない。
4. 既存の挙動を変えない:
   - 既存の 105 テストが引き続き通る。公開 API を変えない。
   - SVG 画像の表示は維持する（残存リスクは ADR に記録する）。

### 必須テスト

- `core/src/sanitize.rs` の `mod tests` に敵対的入力を追加する。
  - `\\evil.com/x.png`・`&#92;&#92;evil.com/x.png`・`\/evil.com/x.png` を拒否
  - `java&#9;script:` のような空白入りスキームを拒否
  - 先頭に C0 制御文字を挟んだ `//host` を拒否
  - `//host`・`/abs`・`data:`・`DATA:`・`blob:` を拒否（既存分と合わせる）
  - `srcset`・`style`・`script`・`svg`・`template`・`noscript`・
    `iframe srcdoc`・イベントハンドラを除去
  - `x\y.png` は `/` 正規化後に相対として許可する
- 既存の 105 テストが通る（合計 105 件以上）。

## Acceptance criteria

- 上記の必須事項がコードとテストで確認できる。
- 次がすべて成功する。
  - `cargo fmt --all -- --check`
  - `cargo check --all-targets --locked`
  - `cargo test --all-targets --locked`
  - `cargo clippy --all-targets --locked -- -D warnings`
  - `cargo check -p openrustbooks-core --target wasm32-unknown-unknown --locked`
  - `cargo check -p openrustbooks-web --target wasm32-unknown-unknown --locked`
  - `trunk build --offline --dist <一時出力先>`（web）
- CI は push 時に `rust-gates.yml@v1` で検証される（本作業では push しない）。

## Constraints

- 新規依存を追加しない。読み取り専用の方針を維持する。
- `desktop/` を変更しない。
- プロダクションコードで `unwrap()` / `expect()` / `panic!` / `todo!` を
  使わない。`unsafe` を使わない。
- コメントと doc コメントは日本語で簡潔に書く。
- 無関係なリファクタリングをしない。
- リモートリポジトリへ push しない。
