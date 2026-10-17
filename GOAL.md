# Goal

`openrustbooks-core` に EPUB の本文読み出しの中核を実装する。
ZIP 閲覧（`core/src/archive.rs`）の上に、`META-INF/container.xml` →
OPF（メタデータ・manifest・spine）→ 章本文のテキスト抽出までを載せる。
読み取り専用でファイル I/O に依存せず、ネイティブと wasm32 の両方でビルドできること。

## Scope

### 背景

- `docs/requirements.md` の REQ-002 は EPUB「本文の閲覧」を求める。
- 同書の検証条件は「目次・ページ・章を移動できる」ことを求めるが、
  本作業は本文と章の読み出しまで（目次 UI との接続は範囲外）。
- `docs/decisions/0003-native-wasm-target-split.md` により、共通コアは
  wasm32 でもビルドできる必要がある。
- `core/src/archive.rs` に読み取り専用の `ArchiveReader` があり、
  `is_epub()` で mimetype を確認できる。EPUB 解析はこの型の上に実装する。

### 成果物

- `core/src/epub.rs`（新規）
- `core/src/lib.rs`: `pub mod epub;` と主要型の再エクスポート
  （`pub use epub::{EpubBook, EpubChapter, EpubMetadata};`）
- `core/Cargo.toml`: 次の依存を追加する

  ```toml
  quick-xml = { version = "0.42", default-features = false }
  ```

  （pure Rust。XML パーサとしてのみ使う。デフォルト機能は空なので追加機能は不要。）
- `core/src/error.rs` と `core/src/archive.rs` は変更しない（既存の
  `CoreError` 変種をそのまま使う）。

### 公開 API（名前とシグネチャを変えない）

```rust
pub struct EpubMetadata {
    pub title: Option<String>,
    pub creators: Vec<String>,       // 文書順を保つ
    pub language: Option<String>,
    pub identifier: Option<String>,
    pub publisher: Option<String>,
    pub date: Option<String>,
    pub modified: Option<String>,    // <meta property="dcterms:modified">
    pub version: Option<String>,     // package 要素の version 属性
}

pub struct EpubChapter {
    pub id: String,                 // manifest の id
    pub href: String,               // OPF の位置を基準に解決済みのアーカイブ内パス
    pub media_type: String,
    pub linear: bool,               // itemref の linear 属性（既定 true）
}

/// ZIP バイト列・解析済み OPF・章一覧を所有する。
pub struct EpubBook { /* ArchiveReader、metadata、chapters を所有 */ }

impl EpubBook {
    pub fn open(bytes: &[u8]) -> CoreResult<Self>;
    pub fn with_limits(bytes: &[u8], limits: ArchiveLimits) -> CoreResult<Self>;
    pub fn metadata(&self) -> &EpubMetadata;
    pub fn chapters(&self) -> &[EpubChapter];
    pub fn chapter_bytes(&self, index: usize) -> CoreResult<Vec<u8>>;
    pub fn chapter_text(&self, index: usize) -> CoreResult<String>;
}
```

`EpubBook` は `Send + Sync` を満たすこと（`Rc`・`RefCell`・`Mutex` を使わない）。
`EpubBook` は入力バイト列を所有し、`open` の戻り値以降は借用を持たない。

### 設計上の必須事項

1. `open` / `with_limits` の手順:
   1. `ArchiveReader::with_limits(bytes, limits)` でアーカイブを開く。
   2. `is_epub()` が false なら `CoreError::UnsupportedFormat` を返す。
   3. `META-INF/container.xml` を読み、最初の `<rootfile>` の `full-path` を得る。
      読み出し・解析に失敗したら `CoreError::Archive`。
   4. その OPF を読み、メタデータ・manifest・spine を解析する。
      OPF の読み出し・解析に失敗したら `CoreError::Archive`。
   5. 失敗はすべて `Result` で返し、panic しない。
2. パス解決（`href` → アーカイブ内パス）:
   - 最初に `#` 以降（フラグメント）を切り落とし、前後の空白を除く。
   - `://` や `mailto:` のようなスキーム付き URL、先頭 `/` は
     `CoreError::Archive`（不正な OPF）。
   - OPF のあるディレクトリを基準に `.` と `..` を字句的に解決する。
     解決結果がルートを越える場合は `CoreError::Validation`。
   - 解決したパスを `EpubChapter::href` に入れる（ディレクトリは `''`、
     ルート直下は `''` を挟まない。例: OPF が `OEBPS/sub/content.opf` で
     href が `../ch1.xhtml` なら `OEBPS/ch1.xhtml`）。
