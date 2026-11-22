//! EPUB 読み取り API の Wasm バインディングと JS 公開用 DTO。
//!
//! 権威は [`crate::epub::EpubBook`] 側にあり、ここでは wasm-bindgen の境界向け
//! 変換だけを担う。呼び出し元の [`super`] が `wasm32` でのみこのモジュールを
//! コンパイルするため、各項目に個別の cfg は付与しない。

use crate::epub::{ChapterLink, EpubBook, EpubCoverImage, EpubTocEntry};
use serde::Serialize;
use wasm_bindgen::prelude::*;

/// EPUB 読み取り API を JavaScript に公開するラッパー。
#[wasm_bindgen]
pub struct WasmEpubBook {
    inner: EpubBook,
}

#[wasm_bindgen]
impl WasmEpubBook {
    /// EPUB ZIP のバイト列から本を開く。
    #[wasm_bindgen(constructor)]
    pub fn new(bytes: Vec<u8>) -> Result<WasmEpubBook, JsValue> {
        // JS から渡されたバイト列の所有権をそのまま使い、二重コピーを避ける。
        let inner = EpubBook::open_owned(bytes).map_err(super::js_error)?;
        Ok(Self { inner })
    }

    /// spine に含まれる章の数を返す。
    pub fn chapter_count(&self) -> usize {
        self.inner.chapters().len()
    }

    /// 指定章のアーカイブ内パスを返す。
    pub fn chapter_href(&self, index: usize) -> Result<String, JsValue> {
        self.inner
            .chapters()
            .get(index)
            .map(|chapter| chapter.href.clone())
            .ok_or_else(|| {
                super::js_error(crate::error::CoreError::DocumentNotFound(format!(
                    "章の index が範囲外です: {index}"
                )))
            })
    }

    /// 指定章の XHTML を返す。
    pub fn chapter_html(&self, index: usize) -> Result<String, JsValue> {
        self.inner.chapter_html(index).map_err(super::js_error)
    }

    /// 指定章のプレーンテキストを返す。
    pub fn chapter_text(&self, index: usize) -> Result<String, JsValue> {
        self.inner.chapter_text(index).map_err(super::js_error)
    }

    /// 指定章の表示名を返す。情報が無ければ `undefined`。
    pub fn chapter_title(&self, index: usize) -> Result<Option<String>, JsValue> {
        self.inner.chapter_title(index).map_err(super::js_error)
    }

    /// 指定章から見た相対リソースをバイト列で返す。
    pub fn chapter_resource(&self, index: usize, href: String) -> Result<Vec<u8>, JsValue> {
        self.inner
            .chapter_resource(index, &href)
            .map_err(super::js_error)
    }

    /// 章内の相対リンクを同じ本の章へ解決する。
    ///
    /// 解決できた場合は `{ chapterIndex, fragment }`、移動しない場合は
    /// `null` を返す。
    pub fn resolve_chapter_link(&self, index: usize, href: String) -> Result<JsValue, JsValue> {
        let link = self
            .inner
            .resolve_chapter_link(index, &href)
            .map_err(super::js_error)?;
        serialize_chapter_link(link)
    }

    /// EPUB3 nav の目次を JavaScript 値に変換して返す。
    pub fn toc(&self) -> Result<JsValue, JsValue> {
        let entries = self.inner.toc().map_err(super::js_error)?;
        serialize_navigation(entries)
    }

    /// EPUB3 nav の landmarks を JavaScript 値に変換して返す。
    pub fn landmarks(&self) -> Result<JsValue, JsValue> {
        let entries = self.inner.landmarks().map_err(super::js_error)?;
        serialize_navigation(entries)
    }

    /// EPUB3 nav の page-list を JavaScript 値に変換して返す。
    pub fn page_list(&self) -> Result<JsValue, JsValue> {
        let entries = self.inner.page_list().map_err(super::js_error)?;
        serialize_navigation(entries)
    }

    /// manifest が `remote-resources` を宣言しているか。
    pub fn has_remote_resources(&self) -> bool {
        self.inner.has_remote_resources()
    }

    /// EPUB の言語（例: `ja`, `en`）。不明なら `undefined`。
    pub fn language(&self) -> Option<String> {
        self.inner.metadata().language.clone()
    }

    /// 表紙画像の情報を JavaScript 値に変換して返す。無ければ空値。
    pub fn cover_image(&self) -> Result<JsValue, JsValue> {
        let cover = self.inner.cover_image().map_err(super::js_error)?;
        serialize_cover(cover)
    }
}

fn serialize_navigation(entries: Vec<EpubTocEntry>) -> Result<JsValue, JsValue> {
    let entries: Vec<WasmEpubTocEntry> = entries.into_iter().map(Into::into).collect();
    serde_wasm_bindgen::to_value(&entries)
        .map_err(|_| JsValue::from_str(super::SERIALIZATION_FAILED_MESSAGE))
}

fn serialize_cover(cover: Option<EpubCoverImage>) -> Result<JsValue, JsValue> {
    let cover: Option<WasmEpubCoverImage> = cover.map(Into::into);
    serde_wasm_bindgen::to_value(&cover)
        .map_err(|_| JsValue::from_str(super::SERIALIZATION_FAILED_MESSAGE))
}

fn serialize_chapter_link(link: Option<ChapterLink>) -> Result<JsValue, JsValue> {
    let Some(link) = link else {
        return Ok(JsValue::NULL);
    };
    serde_wasm_bindgen::to_value(&WasmChapterLink {
        chapter_index: link.chapter_index,
        fragment: link.fragment,
    })
    .map_err(|_| JsValue::from_str(super::SERIALIZATION_FAILED_MESSAGE))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct WasmChapterLink {
    chapter_index: usize,
    fragment: Option<String>,
}

#[derive(Serialize)]
struct WasmEpubTocEntry {
    title: String,
    href: String,
    fragment: Option<String>,
    depth: usize,
}

impl From<EpubTocEntry> for WasmEpubTocEntry {
    fn from(entry: EpubTocEntry) -> Self {
        Self {
            title: entry.title,
            href: entry.href,
            fragment: entry.fragment,
            depth: entry.depth,
        }
    }
}

#[derive(Serialize)]
struct WasmEpubCoverImage {
    id: String,
    href: String,
    media_type: String,
    bytes: Vec<u8>,
}

impl From<EpubCoverImage> for WasmEpubCoverImage {
    fn from(cover: EpubCoverImage) -> Self {
        Self {
            id: cover.id,
            href: cover.href,
            media_type: cover.media_type,
            bytes: cover.bytes,
        }
    }
}
