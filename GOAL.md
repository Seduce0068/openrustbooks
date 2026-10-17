# Goal

`openrustbooks-core` の EPUB 目次に、フラグメント（文書内アンカー）の保持を追加する。
`EpubTocEntry` に `fragment` を足し、nav / NCX の `href` から `#...` を取り出して
保持する。読み取り専用・新規依存なし・ネイティブと wasm32 の両方でビルドできること。

## Scope

### 背景

- ADR-0006 は目次を平坦なリストで返すことを決め、フラグメント（`#fragment`）を
  落とすと決めた。理由は `href` を本文の章と一致させ、UI が突き合わせられるように
  するためだった。
- 実際の EPUB には `ch.xhtml#section-3` のように文書内の位置を指す目次・見出しが
  ある。リーダーの「目次・ページ・章を移動できる」検証条件には章内の位置への
  移動が含まれる。そこでフラグメントを第 2 のフィールドとして保持し、`href` の
  一致規則は変えない。
- 本作業はフラグメントの保持まで。UI への接続、実際のスクロール移動、
  章タイトルの取得は範囲外。

### 成果物

- `core/src/epub.rs`: 変更（`EpubTocEntry.fragment`、nav / NCX の解析で
  フラグメントを分割して保持、パス部が空の href の扱い）
- `core/src/lib.rs`: 変更不要（`EpubTocEntry` は既に公開・再エクスポート済み）
- 新しい依存は追加しない。

### 公開 API（変更点のみ。その他の名前とシグネチャを変えない）

```rust
pub struct EpubTocEntry {
    /// 表示名（実体参照を戻し、空白を 1 つにまとめたもの）。
    pub title: String,
    /// 本文と同じ規則で解決済みのアーカイブ内パス（従来どおりフラグメントを含まない）。
    pub href: String,
    /// 生の href の最初の `#` の後ろ（`#` を除き、前後の空白を除いたもの）。
    /// 元の綴りのまま保持し、パーセント復号しない。`#` が無い・空なら None。
    pub fragment: Option<String>,
    /// 入れ子の深さ。最上位を 0 とする。
    pub depth: usize,
}

impl EpubBook {
    /// 目次を返す。情報源が無い場合は空。
    pub fn toc(&self) -> CoreResult<Vec<EpubTocEntry>>;
}
```

`derive`（`Debug`・`Clone`・`PartialEq`・`Eq`）は変えない。
`EpubChapter`・`EpubMetadata`・`EpubBook` の他の API は変えない。

### 設計上の必須事項

1. フラグメントの取り出し:
   - 生の `href` / `src` を「最初の `#` の前」と「後ろ」に分ける。
   - 後ろの部分は前後の空白（trim）を除いたもの。空なら `None`。
   - 内部の `#` は区切りにしない（`#a#b` のフラグメントは `a#b`）。
   - パーセントエンコードはそのまま保持する（`#%E3%81%82` は `%E3%81%82`）。
   - XML の実体参照は属性の復号で戻った後の文字列を扱う
     （`#a&amp;b` のフラグメントは `a&b`）。
2. パス部が空の href はエントリにしない（エラーにしない）:
   - 対象: `href` 属性が無い、`href=""`、`href="  "`、`href="#x"`（フラグメントのみ）。
   - 該当する `<a>` / `navPoint` は読み飛ばし、残りのエントリの解析は続ける
     （目次全体を失敗させない）。
   - これは従来の挙動（空の `href` は `CoreError::Archive`）の変更であり、
     本契約による意図した更新である。`resolve_href` 自体は変更しない
     （空パスでは引き続きエラーを返し、manifest の検証はそのまま）。
3. nav（EPUB3）:
   - `<a href="ch1.xhtml#s1">` は `href == "OEBPS/ch1.xhtml"`、
     `fragment == Some("s1")`。
   - フラグメント無しの `<a href="ch1.xhtml">` は `fragment == None`。
   - フラグメントの分割は `<a>` の開始時に 1 回行う。
4. NCX（EPUB2）:
   - `<content src="Text/ch1.xhtml#p1"/>` は解決済みパスと `fragment == Some("p1")`。
   - `<text>` や `<content>` が無い `navPoint` はエントリにしない（従来どおり）。
