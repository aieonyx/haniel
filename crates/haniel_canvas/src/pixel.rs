// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// haniel_canvas::pixel — Sovereign pixel buffer

/// RGBA8 pixel buffer — the fundamental output unit of HANIEL CANVAS
#[derive(Debug, Clone)]
pub struct PixelBuffer {
    pub width:  u32,
    pub height: u32,
    pub data:   Vec<u8>,  // RGBA8: 4 bytes per pixel, row-major
}

impl PixelBuffer {
    /// Create a new pixel buffer filled with transparent black
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            data: vec![0u8; (width * height * 4) as usize],
        }
    }

    /// Create filled with a solid color
    pub fn filled(width: u32, height: u32, r: u8, g: u8, b: u8, a: u8) -> Self {
        let mut buf = Self::new(width, height);
        for px in buf.data.chunks_exact_mut(4) {
            px[0] = r; px[1] = g; px[2] = b; px[3] = a;
        }
        buf
    }

    /// Set a single pixel — bounds checked
    #[inline]
    pub fn set_pixel(&mut self, x: u32, y: u32, r: u8, g: u8, b: u8, a: u8) {
        if x >= self.width || y >= self.height { return; }
        let idx = ((y * self.width + x) * 4) as usize;
        self.data[idx]     = r;
        self.data[idx + 1] = g;
        self.data[idx + 2] = b;
        self.data[idx + 3] = a;
    }

    /// Get a single pixel — bounds checked
    #[inline]
    pub fn get_pixel(&self, x: u32, y: u32) -> Option<[u8; 4]> {
        if x >= self.width || y >= self.height { return None; }
        let idx = ((y * self.width + x) * 4) as usize;
        Some([self.data[idx], self.data[idx+1], self.data[idx+2], self.data[idx+3]])
    }

    /// Alpha-blend a pixel over the existing pixel (Porter-Duff over)
    #[inline]
    pub fn blend_pixel(&mut self, x: u32, y: u32, r: u8, g: u8, b: u8, a: u8) {
        if x >= self.width || y >= self.height { return; }
        let idx = ((y * self.width + x) * 4) as usize;

        let src_a = a as f32 / 255.0;
        let dst_a = self.data[idx + 3] as f32 / 255.0;
        let out_a = src_a + dst_a * (1.0 - src_a);

        if out_a < f32::EPSILON {
            self.data[idx]     = 0;
            self.data[idx + 1] = 0;
            self.data[idx + 2] = 0;
            self.data[idx + 3] = 0;
            return;
        }

        self.data[idx]     = ((r as f32 * src_a
            + self.data[idx]     as f32 * dst_a * (1.0 - src_a)) / out_a) as u8;
        self.data[idx + 1] = ((g as f32 * src_a
            + self.data[idx + 1] as f32 * dst_a * (1.0 - src_a)) / out_a) as u8;
        self.data[idx + 2] = ((b as f32 * src_a
            + self.data[idx + 2] as f32 * dst_a * (1.0 - src_a)) / out_a) as u8;
        self.data[idx + 3] = (out_a * 255.0) as u8;
    }

    /// Fill a rectangular region
    pub fn fill_rect(&mut self, x: u32, y: u32, w: u32, h: u32, r: u8, g: u8, b: u8, a: u8) {
        let x_end = (x + w).min(self.width);
        let y_end = (y + h).min(self.height);
        for py in y..y_end {
            for px in x..x_end {
                self.set_pixel(px, py, r, g, b, a);
            }
        }
    }

    /// Draw a 1px border around a rect
    pub fn stroke_rect(&mut self, x: u32, y: u32, w: u32, h: u32, r: u8, g: u8, b: u8, a: u8) {
        if w == 0 || h == 0 { return; }
        // Top and bottom rows
        for px in x..(x+w).min(self.width) {
            self.set_pixel(px, y, r, g, b, a);
            if y + h - 1 < self.height {
                self.set_pixel(px, y + h - 1, r, g, b, a);
            }
        }
        // Left and right columns
        for py in y..(y+h).min(self.height) {
            self.set_pixel(x, py, r, g, b, a);
            if x + w - 1 < self.width {
                self.set_pixel(x + w - 1, py, r, g, b, a);
            }
        }
    }

    /// Clear the buffer to transparent black
    pub fn clear(&mut self) {
        self.data.fill(0);
    }

    /// Clear to a solid color
    pub fn clear_color(&mut self, r: u8, g: u8, b: u8, a: u8) {
        for px in self.data.chunks_exact_mut(4) {
            px[0] = r; px[1] = g; px[2] = b; px[3] = a;
        }
    }

    /// Total pixel count
    pub fn pixel_count(&self) -> u32 {
        self.width * self.height
    }

    /// Total byte size
    pub fn byte_size(&self) -> usize {
        self.data.len()
    }

    /// Check if buffer is fully transparent
    pub fn is_transparent(&self) -> bool {
        self.data.chunks_exact(4).all(|px| px[3] == 0)
    }

    /// Blit another buffer onto this one at (dx, dy)
    pub fn blit(&mut self, src: &PixelBuffer, dx: u32, dy: u32) {
        for sy in 0..src.height {
            let dy_abs = dy + sy;
            if dy_abs >= self.height { break; }
            for sx in 0..src.width {
                let dx_abs = dx + sx;
                if dx_abs >= self.width { break; }
                if let Some([r, g, b, a]) = src.get_pixel(sx, sy) {
                    self.blend_pixel(dx_abs, dy_abs, r, g, b, a);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_buffer_is_transparent() {
        let buf = PixelBuffer::new(100, 100);
        assert_eq!(buf.data.len(), 100 * 100 * 4);
        assert!(buf.is_transparent());
    }

    #[test]
    fn set_and_get_pixel() {
        let mut buf = PixelBuffer::new(10, 10);
        buf.set_pixel(5, 5, 255, 128, 0, 255);
        let px = buf.get_pixel(5, 5).unwrap();
        assert_eq!(px, [255, 128, 0, 255]);
    }

    #[test]
    fn set_pixel_out_of_bounds_no_panic() {
        let mut buf = PixelBuffer::new(10, 10);
        buf.set_pixel(100, 100, 255, 0, 0, 255); // should not panic
    }

    #[test]
    fn get_pixel_out_of_bounds_returns_none() {
        let buf = PixelBuffer::new(10, 10);
        assert!(buf.get_pixel(10, 10).is_none());
    }

    #[test]
    fn fill_rect_correct() {
        let mut buf = PixelBuffer::new(100, 100);
        buf.fill_rect(10, 10, 20, 20, 255, 0, 0, 255);
        assert_eq!(buf.get_pixel(10, 10).unwrap(), [255, 0, 0, 255]);
        assert_eq!(buf.get_pixel(29, 29).unwrap(), [255, 0, 0, 255]);
        assert_eq!(buf.get_pixel(30, 30).unwrap(), [0, 0, 0, 0]);
    }

    #[test]
    fn fill_rect_clips_to_bounds() {
        let mut buf = PixelBuffer::new(50, 50);
        buf.fill_rect(40, 40, 20, 20, 255, 0, 0, 255); // extends beyond bounds
        assert_eq!(buf.get_pixel(49, 49).unwrap(), [255, 0, 0, 255]);
    }

    #[test]
    fn clear_resets_to_transparent() {
        let mut buf = PixelBuffer::new(10, 10);
        buf.fill_rect(0, 0, 10, 10, 255, 0, 0, 255);
        buf.clear();
        assert!(buf.is_transparent());
    }

    #[test]
    fn clear_color_fills_solid() {
        let mut buf = PixelBuffer::new(10, 10);
        buf.clear_color(30, 30, 30, 255);
        assert_eq!(buf.get_pixel(0, 0).unwrap(), [30, 30, 30, 255]);
        assert_eq!(buf.get_pixel(9, 9).unwrap(), [30, 30, 30, 255]);
    }

    #[test]
    fn filled_constructor() {
        let buf = PixelBuffer::filled(10, 10, 0, 255, 0, 255);
        assert_eq!(buf.get_pixel(0, 0).unwrap(), [0, 255, 0, 255]);
        assert!(!buf.is_transparent());
    }

    #[test]
    fn pixel_count_correct() {
        let buf = PixelBuffer::new(800, 600);
        assert_eq!(buf.pixel_count(), 480_000);
    }

    #[test]
    fn byte_size_correct() {
        let buf = PixelBuffer::new(100, 100);
        assert_eq!(buf.byte_size(), 40_000);
    }

    #[test]
    fn blit_copies_pixels() {
        let mut dst = PixelBuffer::new(100, 100);
        let src     = PixelBuffer::filled(10, 10, 255, 0, 0, 255);
        dst.blit(&src, 5, 5);
        assert_eq!(dst.get_pixel(5, 5).unwrap(), [255, 0, 0, 255]);
    }

    #[test]
    fn blit_clips_to_dst_bounds() {
        let mut dst = PixelBuffer::new(10, 10);
        let src     = PixelBuffer::filled(20, 20, 0, 255, 0, 255);
        dst.blit(&src, 5, 5); // src extends beyond dst — should not panic
        assert_eq!(dst.get_pixel(9, 9).unwrap(), [0, 255, 0, 255]);
    }

    #[test]
    fn alpha_blend_opaque_over_transparent() {
        let mut buf = PixelBuffer::new(10, 10);
        buf.blend_pixel(0, 0, 255, 0, 0, 255);
        let px = buf.get_pixel(0, 0).unwrap();
        assert_eq!(px[0], 255);
        assert_eq!(px[3], 255);
    }

    #[test]
    fn stroke_rect_draws_border() {
        let mut buf = PixelBuffer::new(20, 20);
        buf.stroke_rect(2, 2, 10, 10, 255, 255, 255, 255);
        // Top-left corner
        assert_eq!(buf.get_pixel(2, 2).unwrap(), [255, 255, 255, 255]);
        // Interior should be empty
        assert_eq!(buf.get_pixel(5, 5).unwrap(), [0, 0, 0, 0]);
    }
}
