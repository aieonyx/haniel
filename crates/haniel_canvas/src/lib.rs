// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// HANIEL CANVAS — Sovereign Rasterizer and GPU Renderer
// HE-6 (software) + HE-7 (GPU) implementation target

#![forbid(unsafe_code)]

pub mod raster;
pub mod gpu;
pub mod paint;
pub mod surface;

// LayoutTree and u32 imported from haniel_prism at HE-6

/// Pixel buffer — RGBA8
#[derive(Debug)]
pub struct PixelBuffer {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

impl PixelBuffer {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            data: vec![0u8; (width * height * 4) as usize],
        }
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, r: u8, g: u8, b: u8, a: u8) {
        let idx = ((y * self.width + x) * 4) as usize;
        if idx + 3 < self.data.len() {
            self.data[idx]     = r;
            self.data[idx + 1] = g;
            self.data[idx + 2] = b;
            self.data[idx + 3] = a;
        }
    }
}

/// Raster backend selector
#[derive(Debug, Clone, PartialEq)]
pub enum RasterBackend {
    Software,
    Gpu,
}

/// CANVAS sovereign rasterizer trait
pub trait Canvas: Send + Sync {
    fn paint(&self) -> Result<PixelBuffer, CanvasError>;
    fn repaint(&self, dirty: &[u32]) -> Result<PixelBuffer, CanvasError>;
    fn set_backend(&mut self, backend: RasterBackend);
    fn gpu_memory_used(&self) -> usize;
}

/// CANVAS error type
#[derive(Debug)]
pub enum CanvasError {
    GpuUnavailable,
    SurfaceLost,
    OutOfMemory,
    RenderFailed(String),
}

/// Paint command
#[derive(Debug, Clone)]
pub enum PaintCommand {
    FillRect { x: f32, y: f32, w: f32, h: f32, color: [u8; 4] },
    Text     { x: f32, y: f32, text: String, size: f32, color: [u8; 4] },
    Image    { x: f32, y: f32, w: f32, h: f32, texture_id: u32 },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pixel_buffer_constructs() {
        let buf = PixelBuffer::new(800, 600);
        assert_eq!(buf.data.len(), 800 * 600 * 4);
    }

    #[test]
    fn pixel_buffer_set_pixel() {
        let mut buf = PixelBuffer::new(10, 10);
        buf.set_pixel(0, 0, 255, 0, 0, 255);
        assert_eq!(buf.data[0], 255);
        assert_eq!(buf.data[1], 0);
        assert_eq!(buf.data[2], 0);
        assert_eq!(buf.data[3], 255);
    }

    #[test]
    fn raster_backend_variants_exist() {
        assert_ne!(RasterBackend::Software, RasterBackend::Gpu);
    }
}
