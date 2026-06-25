// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// haniel_canvas::raster — Sovereign software rasterizer
// Converts DisplayList + ComputedLayout into PixelBuffer
// Rod pass: immediate structural skeleton
// Cone pass: full detail (text placeholder until HE-8)

use axon_layout::ComputedLayout;
use crate::pixel::PixelBuffer;
use crate::paint::{PaintCommand, DisplayList, Color};

/// Software rasterizer — CPU-only, always available
pub struct SoftwareRasterizer;

impl SoftwareRasterizer {
    pub fn new() -> Self { Self }

    /// Build display list from computed layout (rod pass)
    /// Fast structural skeleton — solid color blocks only
    pub fn build_rod_display_list(
        &self,
        layout: &ComputedLayout,
        dl:     &mut DisplayList,
        depth:  u32,
    ) {
        let r = &layout.rect;
        let x = r.min_x();
        let y = r.min_y();
        let w = r.size.width;
        let h = r.size.height;

        if w <= 0.0 || h <= 0.0 { return; }

        // Rod pass: alternating structural colors by depth
        let bg = Self::depth_color(depth);
        dl.push(PaintCommand::FillRect { x, y, w, h, color: bg });

        // Border to show structure
        dl.push(PaintCommand::StrokeRect {
            x, y, w, h,
            color: Color::SOVEREIGN_BORDER,
        });

        for child in &layout.children {
            self.build_rod_display_list(child, dl, depth + 1);
        }
    }

    /// Build display list from computed layout (cone pass)
    /// Full detail — text placeholders, image placeholders, correct colors
    pub fn build_cone_display_list(
        &self,
        layout:   &ComputedLayout,
        dl:       &mut DisplayList,
        is_text:  bool,
        is_image: bool,
    ) {
        let r = &layout.rect;
        let x = r.min_x();
        let y = r.min_y();
        let w = r.size.width;
        let h = r.size.height;

        if w <= 0.0 || h <= 0.0 { return; }

        if is_image {
            // Image placeholder — checkerboard pattern (real decode at HE-10)
            dl.push(PaintCommand::Image {
                x, y, w, h,
                src: format!("node:{}", layout.id),
            });
        } else if is_text {
            // Text placeholder — sovereign accent underline
            dl.push(PaintCommand::FillRect {
                x, y, w, h,
                color: Color::new(30, 30, 40, 255),
            });
            dl.push(PaintCommand::Text {
                x: x + 2.0, y: y + 2.0,
                content: format!("[text:{}]", layout.id),
                size: 16.0,
                color: Color::SOVEREIGN_WHITE,
            });
        } else {
            // Block/flex/inline node
            dl.push(PaintCommand::FillRect {
                x, y, w, h,
                color: Color::SOVEREIGN_BG,
            });
            dl.push(PaintCommand::StrokeRect {
                x, y, w, h,
                color: Color::SOVEREIGN_BORDER,
            });
        }

        for child in &layout.children {
            self.build_cone_display_list(child, dl, false, false);
        }
    }

    /// Rasterize a display list into a pixel buffer
    pub fn rasterize(
        &self,
        dl:  &DisplayList,
        buf: &mut PixelBuffer,
    ) {
        for cmd in dl.commands() {
            match cmd {
                PaintCommand::Clear(color) => {
                    buf.clear_color(color.r, color.g, color.b, color.a);
                }
                PaintCommand::FillRect { x, y, w, h, color } => {
                    buf.fill_rect(
                        *x as u32, *y as u32,
                        *w as u32, *h as u32,
                        color.r, color.g, color.b, color.a,
                    );
                }
                PaintCommand::StrokeRect { x, y, w, h, color } => {
                    buf.stroke_rect(
                        *x as u32, *y as u32,
                        *w as u32, *h as u32,
                        color.r, color.g, color.b, color.a,
                    );
                }
                PaintCommand::Text { x, y, color, .. } => {
                    // Text placeholder: 4px accent dot at text origin
                    // Real glyph rendering wired at HE-8
                    buf.fill_rect(*x as u32, *y as u32, 4, 4,
                        color.r, color.g, color.b, color.a);
                }
                PaintCommand::Image { x, y, w, h, .. } => {
                    // Image placeholder: magenta checkerboard
                    // Real decode wired at HE-10
                    let x  = *x as u32;
                    let y  = *y as u32;
                    let w  = *w as u32;
                    let h  = *h as u32;
                    let x2 = (x + w).min(buf.width);
                    let y2 = (y + h).min(buf.height);
                    for py in y..y2 {
                        for px in x..x2 {
                            let checker = (px / 8 + py / 8) % 2 == 0;
                            if checker {
                                buf.set_pixel(px, py, 180, 0, 180, 255);
                            } else {
                                buf.set_pixel(px, py, 120, 0, 120, 255);
                            }
                        }
                    }
                }
                PaintCommand::Blit { .. } => {
                    // Sub-buffer blit wired at HE-7
                }
            }
        }
    }

    /// Depth-based structural color for rod pass visualization
    fn depth_color(depth: u32) -> Color {
        match depth % 5 {
            0 => Color::new(20,  20,  35,  255),
            1 => Color::new(25,  30,  50,  255),
            2 => Color::new(30,  35,  60,  255),
            3 => Color::new(20,  40,  55,  255),
            _ => Color::new(25,  45,  65,  255),
        }
    }
}

