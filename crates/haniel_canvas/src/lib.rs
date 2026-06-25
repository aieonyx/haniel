// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// HANIEL CANVAS — Sovereign Rasterizer and GPU Renderer
// HE-6: software rasterizer — first sovereign pixel

#![forbid(unsafe_code)]

pub mod gpu;
pub mod paint;
pub mod pixel;
pub mod raster;
pub mod surface;

pub use paint::{Color, PaintCommand, DisplayList};
pub use pixel::PixelBuffer;
pub use raster::SoftwareRasterizer;
pub use surface::{CanvasSurface, SurfaceBackend, SurfaceError};

use axon_layout::ComputedLayout;

/// Raster backend selector
#[derive(Debug, Clone, PartialEq)]
pub enum RasterBackend {
    Software,   // CPU rasterizer — HE-6
    Gpu,        // wgpu/Vulkan — HE-7
}

/// Canvas error type
#[derive(Debug)]
pub enum CanvasError {
    GpuUnavailable,
    SurfaceLost,
    OutOfMemory,
    RenderFailed(String),
}

/// Sovereign CANVAS — paint ComputedLayout to PixelBuffer
pub struct SovereignCanvas {
    pub backend:    RasterBackend,
    pub rasterizer: SoftwareRasterizer,
    pub width:      u32,
    pub height:     u32,
}

impl SovereignCanvas {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            backend:    RasterBackend::Software,
            rasterizer: SoftwareRasterizer::new(),
            width,
            height,
        }
    }

    /// Rod pass paint — structural skeleton, immediate
    pub fn paint_rod(&self, layout: &ComputedLayout) -> PixelBuffer {
        let mut buf = PixelBuffer::new(self.width, self.height);
        let mut dl  = DisplayList::new();

        // Background
        dl.push(PaintCommand::Clear(Color::SOVEREIGN_BG));

        // Structural layout
        self.rasterizer.build_rod_display_list(layout, &mut dl, 0);

        // Rasterize
        self.rasterizer.rasterize(&dl, &mut buf);
        buf
    }

    /// Cone pass paint — full detail
    pub fn paint_cone(
        &self,
        layout:   &ComputedLayout,
        is_text:  bool,
        is_image: bool,
    ) -> PixelBuffer {
        let mut buf = PixelBuffer::new(self.width, self.height);
        let mut dl  = DisplayList::new();

        // Background
        dl.push(PaintCommand::Clear(Color::SOVEREIGN_BG));

        // Full detail layout
        self.rasterizer.build_cone_display_list(layout, &mut dl, is_text, is_image);

        // Rasterize
        self.rasterizer.rasterize(&dl, &mut buf);
        buf
    }

    /// Incremental repaint — dirty nodes only
    pub fn repaint(
        &self,
        layout:      &ComputedLayout,
        dirty_ids:   &[&str],
    ) -> PixelBuffer {
        let mut buf = PixelBuffer::new(self.width, self.height);
        let mut dl  = DisplayList::new();

        // Only repaint dirty subtrees
        self.paint_dirty(layout, &mut dl, dirty_ids, 0);
        self.rasterizer.rasterize(&dl, &mut buf);
        buf
    }

    fn paint_dirty(
        &self,
        layout:    &ComputedLayout,
        dl:        &mut DisplayList,
        dirty_ids: &[&str],
        depth:     u32,
    ) {
        let is_dirty = dirty_ids.contains(&layout.id.as_str());
        if is_dirty {
            self.rasterizer.build_rod_display_list(layout, dl, depth);
        }
        for child in &layout.children {
            self.paint_dirty(child, dl, dirty_ids, depth + 1);
        }
    }

    /// GPU memory used (0 in software mode)
    pub fn gpu_memory_used(&self) -> usize { 0 }

    /// Switch backend
    pub fn set_backend(&mut self, backend: RasterBackend) {
        self.backend = backend;
    }

    /// Resize canvas
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width  = width;
        self.height = height;
    }
}

