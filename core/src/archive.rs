//! ZIP アーカイブの読み取り専用ビュー。
//!
//! ファイル I/O に依存せず、メモリ上のバイト列だけを扱うためネイティブと
//! `wasm32-unknown-unknown` の両方でビルドできる。アーカイブの作成・書き込み・
//! 展開は提供しない。
//!
//! `docs/requirements.md` の REQ-002（アーカイブ内のコンテンツ閲覧）に対応し、
//! 品質要件の「パスの逸脱と過大な展開を扱う」を [`ArchiveLimits`] とパス正規化で
//! 満たす。

use crate::{CoreError, CoreResult};
use std::collections::HashMap;
use std::io::{Cursor, Read};
use std::sync::atomic::{AtomicU64, Ordering};

use zip::result::ZipError;
use zip::{CompressionMethod, ZipArchive};

/// EPUB 判定に使う `mimetype` エントリのパス。
const EPUB_MIMETYPE_PATH: &str = "mimetype";
/// `mimetype` の内容として期待する値。
const EPUB_MIMETYPE_VALUE: &str = "application/epub+zip";
/// EPUB 判定で読む `mimetype` の最大バイト数。
const EPUB_MIMETYPE_PROBE: u64 = 64;
/// 累計展開サイズの上限超過エラーの固定文言。
///
/// アーカイブ内のパスや `ZipError` の本文は含めない（GOAL.md Constraints）。
const EXPANDED_BUDGET_ERROR: &str = "アーカイブの累計展開サイズが上限を超えています";

/// アーカイブ閲覧のリミット。
///
/// いずれかを超えたら [`CoreError::Validation`] を返す。入力バイト列そのものの
/// 長さは呼び出し側で制限する（`max_entries` はセントラルディレクトリを解析した
/// 後に検証するため、解析分のメモリは入力長に比例する）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArchiveLimits {
    /// エントリ総数の上限。
    pub max_entries: usize,
    /// 1 エントリの非圧縮サイズ上限（バイト）。
    pub max_entry_size: u64,
    /// 全エントリの非圧縮サイズ合計の上限（バイト）。
    pub max_total_size: u64,
}

impl Default for ArchiveLimits {
    fn default() -> Self {
        Self {
            max_entries: 4096,
            max_entry_size: 64 * 1024 * 1024,
            max_total_size: 256 * 1024 * 1024,
        }
    }
}

/// アーカイブ内の 1 エントリ。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveEntry {
    /// 正規化済みパス（`/` 区切り。先頭の `./` や `/` を含まない）。
    pub path: String,
    /// ディレクトリエントリかどうか。
    pub is_dir: bool,
    /// 非圧縮サイズ（セントラルディレクトリの宣言値）。
    pub size: u64,
    /// 圧縮サイズ（セントラルディレクトリの宣言値）。
    pub compressed_size: u64,
}

/// 読み取り専用の ZIP アーカイブ閲覧。
///
/// バイト列とエントリ一覧を所有する。[`ArchiveReader::read_entry`] は `&self` の
/// ままで、読み出しのたびに [`ZipArchive`] を作り直す。累計展開バイトの記録に
/// [`AtomicU64`] の内部可変性を使うが、ロックを持たないため `Send + Sync` を満たす。
pub struct ArchiveReader {
    /// アーカイブ原本。読み出すたびにこのバイト列から `ZipArchive` を作る。
    bytes: Vec<u8>,
    /// セントラルディレクトリの順序を保ったエントリ一覧。
    entries: Vec<ArchiveEntry>,
    /// 正規化済みパス → `entries` の索引。
    index: HashMap<String, usize>,
    /// 適用したリミット。
    limits: ArchiveLimits,
    /// セッション中に実展開したバイト数の累計。
    ///
    /// 宣言サイズ合計だけでは `&self` の再読による累積を防げないため、読み出し
    /// ごとに実バイトを加算して `limits.max_total_size` で制限する。
    expanded: AtomicU64,
}

impl ArchiveReader {
    /// 既定のリミットでアーカイブを開く。
    pub fn new(bytes: &[u8]) -> CoreResult<Self> {
        Self::with_limits(bytes, ArchiveLimits::default())
    }

    /// リミットを指定してアーカイブを開く。
    ///
    /// バイト列を所有し、セントラルディレクトリを走査してエントリ一覧を確定する。
    /// パス逸脱とリミット超過は検出した時点で `CoreError::Validation` を返し、
    /// 不正なエントリを黙って読み飛ばさない。
    pub fn with_limits(bytes: &[u8], limits: ArchiveLimits) -> CoreResult<Self> {
        Self::from_owned(bytes.to_vec(), limits)
    }

