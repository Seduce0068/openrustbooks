//! ZIP アーカイブの読み取り専用 Wasm API（TD-M22 段階 1 / ADR-0223）。
//!
//! パス検証・サイズ制限・不正エントリ拒否の権威は
//! [`crate::archive::ArchiveReader`] が持ち、ここでは JS 境界向けの変換と
//! fail-closed な固定エラー文言だけを提供する。呼び出し元の [`super`] が
//! `wasm32` でのみこのモジュールをコンパイルするため、各項目に個別の cfg は
//! 付与しない。

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

/// ZIP アーカイブの読み取り専用 Wasm API（TD-M22 段階 1 / ADR-0223）。
///
/// パス検証・サイズ制限・不正エントリ拒否の権威は
/// [`crate::archive::ArchiveReader`] が持ち、ここでは JS 境界向けの変換と
/// fail-closed な固定エラー文言だけを提供する。
#[wasm_bindgen]
pub struct WasmArchive {
    inner: crate::archive::ArchiveReader,
}

// JS 境界に出すエラーは固定文言のみ。CoreError の `to_string()` はパスや
// ZipError 本文を含むため絶対に流用しない（親の `js_error()` も使わない）。
// 4 定数は `wasm/tests.rs` から `super::ERR_*` で参照するため親へ見せる。
pub(super) const ERR_INVALID_ARCHIVE: &str = "Invalid ZIP archive";
pub(super) const ERR_ENTRY_READ: &str = "ZIP entry read failed";
pub(super) const ERR_INVALID_UTF8: &str = "ZIP entry is not valid UTF-8";
pub(super) const ERR_ENTRIES: &str = "ZIP entries unavailable";

#[wasm_bindgen]
impl WasmArchive {
    /// ZIP バイト列から読み取り専用ビューを開く。
    ///
    /// 所有権をそのまま `ArchiveReader::from_owned` へ渡し二重コピーを避ける。
    /// いかなる失敗も固定文言へ変換し、内部の理由は露出しない。
    #[wasm_bindgen(constructor)]
    pub fn new(bytes: Vec<u8>) -> Result<WasmArchive, JsValue> {
        let inner = crate::archive::ArchiveReader::from_owned(
            bytes,
            crate::archive::ArchiveLimits::default(),
        )
        .map_err(|_| JsValue::from_str(ERR_INVALID_ARCHIVE))?;
        Ok(Self { inner })
    }

    /// エントリ一覧を camelCase の JS 配列で返す。
    pub fn entries(&self) -> Result<JsValue, JsValue> {
        let entries: Vec<WasmArchiveEntry> = self
            .inner
            .entries()
            .iter()
            .map(|entry| WasmArchiveEntry {
                path: entry.path.clone(),
                is_dir: entry.is_dir,
                size: entry.size,
                compressed_size: entry.compressed_size,
            })
            .collect();
        serde_wasm_bindgen::to_value(&entries).map_err(|_| JsValue::from_str(ERR_ENTRIES))
    }

    /// 指定パスのエントリをバイト列で返す。
    ///
    /// パス逸脱・未検出・壊れたデータなど、全失敗を固定文言にする。
    pub fn read_entry(&self, path: &str) -> Result<Vec<u8>, JsValue> {
        self.inner
            .read_entry(path)
            .map_err(|_| JsValue::from_str(ERR_ENTRY_READ))
    }

    /// 指定パスのエントリを UTF-8 テキストで返す（先頭 BOM は除去）。
    ///
    /// 読み出し失敗と UTF-8 不正を別の固定文言にするため、バイト列取得は
    /// `ArchiveReader::read_entry` に委譲し、デコードのみここで行う。
    pub fn read_entry_text(&self, path: &str) -> Result<String, JsValue> {
        let bytes = self.read_entry(path)?;
        let text = String::from_utf8(bytes).map_err(|_| JsValue::from_str(ERR_INVALID_UTF8))?;
        Ok(text.strip_prefix('\u{FEFF}').unwrap_or(&text).to_string())
    }

    /// `mimetype` が先頭で内容が `application/epub+zip` なら true。
    pub fn is_epub(&self) -> bool {
        self.inner.is_epub()
    }
}

/// `entries()` の JS 公開用 DTO（camelCase: `path` / `isDir` / `size` / `compressedSize`）。
///
/// フィールドは `wasm/tests.rs` が検証するため `pub(super)` で親へ見せる。
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WasmArchiveEntry {
    pub(super) path: String,
    pub(super) is_dir: bool,
    pub(super) size: u64,
    pub(super) compressed_size: u64,
}