3. `chapter_bytes(index)`:
   - 範囲外の index は `CoreError::DocumentNotFound`。
   - `href` でエントリを読む。存在しない場合、`href` に `%` が含まれていれば
     `%XX`（16 進 2 桁）を 1 バイトに復号した名前でも試す。復号結果が
     不正な UTF-8 なら元の名前を使う。`+` は空白にしない。
     どちらも無ければ最初のエラーのまま `DocumentNotFound`。
4. `chapter_text(index)`:
   - バイト列を UTF-8 として復号し（先頭の BOM は除去）、XML として解析する。
     不正な UTF-8、XML の構文エラーは `CoreError::Archive`。
   - `head`・`script`・`style` の内側のテキストは入れない。
   - ブロック要素（`p` `div` `h1`〜`h6` `li` `blockquote` `section` `article`
     `pre` `tr` `td` `th`）の終了タグと `br` で改行を入れる。
   - 実体参照は `&amp;` などを文字に戻す。`&#x30;` などの数値参照も戻す。
     定義済み 5 つ（`amp` `lt` `gt` `quot` `apos`）以外の名前付き参照は無視する。
   - 正規化: 行ごとに空白の連続を 1 つの半角空白にまとめ、空行を落とし、
     改行で連結する。先頭・末尾の空白は残さない。
   - 例: `<h1>T</h1>\n<p>a &amp; b</p>\n<p>c &lt;d&gt; &#x30; &foo;</p>` は
     `"T\na & b\nc <d> 0"` になる。
5. OPF の解析:
   - 要素名は接頭辞を無視して比較する（`local_name()` を使う）。
     `opf:package` や `dc:title` のような接頭辞付きでも解析できること。
   - メタデータ: `dc:title`（最初）、`dc:creator`（すべて・文書順）、
     `dc:language`（最初）、`dc:identifier`（`package` の `unique-identifier`
     属性と同じ `id` を持つものを優先し、無ければ最初）、`dc:publisher`（最初）、
     `dc:date`（最初）。テキストは実体参照を戻し、空白を 1 つにまとめる。
   - `<meta property="dcterms:modified">` の内容を `modified` に入れる。
     属性が無い `meta`（EPUB2 形式）は無視する。
   - manifest の `item`（`id` `href` `media-type`、任意で `properties`）を集める。
   - spine の `itemref` の順に章を作る。`linear` 属性が `no` のときだけ false。
   - `idref` に対応する `item` が無い場合は `CoreError::Archive`（黙って飛ばさない）。
   - `EpubChapter::media_type` は manifest の `media-type` をそのまま入れる。
6. 変更範囲: `core/src/epub.rs`、`core/src/lib.rs`、`core/Cargo.toml` のみ。
   `archive.rs`・`error.rs`・`document.rs`・`docs/`・`desktop/`・`web/` は変更しない。

### `quick-xml` 0.42 の実測済み API（そのまま使ってよい）

```rust
use quick_xml::Reader;
use quick_xml::events::{BytesRef, Event};

let mut reader = Reader::from_str(xml);
reader.config_mut().trim_text(false); // 本文抽出では false（後述）
loop {
    match reader.read_event() {
        Ok(Event::Start(e) | Event::Empty(e)) => {
            // 要素名は &str と比較する（b"rootfile" のようなバイト列比較はできない）。
            if e.local_name().as_ref() == "rootfile" {
                for attr in e.attributes().flatten() {
                    // 属性名も &str 比較。値は esc されたままなので復号する。
                    if attr.key.local_name().as_ref() == "full-path" {
                        let v: std::borrow::Cow<'_, str> = attr.unescape_value()?;
                    }
                }
            }
        }
        Ok(Event::End(e)) => { /* e.local_name().as_ref() で比較 */ }
        Ok(Event::Text(t)) => {
            // テキストは reader が復号済み。unescape() は 0.42 に存在しない。
            let s: std::borrow::Cow<'_, str> = t.into_inner();
        }
        Ok(Event::CData(c)) => {
            let s: std::borrow::Cow<'_, str> = c.into_inner();
        }
        Ok(Event::GeneralRef(r)) => {
            // `&amp;` や `&#x30;` は Event::Text ではなくここに来る。
            if r.is_char_ref() {
                // 数値参照。Err / None は無視してよい。
                let ch: Option<char> = r.resolve_char_ref().ok().flatten();
            } else {
                // 名前付き参照。r.into_inner() が "amp" / "lt" / "gt" / "quot" / "apos"。
            }
        }
        Ok(Event::Eof) => break,
        Ok(_) => {} // Comment, Decl, PI, DocType は無視してよい
        Err(err) => { /* 壊れた XML。CoreError::Archive にする */ }
    }
}
```

`&str` を入力にする `Reader::from_str` では `read_event()` が `Result<Event>` を返し、
借用バッファは不要。`trim_text(false)` で読むこと。`true` にすると
`a &amp; b` の前後の空白まで落ちて `a&b` になる（実測）。
`b"..."` との比較、`BytesText::unescape()`、`BytesStart::name()` の
`QName.0` 直接参照は 0.42 では使えない。

### テスト用の EPUB 生成（`zip` は通常の依存なのでテストから使える）

```rust
let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
let stored = zip::write::SimpleFileOptions::default()
    .compression_method(zip::CompressionMethod::Stored);