5. 既存の挙動を変えない:
   - `title`・`href`・`depth` の規則は従来どおり。既存の 52 テストがすべて通る。
     テストは `fragment` フィールド追加に伴う期待値の追加のみ許可する
     （アサーションの意味を変えない）。
   - 逸脱したパス（`../../evil.xhtml#x`）は従来どおり `CoreError::Validation`。
   - 情報源が無い・エントリが無い場合は従来どおり空。
   - エントリ数が 10000 を超えたら `CoreError::Validation`（従来どおり。
     読み飛ばした `<a>` は数えない）。
6. 分割規則は 1 か所のヘルパーに置き、nav と NCX で共有する
   （同じ規則を二重実装しない）。`NavAnchor`・`NcxNavPoint`・`EpubTocEntry` の
   3 構造体に `fragment: Option<String>` を足す。
7. 変更範囲は `core/src/epub.rs` を基本とする。`lib.rs` は変更不要。
   `cargo fmt` が整形する範囲は許容する。`archive.rs`・`error.rs`・`docs/`・
   `desktop/`・`web/`・`Cargo.toml` を変更しない。

### 実装時に効く実測済みの注意（`quick-xml` 0.42）

- 要素名・属性名は `local_name()` の `&str` 比較。`b"..."` のバイト列比較は不可。
- 属性値の復号は `normalized_value(XmlVersion::Implicit1_0)`。既存の
  `attribute_value` / `unescape_attribute` を再利用する。
- テキストは reader が復号済み。`&amp;` などは `Event::GeneralRef` に来るので
  `named_reference` で戻す（既存の `collect_anchor_title` 系を変えない）。
- nav の `<a>` は開始で `NavAnchor` を作り、終了で確定する。空パスの `<a>` は
  `NavAnchor` を作らない（その場合の既存の終了処理は変更不要）。
- NCX は `points` を走査してエントリを確定する。パス空の `navPoint` は
  その走査で `continue` する。

### 必須テスト（`core/src/epub.rs` の `mod tests`、外部ファイルを使わない）

1. nav: `ch1.xhtml#s1` で `fragment == Some("s1")` かつ `href` が章と同じ値
   （2 階層の一方にフラグメント、他方に無しを混ぜる）。
2. nav: フラグメント無しで `None`。
3. nav: `ch1.xhtml#`（`#` の後ろが空）で `None`。
4. nav: `#s1`（パスのみ空）はエントリにならず、他のエントリは残る。
5. nav: `href` 無し・`href=""`・`href="  "` はエントリにならない。
6. nav: `#%E3%81%82` は `Some("%E3%81%82")`（復号しない）。
7. nav: `ch1.xhtml#a#b` は `Some("a#b")`。
8. nav: `ch1.xhtml#a&amp;b` は `Some("a&b")`、`ch1.xhtml# s1 ` は `Some("s1")`。
9. NCX: `Text/ch1.xhtml#p1` は `Some("p1")`。
10. NCX: `#p` はエントリにならず、他のエントリは残る。
11. 既存の 52 テストが引き続き通る（`cargo test` の件数が減らない）。
12. パス部が空の `<a>` を挟んでも、後続のエントリの `depth` が従来の規則
    （最上位 = 0）のままである。

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
- UI への接続、実際のスクロール移動、章タイトルの `EpubChapter` への埋め込み、
  画像の扱い、`landmarks` / `page-list` は範囲外。
- `desktop/` と `web/` を変更しない。`core` 内で完結させる。
- 無関係なリファクタリング、無関係なファイルの変更をしない。
- プロダクションコードで `unwrap()` / `expect()` / `panic!` / `todo!` を使わない。
  `unsafe` を使わない。
- 新しい依存を追加しない。
- ネットワークアクセスを必要とするテストを書かない。
- コメントと doc コメントは日本語で簡潔に書く。
- リモートリポジトリへ push しない。
- `goal_loop.rs`、`GOAL.md`、`.opencode/agents/verifier.md` を書き換えない。
