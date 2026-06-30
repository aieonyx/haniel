// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// haniel_canvas::codec — PixelBuffer image encoding
// HE-15b: bridge HANIEL's raw RGBA8 output to a standard image format so it
// can be served to a host webview (Tauri custom protocol, etc).
//
// Deliberately a thin, isolated boundary: today this only encodes PNG, but
// the `image` crate dependency is feature-gated to `png` alone. Future
// in-page <img> decoding support (JPEG, WebP, GIF...) is additive — enable
// more `image` features and add decode functions here without touching
// PixelBuffer, CANVAS, or any caller of `encode_png`.

use crate::pixel::PixelBuffer;
use image::{ImageBuffer, ImageError, Rgba};
use std::io::Cursor;

/// Error encoding a PixelBuffer to an image format
#[derive(Debug)]
pub enum CodecError {
    /// The buffer's width/height/data length are inconsistent — cannot
    /// even construct an intermediate image representation.
    InvalidBuffer,
    /// The underlying image crate failed to encode.
    EncodeFailed(ImageError),
}

impl std::fmt::Display for CodecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CodecError::InvalidBuffer => write!(f, "pixel buffer dimensions do not match data length"),
            CodecError::EncodeFailed(e) => write!(f, "image encode failed: {}", e),
        }
    }
}

/// Encode a PixelBuffer (RGBA8, row-major) as PNG bytes.
///
/// This is the only encode path HE-15b needs: HANIEL's CANVAS output goes
/// straight to PNG bytes for serving through a custom protocol handler.
pub fn encode_png(buffer: &PixelBuffer) -> Result<Vec<u8>, CodecError> {
    let image: ImageBuffer<Rgba<u8>, Vec<u8>> =
        ImageBuffer::from_raw(buffer.width, buffer.height, buffer.data.clone())
            .ok_or(CodecError::InvalidBuffer)?;

    let mut out = Cursor::new(Vec::new());
    image
        .write_to(&mut out, image::ImageFormat::Png)
        .map_err(CodecError::EncodeFailed)?;

    Ok(out.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solid_buffer(w: u32, h: u32, r: u8, g: u8, b: u8, a: u8) -> PixelBuffer {
        PixelBuffer::filled(w, h, r, g, b, a)
    }

    #[test]
    fn encode_png_produces_nonempty_bytes() {
        let buf = solid_buffer(4, 4, 255, 0, 0, 255);
        let png = encode_png(&buf).unwrap();
        assert!(!png.is_empty());
    }

    #[test]
    fn encode_png_has_valid_signature() {
        // PNG files always start with this 8-byte magic signature.
        let buf = solid_buffer(2, 2, 0, 255, 0, 255);
        let png = encode_png(&buf).unwrap();
        assert_eq!(&png[0..8], &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
    }

    #[test]
    fn encode_png_roundtrips_dimensions() {
        let buf = solid_buffer(10, 7, 0, 0, 255, 255);
        let png = encode_png(&buf).unwrap();
        let decoded = image::load_from_memory(&png).unwrap();
        assert_eq!(decoded.width(), 10);
        assert_eq!(decoded.height(), 7);
    }

    #[test]
    fn encode_png_roundtrips_pixel_color() {
        let buf = solid_buffer(3, 3, 12, 34, 56, 200);
        let png = encode_png(&buf).unwrap();
        let decoded = image::load_from_memory(&png).unwrap().to_rgba8();
        let px = decoded.get_pixel(1, 1);
        assert_eq!(px.0, [12, 34, 56, 200]);
    }

    #[test]
    fn encode_png_preserves_alpha() {
        let buf = solid_buffer(2, 2, 255, 255, 255, 0);
        let png = encode_png(&buf).unwrap();
        let decoded = image::load_from_memory(&png).unwrap().to_rgba8();
        assert_eq!(decoded.get_pixel(0, 0).0[3], 0);
    }

    #[test]
    fn encode_png_single_pixel() {
        let buf = solid_buffer(1, 1, 1, 2, 3, 4);
        let png = encode_png(&buf).unwrap();
        assert!(!png.is_empty());
    }

    #[test]
    fn encode_png_large_dimensions() {
        let buf = PixelBuffer::new(1920, 1080);
        let png = encode_png(&buf).unwrap();
        let decoded = image::load_from_memory(&png).unwrap();
        assert_eq!(decoded.width(), 1920);
        assert_eq!(decoded.height(), 1080);
    }

    #[test]
    fn invalid_buffer_data_length_errors() {
        let bad = PixelBuffer {
            width: 4,
            height: 4,
            data: vec![0u8; 10], // too short for 4x4 RGBA8 (needs 64 bytes)
        };
        assert!(matches!(encode_png(&bad), Err(CodecError::InvalidBuffer)));
    }

    #[test]
    fn codec_error_display_invalid_buffer() {
        let e = CodecError::InvalidBuffer;
        assert!(e.to_string().contains("dimensions"));
    }
}
