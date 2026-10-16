# Goal

`openrustbooks-core` に ZIP アーカイブ閲覧の中核を実装する。
読み取り専用で、ファイル I/O に依存せずメモリ上のバイト列だけを扱い、
ネイティブと wasm32 の両方でビルドできること。

## Scope

### 背景

- `docs/requirements.md` の REQ-002 は ZIP「アーカイブ内のコンテンツ閲覧」を求める。
- 同書の品質要件は「アーカイブ処理でパスの逸脱と過大な展開を扱う」を求める。
- `docs/decisions/0003-native-wasm-target-split.md` により、共通コアは
  wasm32 でもビルドできる必要がある（OS 依存 API をコアで使わない）。
- 現状 `DocumentFormat::Zip` はあるが、アーカイブを読むコードは存在しない。

### 成果物

- `core/src/archive.rs`（新規）
- `core/src/lib.rs`: `pub mod archive;` と主要型の再エクスポート
  （`pub use archive::{ArchiveEntry, ArchiveLimits, ArchiveReader};`）
- `core/src/error.rs`: `Archive` 変種と `CoreError::archive(msg)` の追加
- `core/Cargo.toml`: 次の依存を追加する

  ```toml
  zip = { version = "9.0", default-features = false, features = ["deflate-flate2-zlib-rs"] }
  ```

  （pure Rust のみ。`deflate-flate2` 単独では flate2 のバックエンドが無くビルドできない。）

### 公開 API（名前とシグネチャを変えない）

```rust
pub struct ArchiveLimits {
    pub max_entries: usize,
    pub max_entry_size: u64,
    pub max_total_size: u64,
}

impl Default for ArchiveLimits {
    // max_entries = 4096, max_entry_size = 64 MiB, max_total_size = 256 MiB
}

pub struct ArchiveEntry {
    pub path: String,           // 正規化済み（'/' 区切り、先頭 './' や '/' なし）
    pub is_dir: bool,
    pub size: u64,              // 非圧縮サイズ（セントラルディレクトリの宣言値）
    pub compressed_size: u64,
}

pub struct ArchiveReader { /* bytes: Vec<u8> と entries: Vec<ArchiveEntry> を所有 */ }

impl ArchiveReader {
    pub fn new(bytes: &[u8]) -> CoreResult<Self>;
    pub fn with_limits(bytes: &[u8], limits: ArchiveLimits) -> CoreResult<Self>;
    pub fn entries(&self) -> &[ArchiveEntry];
    pub fn read_entry(&self, path: &str) -> CoreResult<Vec<u8>>;
    pub fn read_entry_text(&self, path: &str) -> CoreResult<String>;
    pub fn is_epub(&self) -> bool;
}
```

`ArchiveReader` は `Send + Sync` を満たすこと（`Rc`・`RefCell`・`Mutex` を使わない）。

### 設計上の必須事項

1. `new` / `with_limits` はバイト列を `Vec<u8>` として所有し、
   セントラルディレクトリを走査してエントリ一覧を確定する。
2. `read_entry(&self, ..)` は `&self` のまま読む。
   読み出しのたびに `zip::ZipArchive::new(std::io::Cursor::new(&self.bytes))` を
   作って `by_index` する（`by_index` は `&mut self` を取るため、内部可変性を使わない）。
   事前に `entries` の位置から索引できるよう、パス→索引の対応を持ってよい。