    /// 所有権を持つバイト列からアーカイブを開く。
    ///
    /// 呼び出し側で既に `Vec<u8>` を所有している場合（wasm の境界を越えた
    /// バイト列など）の二重コピーを避けるため、`with_limits` の実処理を
    /// そのまま使う。バリデーションとエラーは `with_limits` と同一。
    pub fn from_owned(bytes: Vec<u8>, limits: ArchiveLimits) -> CoreResult<Self> {
        let (entries, index) = {
            // メタデータの走査だけなので展開用の reader は作らない。
            let archive = ZipArchive::new(Cursor::new(bytes.as_slice())).map_err(map_zip_error)?;

            let count = archive.len();
            if count > limits.max_entries {
                return Err(CoreError::validation(format!(
                    "アーカイブのエントリ数が上限を超えています: {count} > {}",
                    limits.max_entries
                )));
            }

            let mut entries = Vec::with_capacity(count);
            let mut index: HashMap<String, usize> = HashMap::with_capacity(count);
            let mut total_size: u64 = 0;

            for position in 0..count {
                let entry = archive.by_index_data(position).map_err(map_zip_error)?;
                let name = entry.name().map_err(map_zip_error)?;
                let path = normalize_path(&name)?;
                let size = entry.size();

                if size > limits.max_entry_size {
                    return Err(CoreError::validation(format!(
                        "エントリの非圧縮サイズが上限を超えています: {path} ({size} > {})",
                        limits.max_entry_size
                    )));
                }
                total_size = total_size.saturating_add(size);
                if total_size > limits.max_total_size {
                    return Err(CoreError::validation(format!(
                        "非圧縮サイズの合計が上限を超えています: {total_size} > {}",
                        limits.max_total_size
                    )));
                }

                // 正規化後に同名となるエントリは、先着を黙って残さずアーカイブ全体を拒否する。
                // `entries()` は重複を露出するため、consumer が後勝ちを選ぶと
                // parser differential が生じ得る（ADR-0004 の fail-closed 方針）。
                if index.insert(path.clone(), position).is_some() {
                    return Err(CoreError::validation(format!(
                        "正規化後に同名のエントリがあります: {path}"
                    )));
                }
                entries.push(ArchiveEntry {
                    path,
                    is_dir: entry.is_dir(),
                    size,
                    compressed_size: entry.compressed_size(),
                });
            }

            (entries, index)
        };

        Ok(Self {
            bytes,
            entries,
            index,
            limits,
            expanded: AtomicU64::new(0),
        })
    }

    /// エントリ一覧をセントラルディレクトリの順序で返す。
    pub fn entries(&self) -> &[ArchiveEntry] {
        &self.entries
    }

    /// 指定パスのエントリ内容を読み出す。
    ///
    /// ディレクトリと存在しないパスは [`CoreError::DocumentNotFound`]、壊れた
    /// アーカイブ・非対応の圧縮方式・暗号化・読み出し中の失敗は
    /// [`CoreError::Archive`]、パス逸脱と実読み出しの上限超過は
    /// [`CoreError::Validation`] を返す。
    pub fn read_entry(&self, path: &str) -> CoreResult<Vec<u8>> {
        self.read_entry_with_limit(path, self.limits.max_entry_size)
    }

    /// エントリを UTF-8 テキストとして読み出す。
    ///
    /// 先頭の BOM (`U+FEFF`) は除去する。不正な UTF-8 は lossy 変換せず
    /// [`CoreError::Archive`] を返し、改行コードの変換は行わない。
    pub fn read_entry_text(&self, path: &str) -> CoreResult<String> {
        let data = self.read_entry(path)?;
        let text = String::from_utf8(data).map_err(|err| {
            CoreError::archive(format!("UTF-8 として復号できません: {path} ({err})"))
        })?;
        Ok(text
            .strip_prefix('\u{FEFF}')
            .unwrap_or(text.as_str())
            .to_string())
    }

    /// 先頭エントリが `mimetype` で、その内容が `application/epub+zip` の
    /// アーカイブかどうか。
    ///
    /// `mimetype` は最大 64 バイトだけ読み、前後の空白・改行を除いて比較する。
    ///
    /// この probe も [`Self::read_entry_with_limit`] を通るため、実バイトが
    /// 累計展開予算に計上される（既定予算では無視できる大きさ）。
    pub fn is_epub(&self) -> bool {
        let Some(first) = self.entries.first() else {
            return false;
        };
        if first.is_dir || first.path != EPUB_MIMETYPE_PATH {
            return false;
        }

        let Ok(data) = self.read_entry_with_limit(EPUB_MIMETYPE_PATH, EPUB_MIMETYPE_PROBE) else {
            return false;
        };
        let Ok(text) = std::str::from_utf8(&data) else {
            return false;
        };
        // `read_entry_text` と同じく先頭の BOM を 1 個だけ除く。連続 BOM の
        // 2 個目は本文として残るため、比較に失敗して EPUB ではないと判定する。
        let text = text.strip_prefix('\u{FEFF}').unwrap_or(text);
        text.trim() == EPUB_MIMETYPE_VALUE
    }

