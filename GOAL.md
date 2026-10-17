# Goal

`openrustbooks-core` の EPUB モジュールに、目次（EPUB3 の nav / EPUB2 の NCX）の
読み出しを追加する。読み取り専用でファイル I/O に依存せず、ネイティブと
wasm32 の両方でビルドできること。

## Scope

### 背景

- `docs/requirements.md` のリーダー検証条件は「目次・ページ・章を移動できる」
  ことを求める。
- `core/src/epub.rs` に `EpubBook`（container.xml → OPF → spine → 本文）がある。
  `EpubChapter.href` は OPF 基準で解決済みのアーカイブ内パスを返す。
- 本作業は目次の取得まで。UI への接続、アンカー（`#fragment`）単位の移動、
  章タイトルの `EpubChapter` への埋め込みは範囲外。

### 成果物

- `core/src/epub.rs`: 変更（`EpubTocEntry`、`EpubBook::toc`、manifest の
  `properties` 取得、nav / NCX の解析）
- `core/src/lib.rs`: `pub use epub::{EpubBook, EpubChapter, EpubMetadata, EpubTocEntry};`
- 新しい依存は追加しない（`quick-xml` 0.42 をそのまま使う）。

### 公開 API（名前とシグネチャを変えない）

```rust
/// 目次の 1 項目。
pub struct EpubTocEntry {
    /// 表示名（実体参照を戻し、空白を 1 つにまとめたもの）。
    pub title: String,
    /// 本文と同じ規則で解決済みのアーカイブ内パス（フラグメントは落とす）。
    pub href: String,
    /// 入れ子の深さ。最上位を 0 とする。
    pub depth: usize,
}

impl EpubBook {
    /// 目次を返す。情報源が無い場合は空。
    pub fn toc(&self) -> CoreResult<Vec<EpubTocEntry>>;
}
```

`EpubTocEntry` は `Debug`・`Clone`・`PartialEq`・`Eq` を derive する。

### 設計上の必須事項

1. 目次の情報源の優先順位:
   - manifest の `properties` に `nav` を含む `item`（`properties` は空白区切りの
     トークン列。`nav scripted` のような複数値でも `nav` を含めば対象）
     → その `href` の XHTML を EPUB3 nav として読む。
   - 無ければ media-type が `application/x-dtbncx+xml` の `item`
     → EPUB2 NCX として読む。
   - どちらも無ければ `Ok(Vec::new())`（エラーにしない）。
2. `toc()` は `&self` のまま。キャッシュ用の内部可変性を持たず、呼び出しごとに
   解析してよい。情報源のエントリがアーカイブに存在しない場合も
   `Ok(Vec::new())`。XML の構文エラーは `CoreError::Archive`。
3. EPUB3 nav（XHTML）:
   - 最初の `<nav>` のうち、キーに `:` を含みローカル名が `type` の属性
     （`epub:type` など）の値を空白区切りで見て `toc` を含むものを選ぶ。
     そのような `<nav>` が無ければ最初の `<nav>` を使う。
   - 入れ子の `<ol>` / `<li>` を文書順に辿り、`<a href="...">` ごとに
     1 エントリを作る。`title` は `<a>` の内側のテキスト（実体参照を戻し、
     空白を 1 つにまとめる）。`<a>` を持たない `<li>`（`<span>` だけの見出し）は
     エントリにしない。
   - `depth` は祖先 `<ol>` の数 − 1（最上位 = 0）。
   - `href` は `EpubChapter::href` と同じ `resolve_href` を使う。空の `href` は
     `CoreError::Archive`。
4. EPUB2 NCX:
   - `<navMap>` の下の入れ子 `<navPoint>` を文書順に辿る。
     `playOrder` 属性は無視する（文書順を正とする）。
   - `title` は同じ `<navPoint>` の `<navLabel>` の `<text>` の内容。
   - `href` は同じ `<navPoint>` の `<content>` の `src`。
   - `depth` は祖先 `<navPoint>` の数 − 1（最上位 = 0）。
   - `<text>` や `<content>` が無い `navPoint` はエントリにしない。
5. リミット: エントリ数が 10000 を超えたら `CoreError::Validation`。
   超過を検出した時点で打ち切ってよい（それ以上は読まない）。