let deflated = zip::write::SimpleFileOptions::default()
    .compression_method(zip::CompressionMethod::Deflated);
writer.start_file("mimetype", stored)?;
writer.write_all(b"application/epub+zip")?;
writer.start_file("META-INF/container.xml", deflated)?;
writer.write_all(container_xml.as_bytes())?;
// 以降も同様。最後に:
let bytes: Vec<u8> = writer.finish()?.into_inner();
```

`mimetype` は必ず最初のエントリにし、`Stored` で書く
（`is_epub()` の判定条件）。

### 必須テスト（`core/src/epub.rs` の `mod tests`、外部ファイルを使わない）

1. EPUB3 相当（mimetype＋container.xml＋`OEBPS/content.opf`＋
   `OEBPS/ch1.xhtml`＋`OEBPS/ch2.xhtml`＋`OEBPS/nav.xhtml`）を組み立て、
   `metadata()` の title・creators（2 件・文書順）・language・identifier・
   publisher・date・modified・version が期待どおり。
2. `chapters()` が spine の順で 2 件、`id`・`href`（`OEBPS/ch1.xhtml`）・
   `media_type`・`linear`（`linear="no"` の章は false、既定は true）。
3. `chapter_text(0)` が `<h1>`／`&amp;`／`&#x30;`／`<style>` 内テキストを
   含む入力から、正規化済みの期待文字列を正確に返す（前後の空白なし）。
4. `chapter_text` が `head` の `title` と `script` の内容を含まない。
5. `chapter_bytes(1)` が書き込んだバイト列と一致する。
6. 範囲外の index で `chapter_bytes` / `chapter_text` が
   `CoreError::DocumentNotFound`。
7. mimetype を持たない通常の ZIP で `open` が
   `CoreError::UnsupportedFormat`。
8. `META-INF/container.xml` 欠落、`rootfile` 要素なし、OPF 欠落、
   container.xml が壊れた XML、OPF が壊れた XML のそれぞれで
   `CoreError::Archive`（panic しない）。
9. `idref` が manifest に無い spine で `open` が `CoreError::Archive`。
10. href が `../../evil.xhtml` のようにルートを越える OPF で
    `open` が `CoreError::Validation`。
11. OPF が `OEBPS/sub/content.opf` にあり、href が `../ch1.xhtml` のとき
    `href` が `OEBPS/ch1.xhtml` に解決される。
12. href の `#fragment` が切り落とされる（`ch1.xhtml#s1` → `OEBPS/ch1.xhtml`）。
13. 接頭辞付き OPF（`opf:package`・`opf:manifest`・`opf:spine`・
    `opf:itemref`・`dc:title`）でも 1・2 と同じ結果になる。
14. `ch 1.xhtml` という実エントリを `ch%201.xhtml` で参照したとき
    `chapter_text` が読める。
15. `with_limits` で `max_entries` を 2 にして `CoreError::Validation`。
16. `EpubBook` が `Send + Sync` であることをコンパイル時に検証する
    （`fn assert_send_sync<T: Send + Sync>() {}` をテスト内で呼ぶ）。

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

- 読み取り専用。EPUB の作成・書き換え・DRM 解除は作らない。
- 目次（nav / NCX）の解析、章タイトルの取得、画像の扱いは範囲外。
- `Document` 型・`DocumentFormat` との統合は範囲外。
- `desktop/` と `web/` を変更しない。`core` 内で完結させる。
- 無関係なリファクタリング、無関係なファイルの変更をしない。
- プロダクションコードで `unwrap()` / `expect()` / `panic!` / `todo!` を使わない。
  `unsafe` を使わない。
- 新しい依存を追加しない（上記の `quick-xml` 以外）。
- ネットワークアクセスを必要とするテストを書かない。
- コメントと doc コメントは日本語で簡潔に書く。
- リモートリポジトリへ push しない。
- `goal_loop.rs`、`GOAL.md`、`.opencode/agents/verifier.md` を書き換えない。
