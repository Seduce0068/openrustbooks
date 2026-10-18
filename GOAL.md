# Goal

`openrustbooks-core` の EPUB に、章の表示に使う読み出しを追加する。
`EpubBook::chapter_html`（章の XHTML を文字列で返す）と
`EpubBook::chapter_resource`（章から見た相対参照でリソースを読む）を足す。
読み取り専用・新規依存なし・ネイティブと wasm32 の両方でビルドできること。

## Scope

### 背景

- `docs/requirements.md` のリーダー検証条件は「対応サンプルを開き、
  目次・ページ・章を移動できる」。core は章のバイト列（`chapter_bytes`）と
  平文テキスト（`chapter_text`）までを読める（ADR-0005）が、UI が章を
  描画するには、章の XHTML そのものと、章が参照する画像・CSS などの
  リソースを読む API が要る。
- 表示は UI（WebView / iframe）が行う。core はサニタイズ、URL の書き換え、
  画像のデコードを行わず、XHTML とリソースのバイト列を返すところまで。
- 本作業は core の読み出しまで。UI への接続、サニタイズ、リソース URL の
  書き換え、表紙や章タイトルの取得は範囲外。

### 成果物

- `core/src/epub.rs`: 変更（`chapter_html`・`chapter_resource`、相対参照解決の
  共通ヘルパーの切り出し、モジュール doc の更新）
- `core/src/lib.rs`: 変更不要（`EpubBook` は既に公開済み）
- 新しい依存は追加しない。

### 公開 API（変更点のみ。その他の名前とシグネチャを変えない）

```rust
impl EpubBook {
    /// 章の XHTML を先頭の BOM を除いた UTF-8 文字列で返す。
    ///
    /// 内容はそのまま返し、XML 解析・タグ除去・整形はしない。範囲外の
    /// `index` は [`CoreError::DocumentNotFound`]、読み出しと復号の失敗は
    /// [`CoreError::Archive`]。
    pub fn chapter_html(&self, index: usize) -> CoreResult<String>;

    /// 章から見た相対参照 `href` のリソースをバイト列で返す。
    ///
    /// 基準ディレクトリは章の `href` の所属ディレクトリ。`#` 以降の
    /// フラグメントは落とす。範囲外の `index` と存在しないパスは
    /// [`CoreError::DocumentNotFound`]、空のパスとスキーム付き URL・
    /// 先頭 `/` は [`CoreError::Archive`]、ルートを越える解決は
    /// [`CoreError::Validation`]。
    pub fn chapter_resource(&self, index: usize, href: &str) -> CoreResult<Vec<u8>>;
}
```

### 設計上の必須事項

1. `chapter_html`:
   - 手順は `chapter_bytes(index)` と同じ（範囲検証、`%XX` 復号フォールバック）。
   - 復号後、先頭に BOM (`\u{FEFF}`) があれば 1 個だけ除く。
   - それ以外は文字列をそのまま返す。
   - 不正な UTF-8 は `CoreError::Archive`。メッセージは既存の `chapter_text`
     の復号エラーに準ずる（対象の href を含める）。
   - 復号と BOM 除去は `chapter_text` と 1 か所に共有する（例: 非公開
     ヘルパーに切り出し、`chapter_text` がそれを呼ぶ形にする）。
     `chapter_text` の挙動とエラーを変えない。
2. `chapter_resource`:
   - 章の `href` から所属ディレクトリを出す（最後の `/` まで。`opf_dir` と
     同じ規則）。
   - `href` は最初の `#` 以降を落とし、前後の空白を除く（`split_fragment` の
     パス部と同じ規則）。
   - パス部が空 → `CoreError::Archive`。
   - スキーム付き URL・先頭 `/` → `CoreError::Archive`。
   - `.` と `..` を字句的に解決。ルートを越える → `CoreError::Validation`。
   - 解決したパスを `ArchiveReader::read_entry` で読む。失敗したら `%XX` を
     復号した名前で再試行する（`chapter_bytes` と同じフォールバック）。
     それも失敗したら最初のエラーを返す（存在しないパスは
     `CoreError::DocumentNotFound`）。
   - 返すのはリソース自体のバイト列のみ。
3. 相対参照の解決規則は 1 か所にまとめ、二重実装しない:
   - `fn resolve_reference(base_dir: &str, href: &str, subject: &str) -> CoreResult<String>`
     を切り出し、既存の `resolve_href` は
     `resolve_reference(opf_dir, href, "manifest の href")` を呼ぶ形にする。
   - メッセージは `"{subject} が空です"` のように組み立て、manifest の
     既存メッセージ（例: `manifest の href が空です`）を変えない。
   - `chapter_resource` は
     `resolve_reference(章の所属ディレクトリ, href, "章リソースの href")` を使う。
   - 既存の `resolve_href` のエラー種別（`Archive` / `Validation`）と
     メッセージを変えない。
4. 既存の挙動を変えない:
   - `metadata`・`chapters`・`toc`・`chapter_bytes`・`chapter_text` の
     シグネチャと挙動は従来どおり。既存の 61 テストがすべて通る。
   - 逸脱したパスの `Validation`、空パスの `Archive`、存在しない章の
     `DocumentNotFound` は従来どおり。
   - 新しい上限やリミットは追加しない。
