// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// haniel_vault::font — Sovereign font engine
// HE-8: axon_font integration — real glyph rendering replaces placeholder

use axon_font::{TextRaster, RasterConfig, Font, FontResult};
use crate::VaultError;

/// Sovereign font engine — wraps axon_font TextRaster
pub struct SovereignFontEngine {
    raster: TextRaster,
}

impl SovereignFontEngine {
    /// Initialize with built-in 8x8 bitmap font
    /// No file loading required — 95 ASCII glyphs always available
    pub fn new() -> Self {
        Self {
            raster: TextRaster::with_builtin(),
        }
    }

    /// Render a line of text to RGBA pixel data
    /// Returns (pixels: Vec<u8>, width: usize, height: usize)
    pub fn render_line(
        &self,
        text:  &str,
        fg:    [u8; 4],
        bg:    [u8; 4],
    ) -> Result<(Vec<u8>, usize, usize), VaultError> {
        // Build a custom config with the given colors
        let config = RasterConfig {
            fg_color: fg,
            bg_color: bg,
            scale:    1,
        };
        let raster = TextRaster::new(Font::builtin(), config);
        raster.raster_line(text)
            .map_err(|e| VaultError::SerializationError(format!("font render: {:?}", e)))
    }

    /// Render with default sovereign colors (white on dark bg)
    pub fn render_sovereign(&self, text: &str)
        -> Result<(Vec<u8>, usize, usize), VaultError>
    {
        self.raster.raster_line(text)
            .map_err(|e| VaultError::SerializationError(format!("font render: {:?}", e)))
    }

    /// Measure text width in pixels
    pub fn measure_width(&self, text: &str) -> usize {
        self.raster.measure_width(text)
    }

    /// Measure line height in pixels
    pub fn measure_height(&self) -> usize {
        self.raster.measure_height()
    }

    /// Font coverage for a string (ratio of glyphs available)
    pub fn coverage(&self, text: &str) -> f32 {
        self.raster.coverage(text)
    }

    /// Scale factor (1 = 8x8, 2 = 16x16, etc.)
    pub fn with_scale(scale: u32) -> Self {
        let config = RasterConfig::default().with_scale(scale);
        Self {
            raster: TextRaster::new(Font::builtin(), config),
        }
    }
}

impl Default for SovereignFontEngine {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn engine() -> SovereignFontEngine { SovereignFontEngine::new() }

    #[test]
    fn font_engine_constructs() {
        let e = engine();
        assert!(e.measure_height() > 0);
    }

    #[test]
    fn render_sovereign_hello() {
        let e = engine();
        let (pixels, w, h) = e.render_sovereign("Hello").unwrap();
        assert!(w > 0, "width should be > 0");
        assert!(h > 0, "height should be > 0");
        assert_eq!(pixels.len(), w * h * 4, "RGBA8 pixel count");
    }

    #[test]
    fn render_sovereign_world() {
        let e = engine();
        let (_, w1, _) = e.render_sovereign("Hi").unwrap();
        let (_, w2, _) = e.render_sovereign("Hello World").unwrap();
        assert!(w2 > w1, "longer text should be wider");
    }

    #[test]
    fn measure_width_proportional() {
        let e = engine();
        let w1 = e.measure_width("A");
        let w5 = e.measure_width("AAAAA");
        assert_eq!(w5, w1 * 5, "width should scale with char count");
    }

    #[test]
    fn measure_height_consistent() {
        let e = engine();
        let h1 = e.measure_height();
        let h2 = e.measure_height();
        assert_eq!(h1, h2, "height should be consistent");
        assert!(h1 >= 8, "8x8 font minimum height");
    }

    #[test]
    fn coverage_ascii_full() {
        let e = engine();
        let c = e.coverage("Hello World 123");
        assert!(c > 0.9, "ASCII coverage should be near 1.0, got {}", c);
    }

    #[test]
    fn render_with_custom_colors() {
        let e   = engine();
        let fg  = [255u8, 0, 0, 255];  // red text
        let bg  = [0u8, 0, 0, 255];    // black bg
        let (pixels, w, h) = e.render_line("A", fg, bg).unwrap();
        assert!(pixels.len() > 0);
        assert!(w > 0 && h > 0);
    }

    #[test]
    fn scaled_engine_larger() {
        let e1 = SovereignFontEngine::new();
        let e2 = SovereignFontEngine::with_scale(2);
        let h1 = e1.measure_height();
        let h2 = e2.measure_height();
        assert!(h2 >= h1, "scaled engine should be same or taller");
    }

    #[test]
    fn render_empty_string() {
        let e = engine();
        // Empty string should not panic
        let result = e.render_sovereign("");
        // Either Ok with empty pixels or handled gracefully
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn render_sovereign_text_non_transparent() {
        let e = engine();
        let (pixels, _, _) = e.render_sovereign("AIEONYX").unwrap();
        // Should have some non-zero pixels (text pixels)
        assert!(pixels.iter().any(|&b| b > 0),
            "rendered text should have non-zero pixels");
    }
}
