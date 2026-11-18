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

    /// SOI + SOF0 からなる最小 JPEG ヘッダー（12 バイト）。
    ///
    /// imagesize 0.15 は SOF0 マーカーの後、長さ 2 バイト + 精度 1 バイトを
    /// 飛ばし、オフセット 7-8 を height、9-10 を width として読む
    /// （PNG と逆の big-endian u16 順序）。形式判定に 12 バイト必要なため
    /// 末尾を 1 バイト埋める。
    fn jpeg_header(width: u16, height: u16) -> Vec<u8> {
        let mut bytes = b"\xFF\xD8\xFF\xC0\x00\x11\x08".to_vec();
        bytes.extend_from_slice(&height.to_be_bytes());
        bytes.extend_from_slice(&width.to_be_bytes());
        bytes.push(0x00);
        bytes
    }

    /// little-endian の下位 24 ビットを 3 バイトにする（WebP VP8X 用）。
    fn u24_le(value: u32) -> [u8; 3] {
        let bytes = value.to_le_bytes();
        [bytes[0], bytes[1], bytes[2]]
    }

    /// RIFF + WEBP + VP8X チャンクからなる最小ヘッダー（30 バイト）。
    ///
    /// imagesize 0.15 はオフセット 24-26 を (width - 1)、27-29 を
    /// (height - 1) として読む（little-endian u24、+1 エンコード）。
    fn webp_vp8x_header(width: u32, height: u32) -> Vec<u8> {
        let mut bytes = b"RIFF".to_vec();
        bytes.extend_from_slice(&22u32.to_le_bytes()); // ダミーのファイルサイズ
        bytes.extend_from_slice(b"WEBP");
        bytes.extend_from_slice(b"VP8X");
        bytes.extend_from_slice(&10u32.to_le_bytes()); // ダミーのチャンクサイズ
        bytes.extend_from_slice(&[0x00; 4]); // フラグ + 予約
        bytes.extend_from_slice(&u24_le(width - 1));
        bytes.extend_from_slice(&u24_le(height - 1));
        bytes
    }

    /// RIFF + WEBP + VP8 (lossy) チャンクからなる最小ヘッダー（30 バイト）。
    ///
    /// imagesize 0.15 はオフセット 26-27 を width、28-29 を height として
    /// 読む（little-endian u16、+1 エンコードなし）。
    fn webp_vp8_header(width: u16, height: u16) -> Vec<u8> {
        let mut bytes = b"RIFF".to_vec();
        bytes.extend_from_slice(&22u32.to_le_bytes()); // ダミーのファイルサイズ
        bytes.extend_from_slice(b"WEBP");
        bytes.extend_from_slice(b"VP8 ");
        bytes.extend_from_slice(&10u32.to_le_bytes()); // ダミーのチャンクサイズ
        // フレームタグ + 同期コード（imagesize は参照しない）。
        bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x9D, 0x01, 0x2A]);
        bytes.extend_from_slice(&width.to_le_bytes());
        bytes.extend_from_slice(&height.to_le_bytes());
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
    fn jpeg_dimensions_from_sof0() {
        assert_eq!(image_dimensions(&jpeg_header(20, 10)), Some((20, 10)));
        assert_eq!(
            image_dimensions(&jpeg_header(3000, 4000)),
            Some((3000, 4000))
        );
    }

    #[test]
    fn webp_dimensions_from_vp8x() {
        assert_eq!(image_dimensions(&webp_vp8x_header(20, 10)), Some((20, 10)));
        assert_eq!(
            image_dimensions(&webp_vp8x_header(MAX_DIMENSION, 1)),
            Some((MAX_DIMENSION, 1))
        );
    }

    #[test]
    fn webp_dimensions_from_vp8_lossy() {
        assert_eq!(image_dimensions(&webp_vp8_header(20, 10)), Some((20, 10)));
        assert_eq!(
            image_dimensions(&webp_vp8_header(3000, 4000)),
            Some((3000, 4000))
        );
    }

    #[test]
    fn png_boundaries_table_driven() {
        let cases = [
            (0, 10, None),
            (10, 0, None),
            (1, 1, Some((1, 1))),
            (MAX_DIMENSION, 1, Some((MAX_DIMENSION, 1))),
            (1, MAX_DIMENSION, Some((1, MAX_DIMENSION))),
            (MAX_DIMENSION + 1, 10, None),
            (10, MAX_DIMENSION + 1, None),
        ];
        for (width, height, expected) in cases {
            assert_eq!(
                image_dimensions(&png_header(width, height)),
                expected,
                "width={width}, height={height}"
            );
        }
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