impl Default for SoftwareRasterizer {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axon_layout::{ComputedLayout, Rect};

    fn make_layout(id: &str, x: f32, y: f32, w: f32, h: f32) -> ComputedLayout {
        ComputedLayout {
            id:       id.to_string(),
            rect:     Rect::new(x, y, w, h).unwrap(),
            children: vec![],
        }
    }

    fn rasterizer() -> SoftwareRasterizer { SoftwareRasterizer::new() }

    #[test]
    fn rasterize_fill_rect() {
        let mut buf = PixelBuffer::new(100, 100);
        let mut dl  = DisplayList::new();
        dl.push(PaintCommand::FillRect {
            x: 10.0, y: 10.0, w: 20.0, h: 20.0,
            color: Color::rgb(255, 0, 0),
        });
        rasterizer().rasterize(&dl, &mut buf);
        assert_eq!(buf.get_pixel(10, 10).unwrap(), [255, 0, 0, 255]);
        assert_eq!(buf.get_pixel(29, 29).unwrap(), [255, 0, 0, 255]);
    }

    #[test]
    fn rasterize_clear() {
        let mut buf = PixelBuffer::new(100, 100);
        let mut dl  = DisplayList::new();
        dl.push(PaintCommand::Clear(Color::rgb(18, 18, 24)));
        rasterizer().rasterize(&dl, &mut buf);
        assert_eq!(buf.get_pixel(0, 0).unwrap(), [18, 18, 24, 255]);
        assert_eq!(buf.get_pixel(99, 99).unwrap(), [18, 18, 24, 255]);
    }

    #[test]
    fn rasterize_stroke_rect() {
        let mut buf = PixelBuffer::new(100, 100);
        let mut dl  = DisplayList::new();
        dl.push(PaintCommand::StrokeRect {
            x: 5.0, y: 5.0, w: 10.0, h: 10.0,
            color: Color::SOVEREIGN_ACCENT,
        });
        rasterizer().rasterize(&dl, &mut buf);
        // Border pixel should be set
        assert_eq!(
            buf.get_pixel(5, 5).unwrap(),
            [Color::SOVEREIGN_ACCENT.r, Color::SOVEREIGN_ACCENT.g,
             Color::SOVEREIGN_ACCENT.b, 255]
        );
    }

    #[test]
    fn rod_display_list_from_layout() {
        let layout = make_layout("root", 0.0, 0.0, 1280.0, 720.0);
        let mut dl = DisplayList::new();
        rasterizer().build_rod_display_list(&layout, &mut dl, 0);
        assert!(!dl.is_empty(), "rod pass should produce paint commands");
        assert!(dl.len() >= 2, "expected at least fill + stroke");
    }

    #[test]
    fn rod_display_list_recursive() {
        let layout = ComputedLayout {
            id:   "root".to_string(),
            rect: Rect::new(0.0, 0.0, 1280.0, 720.0).unwrap(),
            children: vec![
                make_layout("child1", 0.0, 0.0, 640.0, 720.0),
                make_layout("child2", 640.0, 0.0, 640.0, 720.0),
            ],
        };
        let mut dl = DisplayList::new();
        rasterizer().build_rod_display_list(&layout, &mut dl, 0);
        // Root (2 cmds) + child1 (2 cmds) + child2 (2 cmds) = 6
        assert!(dl.len() >= 6, "expected commands for root + 2 children");
    }

    #[test]
    fn rod_rasterize_produces_non_transparent_buffer() {
        let layout = make_layout("root", 0.0, 0.0, 100.0, 100.0);
        let mut dl  = DisplayList::new();
        let mut buf = PixelBuffer::new(100, 100);
        rasterizer().build_rod_display_list(&layout, &mut dl, 0);
        rasterizer().rasterize(&dl, &mut buf);
        // Buffer should have at least some non-transparent pixels
        assert!(!buf.is_transparent(), "rasterized buffer should not be transparent");
    }

    #[test]
    fn cone_display_list_from_layout() {
        let layout = make_layout("root", 0.0, 0.0, 1280.0, 720.0);
        let mut dl = DisplayList::new();
        rasterizer().build_cone_display_list(&layout, &mut dl, false, false);
        assert!(!dl.is_empty());
    }

    #[test]
    fn depth_color_cycles() {
        // Ensure depth colors don't panic
        for d in 0..10 {
            let _ = SoftwareRasterizer::depth_color(d);
        }
    }

    #[test]
    fn rasterize_image_placeholder_paints() {
        let mut buf = PixelBuffer::new(100, 100);
        let mut dl  = DisplayList::new();
        dl.push(PaintCommand::Image {
            x: 0.0, y: 0.0, w: 50.0, h: 50.0,
            src: "test.png".into(),
        });
        rasterizer().rasterize(&dl, &mut buf);
        // Checkerboard: first pixel (0,0) in block 0,0 → even → magenta
        let px = buf.get_pixel(0, 0).unwrap();
        assert_eq!(px[0], 180); // magenta R
        assert_eq!(px[2], 180); // magenta B
    }
}