    /// パスを正規化して索引を引き、`limit` バイトを上限に読み出す。
    ///
    /// 宣言サイズを信用せず、実読み出しでも `limit + 1` バイトで打ち切って
    /// 超過を検出する。読み出し前に宣言サイズ分の予算を予約し、成功時は実
    /// バイトに合わせて精算、失敗時は予約を全額返金する。予約が
    /// `limits.max_total_size` を超える場合は読み出さずに拒否する。
    fn read_entry_with_limit(&self, path: &str, limit: u64) -> CoreResult<Vec<u8>> {
        let normalized = normalize_path(path)?;
        let Some(&position) = self.index.get(&normalized) else {
            return Err(CoreError::DocumentNotFound(path.to_string()));
        };
        // 索引は `entries` 構築時に同じ走査から作るため、範囲外にはならない。
        let Some(entry) = self.entries.get(position) else {
            return Err(CoreError::DocumentNotFound(path.to_string()));
        };
        if entry.is_dir {
            return Err(CoreError::DocumentNotFound(path.to_string()));
        }

        // 宣言サイズ +1 を予約する。実読み出しは最大 `limit + 1` バイトなので、
        // それを予約の上限にもする（宣言サイズが過大でも上限を守る）。
        // `max_total_size` と `max_entry_size` は独立なので、予約は常に宣言
        // サイズ基準にする（`max_total_size < max_entry_size` でも誤拒否しない）。
        let reserve = entry.size.saturating_add(1).min(limit.saturating_add(1));
        self.reserve_budget(reserve)?;

        // 予約後の全ての失敗経路で返金するため、読み出し本体を別関数に分離し、
        // ここで結果を一箇所で精算する。
        match self.read_entry_body(position, entry, limit) {
            Ok(data) => {
                // 成功時は実バイトに合わせて予約を精算する（差額返金）。
                if let Err(err) = self.settle_budget(reserve, data.len() as u64) {
                    // 実バイトが予約を超えて上限に達した場合は予約を戻し、
                    // fail-closed で拒否する。
                    self.refund_budget(reserve);
                    return Err(err);
                }
                Ok(data)
            }
            Err(err) => {
                // 暗号化・非対応圧縮・読み出しエラー・実バイト超過など、
                // 予約後のどの失敗でも予約を全額返金する。
                self.refund_budget(reserve);
                Err(err)
            }
        }
    }

    /// 累計展開バイトに `amount` を予約する。
    ///
    /// CAS (`try_update`、旧 `fetch_update`) で現在値を読み書きするため、
    /// 並行する `&self` から呼ばれても合計が `max_total_size` を超えない。
    /// 加算がオーバーフローする場合も上限超過として扱う。
    fn reserve_budget(&self, amount: u64) -> CoreResult<()> {
        let max = self.limits.max_total_size;
        self.expanded
            .try_update(Ordering::SeqCst, Ordering::SeqCst, |current| {
                current.checked_add(amount).filter(|&next| next <= max)
            })
            .map(|_| ())
            .map_err(|_| CoreError::validation(EXPANDED_BUDGET_ERROR))
    }

    /// 予約 `reserve` を実バイト `actual` に置き換える。
    ///
    /// 実バイトが予約より少なければ差額を返金し、多ければ加算する。加算時に
    /// 上限を超える場合は変更せずエラーにする（宣言サイズが過小な不正入力に
    /// 対する防御）。
    fn settle_budget(&self, reserve: u64, actual: u64) -> CoreResult<()> {
        let max = self.limits.max_total_size;
        self.expanded
            .try_update(Ordering::SeqCst, Ordering::SeqCst, |current| {
                current
                    .checked_sub(reserve)
                    .and_then(|base| base.checked_add(actual))
                    .filter(|&next| next <= max)
            })
            .map(|_| ())
            .map_err(|_| CoreError::validation(EXPANDED_BUDGET_ERROR))
    }

    /// 予約を全額返金する（アンダーフローは 0 で頭打ち）。
    fn refund_budget(&self, amount: u64) {
        let _ = self
            .expanded
            .try_update(Ordering::SeqCst, Ordering::SeqCst, |current| {
                Some(current.saturating_sub(amount))
            });
    }

    /// エントリ本体を読み出す。予算の精算は `read_entry_with_limit` が行う。
    fn read_entry_body(
        &self,
        position: usize,
        entry: &ArchiveEntry,
        limit: u64,
    ) -> CoreResult<Vec<u8>> {
        let mut archive =
            ZipArchive::new(Cursor::new(self.bytes.as_slice())).map_err(map_zip_error)?;
        let file = archive.by_index(position).map_err(map_zip_error)?;

        if file.encrypted() {
            return Err(CoreError::archive(format!(
                "暗号化されたエントリは読み出せません: {}",
                entry.path
            )));
        }
        match file.compression() {
            CompressionMethod::Stored | CompressionMethod::Deflated => {}
            other => {
                return Err(CoreError::archive(format!(
                    "非対応の圧縮方式です: {} ({other:?})",
                    entry.path
                )));
            }
        }

        let mut reader = file.take(limit.saturating_add(1));
        let mut data = Vec::new();
        reader.read_to_end(&mut data).map_err(|err| {
            CoreError::archive(format!(
                "エントリの読み出しに失敗しました: {} ({err})",
                entry.path
            ))
        })?;

        if data.len() as u64 > limit {
            return Err(CoreError::validation(format!(
                "エントリの展開サイズが上限を超えています: {} ({} > {limit})",
                entry.path,
                data.len()
            )));
        }

        Ok(data)
    }
}