3. パス正規化: `\` を `/` に置換し、連続する `/` を 1 つにまとめ、
   `.` コンポーネントを除く。正規化後に次はパス逸脱として拒否する。
   - 先頭が `/`
   - `..` コンポーネントを含む
   - ドライブ文字（`C:` など）で始まる
   - 空文字、`\0` を含む
   逸脱を 1 つでも検出したら `new` が `CoreError::Validation` を返す
   （不正なエントリを黙って読み飛ばさない）。
4. リミット（いずれも超えたら `CoreError::Validation`）:
   - エントリ総数 > `max_entries`
   - いずれかの宣言非圧縮サイズ > `max_entry_size`
   - 宣言非圧縮サイズの合計 > `max_total_size`
   - `read_entry` の実読み出しでも、宣言サイズを信用せず
     `Read::take(max_entry_size + 1)` 相当で打ち切り、超えたらエラーにする。
5. 対応する圧縮方式は `Stored` と `Deflated` のみ。
   それ以外の方式のエントリは panic せず `CoreError::Archive` を返す。
   暗号化エントリ（`file.encrypted()` が true）も `CoreError::Archive` を返す。
6. `read_entry` のエラー分類:
   - ディレクトリ、存在しないパス → `CoreError::DocumentNotFound`
   - 壊れたアーカイブ、非対応方式、暗号化、読み出し中の失敗 → `CoreError::Archive`
   - リミット超過、パス逸脱 → `CoreError::Validation`
7. `read_entry_text` は UTF-8 として復号する。先頭の BOM（`\u{FEFF}`）は除去する。
   不正な UTF-8 は `CoreError::Archive` を返す（lossy 変換しない）。
   改行コードの変換は行わない。エントリ名の照合には `read_entry` と同じ正規化を適用する。
8. `is_epub` は、先頭エントリの名前が `mimetype` で、その内容（最大 64 バイトを読む）が
   前後の空白・改行を除いて `application/epub+zip` のときだけ true。
9. `DocumentFormat::from_extension` は変更しない。`Document` 型との統合は本作業の範囲外。

### `zip` クレート 9.x の実測済み API（そのまま使ってよい）

```rust
let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes))?; // ZipResult<ZipArchive<Cursor<&[u8]>>>
let count = archive.len();
let mut file = archive.by_index(i)?;            // ZipFile（Read を実装）
let name: std::borrow::Cow<'_, str> = file.name()?; // 9.x では Result を返す
let enclosed: Option<std::path::PathBuf> = file.enclosed_name();
let (size, compressed, is_dir, encrypted) =
    (file.size(), file.compressed_size(), file.is_dir(), file.encrypted());
```

テスト用の ZIP 生成（`zip` は通常の依存なのでテストから使える）:

```rust
let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
let options = zip::write::SimpleFileOptions::default()
    .compression_method(zip::CompressionMethod::Deflated); // または Stored
writer.start_file("name.txt", options)?;   // ディレクトリは add_directory(name, options)
writer.write_all(b"...")?;
let bytes: Vec<u8> = writer.finish()?.into_inner();
```

### 必須テスト（`core/src/archive.rs` の `mod tests`、外部ファイルを使わない）

1. stored と deflate の両方のエントリを持つ ZIP で、`entries()` が正規化済みパス・
   `size`・`is_dir` をアーカイブ内の順序で返す。
2. `read_entry` の内容が書き込んだ内容と一致する（stored / deflate の両方）。
3. `read_entry_text` が BOM 付き UTF-8 から BOM を除いた文字列を返す。
4. ディレクトリ、存在しないパス、不正な UTF-8 のエントリでそれぞれ
   `DocumentNotFound` / `DocumentNotFound` / `Archive` を返す。
5. `../evil.txt`、`/abs.txt`、`a/../../b.txt`、`..\evil.txt`、`C:\evil.txt` を
   含む ZIP で `new` が `CoreError::Validation` を返す。
6. `with_limits` で `max_entries`・`max_entry_size`・`max_total_size` を
   それぞれ小さく設定し、`CoreError::Validation` を返す。
7. 壊れたバイト列（ZIP でないデータ、途中で切れたデータ）で `new` が
   `CoreError::Archive` を返し、panic しない。
8. `mimetype` が `application/epub+zip` の ZIP で `is_epub()` が true、
   通常の ZIP で false。
9. エントリ 0 個の空 ZIP で `new` が成功し `entries()` が空。
10. `ArchiveReader` が `Send + Sync` であることをコンパイル時に検証する
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

- ZIP の作成・書き込み・展開機能は作らない（読み取り専用）。
- `desktop/` と `web/` を変更しない。`core` 内で完結させる。
- 無関係なリファクタリング、無関係なファイルの変更をしない。
- プロダクションコードで `unwrap()` / `expect()` / `panic!` / `todo!` を使わない。
  `unsafe` を使わない。
- 新しい依存を追加しない（上記の `zip` 以外）。
- ネットワークアクセスを必要とするテストを書かない。
- コメントと doc コメントは日本語で簡潔に書く。
- リモートリポジトリへ push しない。
- `goal_loop.rs`、`GOAL.md`、`.opencode/agents/verifier.md` を書き換えない。
