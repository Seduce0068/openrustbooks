# Goal

Web 版の EPUB リーダーで、章の XHTML を許可リスト方式でサニタイズして
表示する。章内の画像と表紙を Blob URL で表示し、目次フラグメントに
よる章内移動と章タイトルの表示を追加する。

## Scope

### 背景

ADR-0011 は `chapter_text()` を `<pre>` に表示する最小 UI を実装し、
XHTML の描画をサニタイズ方針が未決定であるとして範囲外にした。
core には `chapter_html()`・`chapter_resource()`・`cover_image()`・
`toc()` が揃っており、Web UI 側の表示だけが残っている。

### 成果物

- `core/src/sanitize.rs`: 追加（`ammonia` による許可リストサニタイズ）
- `core/src/lib.rs`: `sanitize` モジュールの公開
- `core/Cargo.toml`: `ammonia` の追加
- `web/src/lib.rs`: `Reader` の HTML 表示、画像・表紙の Blob URL 化、
  TOC フラグメント移動、章タイトル表示、`Preview` のサニタイズ
- `web/Cargo.toml`: `web-sys` の必要機能
- `web/index.html`: リーダーのレイアウトとスタイル
- `docs/decisions/0012-epub-html-display.md`: 決定記録
- `GOAL.md`・`README.md`・`AGENTS.md`: 契約と検証の更新

### 必須事項

1. サニタイズ:
   - `script`・`style`・`iframe`・`object`・`embed`・`form`・`input` を
     除去する。`script`・`style`・`title` は内容ごと削除する。
   - イベントハンドラ属性と `style` 属性を除去する。
   - `a` は `http`・`https`・`mailto` を許可し、`rel` を付与する。
     `javascript:` と `data:` は除去する。
   - `img` は相対 URL のみ許可し、外部 URL は除去する。
   - 安全な構造・見出し・表・ルビ・`id` は保持する。
2. 画像と表紙:
   - `chapter_resource()` で解決したバイト列を Blob URL にし、
     `img` の `src` に設定する。
   - 解決できない画像は `src` を削除する。
   - 章の切り替えと本の開き直しで Blob URL を revoke する。
   - 表紙をサイドバーに表示する。
3. 章内移動:
   - TOC の `fragment` を章選択に保持し、表示後にスクロールする。
   - 本文内の `#fragment` リンクもスクロールする。
4. 表示:
   - `Reader` の本文を `inner_html` にサニタイズ済み HTML を設定する。
   - 現在の章タイトルをツールバーに表示する。
   - `Preview` も同じサニタイザを通す。
5. 既存の挙動を変えない:
   - core の既存公開 API とテスト 95 件を壊さない。
   - EPUB の読み取り専用方針を維持する。

### 必須テスト

- core のサニタイズ:
  - script とイベントハンドラの除去
  - 埋め込み要素とフォームの除去
  - 安全な構造と `id` の保持
  - `style` 属性・`style` タグの除去
  - 相対画像 URL の保持
  - 外部画像 URL・`data:` の除去
  - `javascript:` リンクの除去と `rel` の付与
  - `html`・`head` ラッパーの除去
  - 表と span 属性の保持
- 既存の 95 テストが通る（合計 105 件以上）。

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
- CI（`onoderakeita/workflows/rust-gates.yml@v1`）が成功する。

## Constraints

- 読み取り専用。EPUB の作成・書き換えは作らない。
- CSS は適用しない。外部画像・外部リンクの埋め込みは表示しない。
- サニタイズを迂回して生 XHTML を `inner_html` に挿入しない。
- 新規依存は `ammonia` のみ。
- プロダクションコードで `unwrap()` / `expect()` / `panic!` / `todo!` を
  使わない。`unsafe` を使わない。
- ネットワークアクセスを必要とするテストを書かない。
- コメントと doc コメントは日本語で簡潔に書く。
- 無関係なリファクタリングをしない。