/// `zip` クレートのエラーを [`CoreError::Archive`] に変換する。
fn map_zip_error(err: ZipError) -> CoreError {
    CoreError::archive(err.to_string())
}

/// エントリ名を正規化する。
///
/// `\` を `/` に置換し、連続する `/` を 1 つにまとめ、`.` コンポーネントを除く。
/// 次のいずれかに該当する場合はパス逸脱として [`CoreError::Validation`] を返す。
///
/// - 先頭が `/`
/// - `..` コンポーネントを含む
/// - ドライブ文字（`C:` など）で始まる
/// - 正規化後に空になる、または `\0` を含む
fn normalize_path(raw: &str) -> CoreResult<String> {
    if raw.contains('\0') {
        return Err(CoreError::validation(format!(
            "エントリ名に NUL 文字が含まれています: {raw:?}"
        )));
    }

    // Windows の区切りを `/` に寄せてから判定する。
    let replaced = raw.replace('\\', "/");

    // 先頭の `./` を落としてから絶対パス判定する。`./C:/evil.txt` や
    // `.//abs.txt` のように `.` を挟んで絶対パスを隠す入力でも、正規化後の
    // 先頭コンポーネントで逸脱を検出するため。
    let mut candidate = replaced.as_str();
    while let Some(rest) = candidate.strip_prefix("./") {
        candidate = rest;
    }

    if candidate.starts_with('/') {
        return Err(CoreError::validation(format!(
            "絶対パスのエントリは許可されません: {raw:?}"
        )));
    }

    let mut chars = candidate.chars();
    if let (Some(first), Some(':')) = (chars.next(), chars.next())
        && first.is_ascii_alphabetic()
    {
        return Err(CoreError::validation(format!(
            "ドライブ文字で始まるエントリは許可されません: {raw:?}"
        )));
    }

    let mut components: Vec<&str> = Vec::new();
    for component in candidate.split('/') {
        match component {
            "" | "." => continue,
            ".." => {
                return Err(CoreError::validation(format!(
                    "親ディレクトリを参照するエントリは許可されません: {raw:?}"
                )));
            }
            other => components.push(other),
        }
    }

    if components.is_empty() {
        return Err(CoreError::validation(format!(
            "空のエントリ名は許可されません: {raw:?}"
        )));
    }

    Ok(components.join("/"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use zip::ZipWriter;
    use zip::write::SimpleFileOptions;

    /// テストで書き込むエントリ。
    enum Item<'a> {
        /// ファイル（名前・内容・圧縮方式）。
        File(&'a str, &'a [u8], CompressionMethod),
        /// ディレクトリ（末尾に `/` を付けた名前）。
        Dir(&'a str),
    }

    /// エントリを順に書き込んだ ZIP を組み立てる。
    fn build_zip(items: &[Item<'_>]) -> Vec<u8> {
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        for item in items {
            match item {
                Item::File(name, data, method) => {
                    let options = SimpleFileOptions::default().compression_method(*method);
                    writer.start_file(*name, options).unwrap();
                    writer.write_all(data).unwrap();
                }
                Item::Dir(name) => {
                    writer
                        .add_directory(*name, SimpleFileOptions::default())
                        .unwrap();
                }
            }
        }
        writer.finish().unwrap().into_inner()
    }

    /// stored と deflate を混ぜた基本のアーカイブ。
    fn sample_zip() -> Vec<u8> {
        build_zip(&[
            Item::Dir("docs/"),
            Item::File("docs/readme.txt", b"hello", CompressionMethod::Stored),
            Item::File(
                "docs/body.txt",
                b"world world world",
                CompressionMethod::Deflated,
            ),
        ])
    }

    /// `new` が `CoreError::Validation` を返すことを検証する。
    fn expect_validation(bytes: &[u8]) -> CoreError {
        match ArchiveReader::new(bytes) {
            Err(err @ CoreError::Validation(_)) => err,
            Err(other) => panic!("Validation を期待したが {other:?} だった"),
            Ok(_) => panic!("Validation を期待したが成功した"),
        }
    }

    /// `with_limits` が `CoreError::Validation` を返すことを検証する。
    fn expect_validation_with(bytes: &[u8], limits: ArchiveLimits) -> CoreError {
        match ArchiveReader::with_limits(bytes, limits) {
            Err(err @ CoreError::Validation(_)) => err,
            Err(other) => panic!("Validation を期待したが {other:?} だった"),
            Ok(_) => panic!("Validation を期待したが成功した"),
        }
    }

    /// `new` が `CoreError::Archive` を返すことを検証する。
    fn expect_archive(bytes: &[u8]) -> CoreError {
        match ArchiveReader::new(bytes) {
            Err(err @ CoreError::Archive(_)) => err,
            Err(other) => panic!("Archive を期待したが {other:?} だった"),
            Ok(_) => panic!("Archive を期待したが成功した"),
        }
    }

    /// `read_entry` のエラーを取り出す。
    fn entry_error(reader: &ArchiveReader, path: &str) -> CoreError {
        match reader.read_entry(path) {
            Err(err) => err,
            Ok(_) => panic!("エラーを期待したが読み出せた: {path}"),
        }
    }

    #[test]
    fn lists_entries_in_archive_order() {
        let bytes = sample_zip();
        let reader = ArchiveReader::new(&bytes).unwrap();
        let entries = reader.entries();

        assert_eq!(entries.len(), 3);

        // ディレクトリは末尾の `/` を落として正規化する。
        assert_eq!(entries[0].path, "docs");
        assert!(entries[0].is_dir);
        assert_eq!(entries[0].size, 0);

        assert_eq!(entries[1].path, "docs/readme.txt");
        assert!(!entries[1].is_dir);
        assert_eq!(entries[1].size, 5);
        // stored なので圧縮サイズは非圧縮サイズと同じ。
        assert_eq!(entries[1].compressed_size, 5);

        assert_eq!(entries[2].path, "docs/body.txt");
        assert!(!entries[2].is_dir);
        assert_eq!(entries[2].size, 17);

        // 圧縮率ではなく、ZIP の方式と圧縮サイズそのものを検証する。
        let mut zip = ZipArchive::new(Cursor::new(bytes.as_slice())).unwrap();
        let file = zip.by_index(2).unwrap();
        assert_eq!(file.compression(), CompressionMethod::Deflated);
        assert_eq!(entries[2].compressed_size, file.compressed_size());
    }

    #[test]
    fn normalizes_redundant_separators_and_dot_components() {
        let bytes = build_zip(&[
            Item::File("./a//b.txt", b"x", CompressionMethod::Stored),
            Item::File("c/./d.txt", b"y", CompressionMethod::Stored),
        ]);
        let reader = ArchiveReader::new(&bytes).unwrap();
        let entries = reader.entries();

        assert_eq!(entries[0].path, "a/b.txt");
        assert_eq!(entries[1].path, "c/d.txt");
        assert_eq!(reader.read_entry("./c//d.txt").unwrap(), b"y");
    }

    #[test]
    fn reads_entry_contents_for_both_compression_methods() {
        let bytes = sample_zip();
        let reader = ArchiveReader::new(&bytes).unwrap();

        assert_eq!(reader.read_entry("docs/readme.txt").unwrap(), b"hello");
        assert_eq!(
            reader.read_entry("docs/body.txt").unwrap(),
            b"world world world"
        );
    }

    #[test]
    fn read_entry_text_strips_utf8_bom() {
        let bytes = build_zip(&[
            Item::File(
                "bom.md",
                b"\xEF\xBB\xBF# \xE3\x81\x82",
                CompressionMethod::Deflated,
            ),
            Item::File("crlf.txt", b"a\r\nb", CompressionMethod::Stored),
        ]);
        let reader = ArchiveReader::new(&bytes).unwrap();

        assert_eq!(reader.read_entry_text("bom.md").unwrap(), "# あ");
        // 改行コードは変換しない。
        assert_eq!(reader.read_entry_text("./crlf.txt").unwrap(), "a\r\nb");
    }

    #[test]
    fn classifies_read_errors() {
        let bytes = build_zip(&[
            Item::Dir("dir/"),
            Item::File("bad.txt", &[0xFF, 0xFE, 0xFD], CompressionMethod::Stored),
        ]);
        let reader = ArchiveReader::new(&bytes).unwrap();

        // ディレクトリ。
        assert!(matches!(
            entry_error(&reader, "dir"),
            CoreError::DocumentNotFound(_)
        ));
        // 存在しないパス。
        assert!(matches!(
            entry_error(&reader, "missing.txt"),
            CoreError::DocumentNotFound(_)
        ));
        // 不正な UTF-8（lossy 変換しない）。
        assert!(matches!(
            reader.read_entry_text("bad.txt"),
            Err(CoreError::Archive(_))
        ));
        // パス逸脱は Validation。
        assert!(matches!(
            entry_error(&reader, "../bad.txt"),
            CoreError::Validation(_)
        ));
    }

    #[test]
    fn rejects_path_traversal_entries() {
        for name in [
            "../evil.txt",
            "/abs.txt",
            "a/../../b.txt",
            "..\\evil.txt",
            "C:\\evil.txt",
            // `.` を挟んで絶対パスやドライブ文字を隠す形も正規化後の先頭で判定する。
            "./C:/evil.txt",
            ".\\C:\\evil.txt",
            "./C:",
            ".//abs.txt",
            "././C:/evil.txt",
            "bad\0name.txt",
            "",
        ] {
            let bytes = build_zip(&[Item::File(name, b"x", CompressionMethod::Stored)]);
            expect_validation(&bytes);
        }

        // 照合パスにも同じ正規化を適用し、逸脱は `Validation` にする。
        let reader = ArchiveReader::new(&sample_zip()).unwrap();
        for path in [
            "../evil.txt",
            "/abs.txt",
            "a/../../b.txt",
            "..\\evil.txt",
            "C:\\evil.txt",
            "./C:/evil.txt",
            "",
        ] {
            assert!(matches!(
                entry_error(&reader, path),
                CoreError::Validation(_)
            ));
        }
    }

    #[test]
    fn rejects_archive_that_contains_any_escape_entry() {
        // 逸脱エントリを 1 つでも含むアーカイブは、読み飛ばさずに拒否する。
        let bytes = build_zip(&[
            Item::File("../evil.txt", b"a", CompressionMethod::Stored),
            Item::File("/abs.txt", b"b", CompressionMethod::Stored),
            Item::File("a/../../b.txt", b"c", CompressionMethod::Stored),
            Item::File("..\\evil.txt", b"d", CompressionMethod::Deflated),
            Item::File("C:\\evil.txt", b"e", CompressionMethod::Stored),
        ]);
        expect_validation(&bytes);
    }

    /// 正規化後に同名になるエントリは、入力名の表記に関わらずアーカイブ全体を拒否する。
    ///
    /// `zip` 9.1.3 は同じ raw 名を Writer も Reader も 1 つへまとめるため、
    /// 「`a.txt` × 2」という完全同一 raw 名の ZIP はこのクレートでは作れない。
    /// 代替として末尾 `/` 付きの `a.txt/` と `a.txt` を使う（正規化後は両方
    /// `a.txt` で、Reader から見える名前は完全に同じ）。
    #[test]
    fn rejects_duplicate_entries_after_normalization() {
        for (first, second, normalized) in [
            ("a.txt/", "a.txt", "a.txt"),  // 正規化後完全相同（末尾 `/` を除く）
            ("./a.txt", "a.txt", "a.txt"), // 先頭の `./`
            ("a//b.txt", "a/b.txt", "a/b.txt"), // 二重スラッシュ
            ("a\\b.txt", "a/b.txt", "a/b.txt"), // バックスラッシュ
            ("a/./b.txt", "a/b.txt", "a/b.txt"), // 途中の `.`
        ] {
            let bytes = build_zip(&[
                Item::File(first, b"x", CompressionMethod::Stored),
                Item::File(second, b"y", CompressionMethod::Stored),
            ]);
            match expect_validation(&bytes) {
                CoreError::Validation(message) => assert!(
                    message.contains(normalized),
                    "エラー文言に正規化後のパスがありません: {message}"
                ),
                other => panic!("Validation を期待したが {other:?} だった"),
            }
        }
    }

    /// 正規化後に別名のエントリは従来どおり開ける。
    #[test]
    fn accepts_entries_with_distinct_normalized_names() {
        let bytes = build_zip(&[
            Item::File("a.txt", b"x", CompressionMethod::Stored),
            Item::File("b.txt", b"y", CompressionMethod::Stored),
        ]);
        let reader = ArchiveReader::new(&bytes).unwrap();

        assert_eq!(reader.entries().len(), 2);
        assert_eq!(reader.entries()[0].path, "a.txt");
        assert_eq!(reader.entries()[1].path, "b.txt");
        assert_eq!(reader.read_entry("a.txt").unwrap(), b"x");
        assert_eq!(reader.read_entry("b.txt").unwrap(), b"y");
    }

    /// 既存の `sample_zip` は、Dir("docs/") と直下の File が正規化後も同名に
    /// ならず、重複拒否の導入後も開ける（回帰）。
    #[test]
    fn sample_zip_directory_and_files_have_distinct_normalized_paths() {
        let bytes = sample_zip();
        let reader = ArchiveReader::new(&bytes).expect("sample_zip は開けるままであること");
        let paths: Vec<&str> = reader
            .entries()
            .iter()
            .map(|entry| entry.path.as_str())
            .collect();

        assert_eq!(paths, vec!["docs", "docs/readme.txt", "docs/body.txt"]);
        let unique: std::collections::HashSet<&str> = paths.iter().copied().collect();
        assert_eq!(unique.len(), paths.len());
    }

    #[test]
    fn enforces_limits() {
        let zip = build_zip(&[
            Item::File("a.txt", b"aaaa", CompressionMethod::Stored),
            Item::File("b.txt", b"bbbb", CompressionMethod::Deflated),
        ]);

        expect_validation_with(
            &zip,
            ArchiveLimits {
                max_entries: 1,
                ..ArchiveLimits::default()
            },
        );
        expect_validation_with(
            &zip,
            ArchiveLimits {
                max_entry_size: 3,
                ..ArchiveLimits::default()
            },
        );
        expect_validation_with(
            &zip,
            ArchiveLimits {
                max_total_size: 7,
                ..ArchiveLimits::default()
            },
        );

        // 上限ちょうどは通る。
        let exact = ArchiveLimits {
            max_entries: 2,
            max_entry_size: 4,
            max_total_size: 8,
        };
        assert!(ArchiveReader::with_limits(&zip, exact).is_ok());

        // 実読み出しでも上限を超えたら打ち切る（宣言サイズを信用しない場合の防御）。
        let reader = ArchiveReader::new(&zip).unwrap();
        match reader.read_entry_with_limit("a.txt", 2) {
            Err(CoreError::Validation(_)) => {}
            Err(other) => panic!("Validation を期待したが {other:?} だった"),
            Ok(data) => panic!("Validation を期待したが {} バイト読めた", data.len()),
        }
    }

    #[test]
    fn rejects_corrupted_bytes_without_panicking() {
        // ZIP ではないデータ。
        expect_archive(b"this is definitely not a zip archive");
        // 空データ。
        expect_archive(&[]);
        // 途中で切れたデータ。
        let truncated = sample_zip();
        expect_archive(&truncated[..truncated.len() / 2]);
    }

    #[test]
    fn detects_epub_archives() {
        let epub = build_zip(&[
            Item::File(
                "mimetype",
                b"application/epub+zip\n",
                CompressionMethod::Stored,
            ),
            Item::File(
                "META-INF/container.xml",
                b"<container/>",
                CompressionMethod::Stored,
            ),
        ]);
        assert!(ArchiveReader::new(&epub).unwrap().is_epub());

        // 先頭が `mimetype` でない ZIP。
        assert!(!ArchiveReader::new(&sample_zip()).unwrap().is_epub());

        // `mimetype` の内容が違う ZIP。
        let other = build_zip(&[Item::File(
            "mimetype",
            b"text/plain",
            CompressionMethod::Stored,
        )]);
        assert!(!ArchiveReader::new(&other).unwrap().is_epub());
    }

    /// `mimetype` の先頭 BOM は 1 個だけ除いて判定する（read_entry_text と同じ規則）。
    #[test]
    fn detects_epub_with_single_leading_bom() {
        let mimetype = b"\xEF\xBB\xBFapplication/epub+zip";
        let bytes = build_zip(&[Item::File("mimetype", mimetype, CompressionMethod::Stored)]);
        assert!(ArchiveReader::new(&bytes).unwrap().is_epub());
    }

    /// 連続 BOM の 2 個目は本文として残るため EPUB として扱わない。
    #[test]
    fn rejects_epub_with_consecutive_boms() {
        let mimetype = b"\xEF\xBB\xBF\xEF\xBB\xBFapplication/epub+zip";
        let bytes = build_zip(&[Item::File("mimetype", mimetype, CompressionMethod::Stored)]);
        assert!(!ArchiveReader::new(&bytes).unwrap().is_epub());
    }

    #[test]
    fn opens_empty_archive() {
        let bytes = build_zip(&[]);
        let reader = ArchiveReader::new(&bytes).unwrap();
        assert!(reader.entries().is_empty());
        assert!(!reader.is_epub());
    }

    /// 同じエントリを再読すると、累計予算を超えた時点で拒否される。
    ///
    /// 同じ path の再読それ自体は許可し、予算が残る限り再課金して成功する
    /// （EPUB の章を複数回読む通常操作を壊さないため）。
    #[test]
    fn rejects_rereading_only_after_budget_is_exhausted() {
        let bytes = build_zip(&[Item::File("a.txt", b"hello", CompressionMethod::Stored)]);
        // 宣言合計 5 は開ける。予約は 6 (declared + 1) なので、予算 8 なら
        // 1 回目は通り、累計 5 + 予約 6 = 11 > 8 の 2 回目は拒否される。
        let limits = ArchiveLimits {
            max_total_size: 8,
            ..ArchiveLimits::default()
        };
        let reader = ArchiveReader::with_limits(&bytes, limits).unwrap();

        assert_eq!(reader.read_entry("a.txt").unwrap(), b"hello");
        match reader.read_entry("a.txt") {
            Err(CoreError::Validation(message)) => {
                assert!(
                    !message.contains("a.txt"),
                    "予算エラーにパスを含めないこと: {message}"
                );
            }
            Err(other) => panic!("Validation を期待したが {other:?} だった"),
            Ok(data) => panic!("2 回目が読めてしまった: {} バイト", data.len()),
        }
    }

    /// 読み出し失敗時に予約が返金され、後続の別エントリが成功する。
    #[test]
    fn refunds_budget_on_read_failure() {
        // Stored エントリのデータは ZIP 内にそのまま現れる。1 バイト書き換えて
        // CRC 不一致を起こし、`read_to_end` を失敗させる。
        let mut bytes = build_zip(&[
            Item::File("bad.txt", b"hello world", CompressionMethod::Stored),
            Item::File("good.txt", b"ok", CompressionMethod::Stored),
        ]);
        let needle: &[u8] = b"hello world";
        let position = bytes
            .windows(needle.len())
            .position(|window| window == needle)
            .expect("Stored データが ZIP 内にそのまま存在する");
        bytes[position] ^= 0xFF;

        // 宣言合計 13。bad の予約 12 と good の予約 3 が通る予算にする。
        // 返金が無ければ bad の予約 12 が残り good の予約 3 が上限を超える。
        let limits = ArchiveLimits {
            max_total_size: 13,
            ..ArchiveLimits::default()
        };
        let reader = ArchiveReader::with_limits(&bytes, limits).unwrap();

        assert!(matches!(
            entry_error(&reader, "bad.txt"),
            CoreError::Archive(_)
        ));
        assert_eq!(reader.read_entry("good.txt").unwrap(), b"ok");
    }

    /// 予算内にある複数のエントリを連続して読める。
    #[test]
    fn reads_multiple_entries_within_session_budget() {
        let bytes = build_zip(&[
            Item::File("a.txt", b"aaaa", CompressionMethod::Stored),
            Item::File("b.txt", b"bbbb", CompressionMethod::Deflated),
        ]);
        // 宣言合計 8。各予約は 5 なので、2 回目の予約 (4 + 5 = 9) を許容する
        // 予算 10 なら両方成功する。
        let limits = ArchiveLimits {
            max_total_size: 10,
            ..ArchiveLimits::default()
        };
        let reader = ArchiveReader::with_limits(&bytes, limits).unwrap();

        assert_eq!(reader.read_entry("a.txt").unwrap(), b"aaaa");
        assert_eq!(reader.read_entry("b.txt").unwrap(), b"bbbb");
    }

    /// EPUB の通常操作で同じ章を連続して読んでも予算内なら成功する（回帰）。
    #[test]
    fn repeated_epub_reads_within_default_budget_still_work() {
        use crate::epub::EpubBook;

        const CONTAINER: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>"#;
        const OPF: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="pub-id">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:title>サンプル本</dc:title>
    <dc:language>ja</dc:language>
    <dc:identifier id="pub-id">urn:isbn:1</dc:identifier>
  </metadata>
  <manifest>
    <item id="ch1" href="ch1.xhtml" media-type="application/xhtml+xml"/>
    <item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/>
  </manifest>
  <spine><itemref idref="ch1"/></spine>
</package>"#;
        const CH1: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<html xmlns="http://www.w3.org/1999/xhtml"><body><p>hello</p></body></html>"#;
        const NAV: &str = r#"<html xmlns:epub="http://www.idpf.org/2007/ops"><body><nav epub:type="toc"><ol><li><a href="ch1.xhtml">Chapter</a></li></ol></nav></body></html>"#;

        let bytes = build_zip(&[
            Item::File(
                "mimetype",
                b"application/epub+zip",
                CompressionMethod::Stored,
            ),
            Item::File(
                "META-INF/container.xml",
                CONTAINER.as_bytes(),
                CompressionMethod::Deflated,
            ),
            Item::File(
                "OEBPS/content.opf",
                OPF.as_bytes(),
                CompressionMethod::Deflated,
            ),
            Item::File(
                "OEBPS/ch1.xhtml",
                CH1.as_bytes(),
                CompressionMethod::Deflated,
            ),
            Item::File(
                "OEBPS/nav.xhtml",
                NAV.as_bytes(),
                CompressionMethod::Deflated,
            ),
        ]);
        let book = EpubBook::open(&bytes).unwrap();

        // 同じ章を続けて読んでも、既定予算内では再課金されて成功する。
        assert!(book.chapter_text(0).unwrap().contains("hello"));
        assert!(book.chapter_html(0).unwrap().contains("hello"));
        assert_eq!(book.toc().unwrap().len(), 1);
        // さらに再読しても成功する（同じ path の再読を常に拒否しない）。
        assert!(book.chapter_text(0).unwrap().contains("hello"));
    }

    /// 並行読み出しでも累計予算を超えない（CAS 予約の検証）。
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn concurrent_reads_respect_session_budget() {
        use std::sync::Arc;
        use std::thread;

        let bytes = build_zip(&[Item::File("a.txt", b"aaaa", CompressionMethod::Stored)]);
        // 予約は 5。予算 8 なら同時に読めるのは 1 スレッドだけ。
        let reader = Arc::new(
            ArchiveReader::with_limits(
                &bytes,
                ArchiveLimits {
                    max_total_size: 8,
                    ..ArchiveLimits::default()
                },
            )
            .unwrap(),
        );

        let handles: Vec<_> = (0..8)
            .map(|_| {
                let reader = Arc::clone(&reader);
                thread::spawn(move || reader.read_entry("a.txt").is_ok())
            })
            .collect();
        let successes = handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .filter(|ok| *ok)
            .count();

        assert_eq!(successes, 1, "予算を超える並行展開が成功してしまった");
    }

    #[test]
    fn archive_reader_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<ArchiveReader>();
    }
}
