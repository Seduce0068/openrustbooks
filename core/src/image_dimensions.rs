//! 画像の自然寸法を取得する pure 関数（native / wasm 共通、ADR-0136 / 0137）。
//!
//! 画像データはデコードせずヘッダーだけを読む。取得できない値・不正な値は
//! fail-closed で `None` とし、呼び出し側が属性を付けないようにする。

/// 属性付与を許す寸法の上限（px）。これより大きい値は疑わしいとして拒否する。
pub const MAX_DIMENSION: u32 = 16_384;

/// 画像バイト列から自然寸法 `(width, height)` を返す。
///
/// PNG / JPEG / GIF / WebP を支持する。SVG は `imagesize` の対象外のため
/// `None` を返す（web 側でルート要素の属性を読む）。
///
/// 0 値・上限超え・`u32` に収まらない値・ヘッダー解析失敗はすべて `None`。
pub fn image_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    let size = imagesize::blob_size(bytes).ok()?;
    let width = u32::try_from(size.width).ok()?;
    let height = u32::try_from(size.height).ok()?;
    if width == 0 || height == 0 || width > MAX_DIMENSION || height > MAX_DIMENSION {
        return None;
    }
    Some((width, height))
}

#[cfg(test)]
mod tests {
    use super::{MAX_DIMENSION, image_dimensions};

    /// PNG シグネチャ + IHDR（幅・高さのみ）からなる最小ヘッダー。
    fn png_header(width: u32, height: u32) -> Vec<u8> {
        let mut bytes = b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR".to_vec();
        bytes.extend_from_slice(&width.to_be_bytes());
        bytes.extend_from_slice(&height.to_be_bytes());
        bytes
    }

    fn gif_header(width: u16, height: u16) -> Vec<u8> {
        let mut bytes = b"GIF89a".to_vec();
        bytes.extend_from_slice(&width.to_le_bytes());
        bytes.extend_from_slice(&height.to_le_bytes());
        // imagesize は最低 12 バイトを要求するため、GCT fields 等を埋める。
        bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
        bytes
    }

    #[test]
    fn png_dimensions() {
        assert_eq!(image_dimensions(&png_header(1, 1)), Some((1, 1)));
        assert_eq!(
            image_dimensions(&png_header(3000, 4000)),
            Some((3000, 4000))
        );
    }

    #[test]
    fn gif_dimensions() {
        assert_eq!(image_dimensions(&gif_header(20, 10)), Some((20, 10)));
    }

    #[test]
    fn rejects_zero_and_oversized() {
        assert_eq!(image_dimensions(&png_header(0, 10)), None);
        assert_eq!(image_dimensions(&png_header(10, 0)), None);
        assert_eq!(image_dimensions(&png_header(MAX_DIMENSION + 1, 10)), None);
        assert_eq!(image_dimensions(&png_header(10, MAX_DIMENSION + 1)), None);
        assert_eq!(
            image_dimensions(&png_header(MAX_DIMENSION, 1)),
            Some((MAX_DIMENSION, 1))
        );
    }

    #[test]
    fn rejects_unknown_or_broken_data() {
        assert_eq!(image_dimensions(b""), None);
        assert_eq!(image_dimensions(b"not an image"), None);
        // SVG は imagesize の対象外（web 側で別途処理する）。
        assert_eq!(image_dimensions(br#"<svg width="20" height="20"/>"#), None);
        // ヘッダーが切れていても panic せず None。
        assert_eq!(image_dimensions(&png_header(10, 10)[..20]), None);
    }
}