5. ファイル先頭のモジュール doc（`//!`）を新 API に合わせて更新する:
   - 提供するものに「表示用の XHTML（`chapter_html`）とリソースの読み出し
     （`chapter_resource`）」を加える。
   - 範囲外の記述から画像の扱いを外し、「画像のデコード・表示、リソース URL
     の書き換え、サニタイズ」に直す（文言は簡潔な日本語で）。
6. 変更範囲は `core/src/epub.rs` に限る。`lib.rs` は変更不要。

### 実装時に効く実測済みの注意

- テストは `mod tests` の既存ヘルパー `build_epub`・`sample_epub` を使い、
  外部ファイルを使わない。`mimetype` は `build_epub` が先頭に付ける。
- `sample_epub()` の章は `OEBPS/ch1.xhtml`・`OEBPS/ch2.xhtml`。章の所属
  ディレクトリは `OEBPS`。
- `%XX` 復号は既存の `percent_decode`（`read_decoded_entry` が使用。`%` が
  無い場合と復号結果が不正な UTF-8 の場合は `None`）。
- 読み出しの核は `ArchiveReader::read_entry`。存在しないパスは
  `CoreError::DocumentNotFound` を返す。
- `split_fragment` は nav / NCX と共有済みの既存ヘルパー。
- `chapter(index)` は範囲外で `CoreError::DocumentNotFound` を返す既存
  ヘルパー。
- サブディレクトリの親参照テストは、OPF の章 `href` を `text/ch1.xhtml` に
  した EPUB（`build_epub` で container・OPF・`OEBPS/text/ch1.xhtml`・
  `OEBPS/images/pic.png` を書く）で確認できる。
- clippy は `-D warnings`。プロダクションコードで `unwrap()` / `expect()` /
  `panic!` / `todo!` を使わない。`unsafe` を使わない。
- `cargo fmt` の整形範囲は許容する。

### 必須テスト（`core/src/epub.rs` の `mod tests`、外部ファイルを使わない）

1. `chapter_html(0)` が `CH1` と完全に一致する（XML 宣言・タグ・属性を保持）。
2. `chapter_html` が先頭の BOM を 1 個だけ除く（BOM 付きの章を持つ EPUB）。
3. `chapter_html` は不正な UTF-8 の章で `CoreError::Archive`。
4. `chapter_html` は範囲外の `index` で `CoreError::DocumentNotFound`。
5. `chapter_resource` が同じディレクトリの `pic.png` を読める
   （`OEBPS/ch1.xhtml` から `pic.png` → `OEBPS/pic.png`）。
6. `chapter_resource` がサブディレクトリ (`images/pic.png`) と
   親ディレクトリ (`../images/pic.png`、`OEBPS/text/ch1.xhtml` から) を
   解決できる。
7. `chapter_resource` はフラグメント付き (`pic.png#frag`) でも読める。
8. `chapter_resource` は `a%20b.png` の `%XX` 復号フォールバックで
   エントリ `a b.png` を読める。
9. `chapter_resource` はルートを越える `../../evil.png` で
   `CoreError::Validation`。
10. `chapter_resource` はスキーム付き URL（`https://example.com/x.png`）と
    先頭 `/`（`/x.png`）で `CoreError::Archive`。
11. `chapter_resource` は空・空白のみ・`#frag` のみで `CoreError::Archive`。
12. `chapter_resource` は存在しないパスで `CoreError::DocumentNotFound`。
13. `chapter_resource` は範囲外の `index` で `CoreError::DocumentNotFound`。
14. 既存の 61 テストが引き続き通る（`cargo test` の件数が減らない）。

## Acceptance criteria

- 上記 API と設計上の必須事項がすべてコードとテストで確認できる。
- 既存の公開 API を壊さない（`cargo check --all-targets` が通る）。
- 次がすべて成功する。
  - `cargo fmt --all -- --check`
  - `cargo check --all-targets`
  - `cargo test --all-targets`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo check -p openrustbooks-core --target wasm32-unknown-unknown`
  - `cargo check -p openrustbooks-web --target wasm32-unknown-unknown`

## Constraints

- 読み取り専用。EPUB の作成・書き換えは作らない。
- UI への接続、サニタイズ、リソース URL の書き換え、表紙・章タイトルの
  取得、`landmarks` / `page-list` は範囲外。
- `desktop/` と `web/` を変更しない。`core` 内で完結させる。
- 無関係なリファクタリング、無関係なファイルの変更をしない。
- プロダクションコードで `unwrap()` / `expect()` / `panic!` / `todo!` を
  使わない。`unsafe` を使わない。
- 新しい依存を追加しない。
- ネットワークアクセスを必要とするテストを書かない。
- コメントと doc コメントは日本語で簡潔に書く。
- リモートリポジトリへ push しない。
- `goal_loop_hardened.rs`、`GOAL.md`、`.opencode/agents/verifier.md` を
  書き換えない。