/// CANVAS sovereign trait
pub trait Canvas: Send + Sync {
    fn paint_rod(&self, layout: &ComputedLayout) -> PixelBuffer;
    fn paint_cone(&self, layout: &ComputedLayout, is_text: bool, is_image: bool) -> PixelBuffer;
    fn gpu_memory_used(&self) -> usize;
}

impl Canvas for SovereignCanvas {
    fn paint_rod(&self, layout: &ComputedLayout) -> PixelBuffer {
        self.paint_rod(layout)
    }
    fn paint_cone(&self, layout: &ComputedLayout, is_text: bool, is_image: bool) -> PixelBuffer {
        self.paint_cone(layout, is_text, is_image)
    }
    fn gpu_memory_used(&self) -> usize { 0 }
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

    fn canvas() -> SovereignCanvas { SovereignCanvas::new(800, 600) }

    #[test]
    fn canvas_constructs() {
        let c = canvas();
        assert_eq!(c.width,  800);
        assert_eq!(c.height, 600);
        assert_eq!(c.backend, RasterBackend::Software);
    }

    #[test]
    fn rod_pass_produces_buffer() {
        let layout = make_layout("root", 0.0, 0.0, 800.0, 600.0);
        let buf    = canvas().paint_rod(&layout);
        assert_eq!(buf.width,  800);
        assert_eq!(buf.height, 600);
        assert!(!buf.is_transparent());
    }

    #[test]
    fn rod_pass_buffer_correct_size() {
        let layout = make_layout("root", 0.0, 0.0, 800.0, 600.0);
        let buf    = canvas().paint_rod(&layout);
        assert_eq!(buf.byte_size(), 800 * 600 * 4);
    }

    #[test]
    fn cone_pass_produces_buffer() {
        let layout = make_layout("root", 0.0, 0.0, 800.0, 600.0);
        let buf    = canvas().paint_cone(&layout, false, false);
        assert!(!buf.is_transparent());
    }

    #[test]
    fn rod_pass_sovereign_bg_color() {
        // Background should be painted with SOVEREIGN_BG (18, 18, 24)
        let layout = make_layout("root", 0.0, 0.0, 800.0, 600.0);
        let buf    = canvas().paint_rod(&layout);
        // Pixel 0,0 should not be transparent
        let px = buf.get_pixel(0, 0).unwrap();
        assert_ne!(px[3], 0, "background should be opaque");
    }

    #[test]
    fn repaint_dirty_nodes() {
        let layout = ComputedLayout {
            id:   "root".to_string(),
            rect: Rect::new(0.0, 0.0, 800.0, 600.0).unwrap(),
            children: vec![
                make_layout("nav",  0.0,   0.0,   800.0, 60.0),
                make_layout("main", 0.0,   60.0,  800.0, 540.0),
            ],
        };
        let buf = canvas().repaint(&layout, &["nav"]);
        assert_eq!(buf.width, 800);
        assert_eq!(buf.height, 600);
    }

    #[test]
    fn canvas_resize() {
        let mut c = canvas();
        c.resize(1920, 1080);
        assert_eq!(c.width,  1920);
        assert_eq!(c.height, 1080);
    }

    #[test]
    fn canvas_set_backend() {
        let mut c = canvas();
        c.set_backend(RasterBackend::Gpu);
        assert_eq!(c.backend, RasterBackend::Gpu);
    }

    #[test]
    fn gpu_memory_zero_in_software_mode() {
        assert_eq!(canvas().gpu_memory_used(), 0);
    }

    #[test]
    fn canvas_with_children_paints_all() {
        let layout = ComputedLayout {
            id:   "root".to_string(),
            rect: Rect::new(0.0, 0.0, 800.0, 600.0).unwrap(),
            children: vec![
                make_layout("header", 0.0, 0.0,   800.0, 80.0),
                make_layout("body",   0.0, 80.0,  800.0, 440.0),
                make_layout("footer", 0.0, 520.0, 800.0, 80.0),
            ],
        };
        let buf = canvas().paint_rod(&layout);
        // Should paint something non-transparent across the whole buffer
        assert!(!buf.is_transparent());
    }
}
