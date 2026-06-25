// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// haniel_canvas::surface — Display surface abstraction
// Software surface for HE-6; GPU surface (wgpu/Vulkan) at HE-7

use crate::pixel::PixelBuffer;

/// Surface backend selector
#[derive(Debug, Clone, PartialEq)]
pub enum SurfaceBackend {
    Software,   // CPU rasterizer — current
    Gpu,        // wgpu/Vulkan — HE-7
}

/// Display surface — where pixels go after rasterization
pub struct CanvasSurface {
    pub backend: SurfaceBackend,
    pub width:   u32,
    pub height:  u32,
}

impl CanvasSurface {
    pub fn new_software(width: u32, height: u32) -> Self {
        Self { backend: SurfaceBackend::Software, width, height }
    }

    /// Present a pixel buffer to this surface
    /// Software path: writes to memory (real display at HE-7)
    pub fn present(&self, buf: &PixelBuffer) -> Result<(), SurfaceError> {
        if buf.width != self.width || buf.height != self.height {
            return Err(SurfaceError::SizeMismatch {
                expected: (self.width, self.height),
                got:      (buf.width, buf.height),
            });
        }
        // Software path: buffer is ready — GPU blit at HE-7
        Ok(())
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width  = width;
        self.height = height;
    }
}

/// Surface error
#[derive(Debug)]
pub enum SurfaceError {
    SizeMismatch { expected: (u32, u32), got: (u32, u32) },
    GpuUnavailable,
    SurfaceLost,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn software_surface_constructs() {
        let s = CanvasSurface::new_software(1280, 720);
        assert_eq!(s.backend, SurfaceBackend::Software);
        assert_eq!(s.width,  1280);
        assert_eq!(s.height, 720);
    }

    #[test]
    fn present_matching_buffer_ok() {
        let s   = CanvasSurface::new_software(100, 100);
        let buf = PixelBuffer::new(100, 100);
        assert!(s.present(&buf).is_ok());
    }

    #[test]
    fn present_mismatched_buffer_errors() {
        let s   = CanvasSurface::new_software(100, 100);
        let buf = PixelBuffer::new(200, 200);
        assert!(matches!(s.present(&buf), Err(SurfaceError::SizeMismatch { .. })));
    }

    #[test]
    fn resize_updates_dimensions() {
        let mut s = CanvasSurface::new_software(1280, 720);
        s.resize(1920, 1080);
        assert_eq!(s.width,  1920);
        assert_eq!(s.height, 1080);
    }

    #[test]
    fn backend_variants_exist() {
        assert_ne!(SurfaceBackend::Software, SurfaceBackend::Gpu);
    }
}