6. `resolve_href`・`named_reference`・`normalize_text`・`read_text_entry`・
   `attribute_value`・`unescape_attribute` など既存のヘルパーを再利用し、
   同じ規則を二重実装しない。`parse_opf` は manifest の `properties` 属性も
   読む（無い場合は空文字列）。`ManifestItem` に `properties: String` を足す。
7. 既存の公開 API と挙動を変えない。`EpubChapter`・`EpubMetadata` のフィールドは
   増やさない。既存の 19 テストがすべて通ること。
8. 変更範囲は `core/src/epub.rs` と `core/src/lib.rs` のみ。
   `archive.rs`・`error.rs`・`document.rs`・`docs/`・`desktop/`・`web/`・
   `Cargo.toml` を変更しない。

### 実装時に効く実測済みの注意（`quick-xml` 0.42）

- 要素名・属性名は `local_name()` の `&str` 比較。`b"rootfile"` のような
 バイト列比較はコンパイルできない。
- 属性値の復号は `attr.normalized_value(XmlVersion::Implicit1_0)`
  （既存の `unescape_attribute` を使う）。
- テキストは reader が復号済み（`Event::Text::into_inner()`）。
  `&amp;` などは `Event::GeneralRef` に来るので `named_reference` で戻す。
- 接頭辞付き属性の判定は `attr.key.local_name().into_inner() == "type"` と
  `attr.key.as_ref().contains(':')` の組み合わせで行える
  （`QName` は `pub &str` を持つタプル構造体なので `as_ref()` が生の名前を返す）。
- 入れ子の深さは再帰ではなく明示的なカウンタで追う（深い入力でスタックを
  消費しない）。

### 必須テスト（`core/src/epub.rs` の `mod tests`、外部ファイルを使わない）

1. EPUB3 nav を 1 つ持つ EPUB で、`toc()` が入れ子の `<ol>`/`<li>`/`<a>` を
   文書順に返し、`title`・`href`・`depth` が期待どおり（2 階層）。
2. `<span>` だけの `<li>`（href 無しの見出し）がエントリにならない。
3. `<a href="ch1.xhtml#s1">` のフラグメントが落ちて `href` が章と同じ値になる。
4. `epub:type="landmarks"` の nav が先、`epub:type="toc"` の nav が後ろにある
   XHTML で、`toc` の nav が選ばれる。
5. `epub:type` を持たない `<nav>` が 1 つだけなら、それを使う。
6. `properties="nav scripted"` のように複数トークンでも nav として扱う。
7. EPUB2 NCX（入れ子の `navPoint`・`navLabel`/`text`・`content`/`src`）で
   `toc()` が入れ子の深さ付きで返る。
8. nav と NCX の両方があるとき nav を優先する。
9. どちらの情報源も無い EPUB で `toc()` が空を返す。
10. nav の XHTML が壊れた XML のとき `CoreError::Archive`。
11. 目次の `href` が `../../evil.xhtml` のとき `CoreError::Validation`。
12. 10001 個の `<a>` を持つ nav で `CoreError::Validation`（生成はテスト内で
    文字列を組み立ててよい）。
13. 既存の 19 テストが引き続き通る（`cargo test` の件数が 38 件から減らない）。

## Acceptance criteria

- 上記 API と設計上の必須事項がすべてコードとテストで確認できる。
- 既存の公開 API を壊さない（`cargo check --all-targets` が通る）。
- 次がすべて成功する。
  - `cargo fmt --all -- --check`
  - `cargo check --all-targets`
  - `cargo test --all-targets`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo check -p openrustbooks-core --target wasm32-unknown-unknown`

## Constraints

- 読み取り専用。EPUB の作成・書き換えは作らない。
- UI への接続、アンカー単位の移動、章タイトルの `EpubChapter` への埋め込み、
  画像の扱いは範囲外。
- `desktop/` と `web/` を変更しない。`core` 内で完結させる。
- 無関係なリファクタリング、無関係なファイルの変更をしない。
- プロダクションコードで `unwrap()` / `expect()` / `panic!` / `todo!` を使わない。
  `unsafe` を使わない。
- 新しい依存を追加しない。
- ネットワークアクセスを必要とするテストを書かない。
- コメントと doc コメントは日本語で簡潔に書く。
- リモートリポジトリへ push しない。
- `goal_loop.rs`、`GOAL.md`、`.opencode/agents/verifier.md` を書き換えない。
