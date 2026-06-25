// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// haniel_prism::text — Text layout helpers (wraps axon_layout text)

use axon_layout::{TextStyle, TextMetrics, measure_text, break_lines, text_fits_in, Size};
use crate::PrismError;

/// Sovereign text layout helper
pub struct TextLayout;

impl TextLayout {
    /// Measure text at default (16px) size
    pub fn measure(text: &str, max_width: f32) -> Result<TextMetrics, PrismError> {
        let style = TextStyle::default();
        measure_text(text, &style, max_width)
            .map_err(|e| PrismError::InvalidAxbw(format!("text measure error: {:?}", e)))
    }

    /// Measure text at specific font size
    pub fn measure_at(text: &str, font_size: f32, max_width: f32) -> Result<TextMetrics, PrismError> {
        let style = TextStyle::new(font_size);
        measure_text(text, &style, max_width)
            .map_err(|e| PrismError::InvalidAxbw(format!("text measure error: {:?}", e)))
    }

    /// Count lines needed to render text in given width
    pub fn line_count(text: &str, max_width: f32) -> usize {
        let style = TextStyle::default();
        break_lines(text, &style, max_width)
    }

    /// Check if text fits in given dimensions at default size
    pub fn fits(text: &str, w: f32, h: f32) -> bool {
        let style = TextStyle::default();
        let size  = Size { width: w, height: h };
        text_fits_in(text, &style, &size)
    }

    /// Heading size by level (h1=32, h2=28, h3=24, h4=20, h5=18, h6=16)
    pub fn heading_size(level: u8) -> f32 {
        match level {
            1 => 32.0,
            2 => 28.0,
            3 => 24.0,
            4 => 20.0,
            5 => 18.0,
            _ => 16.0,
        }
    }

    /// Measure heading text
    pub fn measure_heading(text: &str, level: u8, max_width: f32)
        -> Result<TextMetrics, PrismError>
    {
        Self::measure_at(text, Self::heading_size(level), max_width)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn measure_short_text() {
        let m = TextLayout::measure("Hello", 1280.0).unwrap();
        assert!(m.width > 0.0);
        assert!(m.height > 0.0);
        assert_eq!(m.lines, 1);
        assert_eq!(m.chars, 5);
    }

    #[test]
    fn measure_empty_text() {
        let m = TextLayout::measure("", 1280.0).unwrap();
        assert_eq!(m.width, 0.0);
        assert_eq!(m.height, 0.0);
        assert_eq!(m.lines, 0);
    }

    #[test]
    fn measure_wraps_long_text() {
        let long = "word ".repeat(100);
        let m    = TextLayout::measure(&long, 200.0).unwrap();
        assert!(m.lines > 1, "expected multiple lines, got {}", m.lines);
    }

    #[test]
    fn line_count_narrow_viewport() {
        let text  = "This is a somewhat long sentence for testing.";
        let lines = TextLayout::line_count(text, 100.0);
        assert!(lines >= 1);
    }

    #[test]
    fn text_fits_in_large_box() {
        assert!(TextLayout::fits("Hello", 1000.0, 1000.0));
    }

    #[test]
    fn text_does_not_fit_tiny_box() {
        let long = "word ".repeat(50);
        assert!(!TextLayout::fits(&long, 50.0, 10.0));
    }

    #[test]
    fn heading_sizes_decrease_by_level() {
        assert!(TextLayout::heading_size(1) > TextLayout::heading_size(2));
        assert!(TextLayout::heading_size(2) > TextLayout::heading_size(3));
        assert!(TextLayout::heading_size(3) > TextLayout::heading_size(4));
    }

    #[test]
    fn measure_heading_h1_larger_than_body() {
        let body    = TextLayout::measure("Hello", 1280.0).unwrap();
        let heading = TextLayout::measure_heading("Hello", 1, 1280.0).unwrap();
        assert!(heading.height > body.height,
            "h1 height {} should > body height {}", heading.height, body.height);
    }

    #[test]
    fn measure_at_large_font() {
        let m = TextLayout::measure_at("Big text", 48.0, 1280.0).unwrap();
        let s = TextLayout::measure("Big text", 1280.0).unwrap();
        assert!(m.height > s.height,
            "48px text {} should be taller than 16px text {}", m.height, s.height);
    }
}
