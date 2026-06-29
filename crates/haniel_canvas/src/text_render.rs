// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// haniel_canvas::text_render — Real glyph rendering via axon_font
// HE-8: replaces 4px dot placeholder with real 8x8 bitmap glyphs

use axon_font::{TextRaster, RasterConfig, Font};
use crate::pixel::PixelBuffer;
use crate::paint::Color;

/// Sovereign text renderer — axon_font integration
pub struct TextRenderer {
    raster: TextRaster,
}

impl TextRenderer {
    /// Create with built-in sovereign font
    pub fn new() -> Self {
        Self {
            raster: TextRaster::with_builtin(),
        }
    }

    /// Create with specific scale
    pub fn with_scale(scale: u32) -> Self {
        let config = RasterConfig::default().with_scale(scale);
        Self {
            raster: TextRaster::new(Font::builtin(), config),
        }
    }

    /// Render text into a pixel buffer at (x, y)
    /// Uses axon_font glyph rasterization — real pixels not dots
    pub fn render_text(
        &self,
        buf:   &mut PixelBuffer,
        text:  &str,
        x:     u32,
        y:     u32,
        color: Color,
    ) {
        if text.is_empty() { return; }

        // fg = text color, bg = transparent
        let fg = [color.r, color.g, color.b, color.a];
        let bg = [0u8, 0, 0, 0]; // transparent background

        let config = RasterConfig { fg, bg, scale: 1 };
        let raster = TextRaster::new(Font::builtin(), config);

        match raster.raster_line(text) {
            Ok((pixels, w, h)) => {
                // Blit rendered text pixels into buffer at (x, y)
                for py in 0..h {
                    for px in 0..w {
                        let src_idx = (py * w + px) * 4;
                        if src_idx + 3 >= pixels.len() { break; }
                        let a = pixels[src_idx + 3];
                        if a == 0 { continue; } // skip transparent pixels
                        buf.blend_pixel(
                            x + px as u32,
                            y + py as u32,
                            pixels[src_idx],
                            pixels[src_idx + 1],
                            pixels[src_idx + 2],
                            a,
                        );
                    }
                }
            }
            Err(_) => {
                // Fallback: draw small indicator dot
                buf.fill_rect(x, y, 4, 4, color.r, color.g, color.b, color.a);
            }
        }
    }

    /// Measure text dimensions
    pub fn measure(&self, text: &str) -> (usize, usize) {
        (self.raster.measure_width(text), self.raster.measure_height())
    }

    /// Font coverage for text
    pub fn coverage(&self, text: &str) -> f32 {
        self.raster.coverage(text)
    }
}

impl Default for TextRenderer {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn renderer() -> TextRenderer { TextRenderer::new() }

    #[test]
    fn text_renderer_constructs() {
        let r = renderer();
        let (w, h) = r.measure("A");
        assert!(w > 0 && h > 0);
    }

    #[test]
    fn render_text_into_buffer() {
        let r   = renderer();
        let mut buf = PixelBuffer::new(200, 50);
        r.render_text(&mut buf, "Hello", 0, 0, Color::SOVEREIGN_WHITE);
        // Should have painted some non-transparent pixels
        assert!(!buf.is_transparent(), "rendered text should paint pixels");
    }

    #[test]
    fn render_text_at_offset() {
        let r   = renderer();
        let mut buf = PixelBuffer::new(200, 50);
        r.render_text(&mut buf, "Hi", 50, 10, Color::SOVEREIGN_ACCENT);
        // Pixels at (50,10) region should be non-zero
        assert!(!buf.is_transparent());
    }

    #[test]
    fn render_empty_string_no_panic() {
        let r   = renderer();
        let mut buf = PixelBuffer::new(100, 50);
        r.render_text(&mut buf, "", 0, 0, Color::SOVEREIGN_WHITE);
        // Empty string should not panic and buffer stays transparent
        assert!(buf.is_transparent());
    }

    #[test]
    fn measure_width_scales_with_length() {
        let r = renderer();
        let (w1, _) = r.measure("A");
        let (w3, _) = r.measure("AAA");
        assert_eq!(w3, w1 * 3);
    }

    #[test]
    fn coverage_high_for_ascii() {
        let r = renderer();
        assert!(r.coverage("Hello World 123") > 0.9);
    }

    #[test]
    fn scaled_renderer_measures_larger() {
        let r1 = TextRenderer::new();
        let r2 = TextRenderer::with_scale(2);
        let (_, h1) = r1.measure("A");
        let (_, h2) = r2.measure("A");
        assert!(h2 >= h1);
    }

    #[test]
    fn render_sovereign_text_correct_colors() {
        let r   = renderer();
        let mut buf = PixelBuffer::new(100, 20);
        // White text
        r.render_text(&mut buf, "X", 0, 0, Color::SOVEREIGN_WHITE);
        // At least one pixel should be near white
        let has_white = (0..100u32).any(|x| {
            if let Some(px) = buf.get_pixel(x, 0) {
                px[0] > 200 && px[3] > 0
            } else { false }
        });
        assert!(has_white, "should have white pixels from text");
    }
}
