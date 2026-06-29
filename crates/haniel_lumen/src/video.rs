// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// haniel_lumen::video — Sovereign video frame pipeline
// Wraps axon_media::VideoFrame for HANIEL rendering

use axon_media::{VideoFrame as AxVideoFrame, PixelFormat};
use crate::LumenError;

/// Sovereign video frame — wraps axon_media VideoFrame
pub struct SovereignVideoFrame {
    pub pts:    u64,    // presentation timestamp (ms)
    pub inner:  AxVideoFrame,
}

impl SovereignVideoFrame {
    /// Create a new sovereign video frame
    pub fn new(pts: u64, width: u32, height: u32) -> Result<Self, LumenError> {
        let inner = AxVideoFrame::new(width, height, PixelFormat::Rgb24)
            .map_err(|e| LumenError::DecodeError(format!("{:?}", e)))?;
        Ok(Self { pts, inner })
    }

    /// Create from raw pixel data
    pub fn from_data(
        pts:    u64,
        width:  u32,
        height: u32,
        data:   Vec<u8>,
    ) -> Result<Self, LumenError> {
        let inner = AxVideoFrame::from_data(width, height, PixelFormat::Rgb24, data)
            .map_err(|e| LumenError::DecodeError(format!("{:?}", e)))?;
        Ok(Self { pts, inner })
    }

    /// Fill frame with solid color (for test frames)
    pub fn fill_color(&mut self, r: u8, g: u8, b: u8) -> Result<(), LumenError> {
        self.inner.fill(r, g, b)
            .map_err(|e| LumenError::DecodeError(format!("{:?}", e)))
    }

    /// Get a pixel
    pub fn get_pixel(&self, x: u32, y: u32) -> Result<(u8, u8, u8), LumenError> {
        self.inner.get_pixel_rgb(x, y)
            .map_err(|e| LumenError::DecodeError(format!("{:?}", e)))
    }

    /// Convert to RGBA pixel buffer for CANVAS
    pub fn to_rgba(&self) -> Vec<u8> {
        let size  = (self.inner.width * self.inner.height) as usize;
        let mut rgba = Vec::with_capacity(size * 4);
        for y in 0..self.inner.height {
            for x in 0..self.inner.width {
                if let Ok((r, g, b)) = self.inner.get_pixel_rgb(x, y) {
                    rgba.push(r);
                    rgba.push(g);
                    rgba.push(b);
                    rgba.push(255); // full alpha
                } else {
                    rgba.extend_from_slice(&[0, 0, 0, 255]);
                }
            }
        }
        rgba
    }

    pub fn width(&self)      -> u32 { self.inner.width }
    pub fn height(&self)     -> u32 { self.inner.height }
    pub fn size_bytes(&self) -> usize { self.inner.size_bytes() }
}

/// Video frame buffer — ring buffer for decoded frames
pub struct FrameBuffer {
    frames:   Vec<SovereignVideoFrame>,
    capacity: usize,
}

impl FrameBuffer {
    pub fn new(capacity: usize) -> Self {
        Self { frames: Vec::with_capacity(capacity), capacity }
    }

    pub fn push(&mut self, frame: SovereignVideoFrame) {
        if self.frames.len() >= self.capacity {
            self.frames.remove(0); // drop oldest
        }
        self.frames.push(frame);
    }

    pub fn pop(&mut self) -> Option<SovereignVideoFrame> {
        if self.frames.is_empty() { None } else { Some(self.frames.remove(0)) }
    }

    pub fn len(&self)      -> usize { self.frames.len() }
    pub fn is_empty(&self) -> bool  { self.frames.is_empty() }
    pub fn is_full(&self)  -> bool  { self.frames.len() >= self.capacity }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn video_frame_constructs() {
        let f = SovereignVideoFrame::new(0, 320, 240).unwrap();
        assert_eq!(f.width(),  320);
        assert_eq!(f.height(), 240);
        assert_eq!(f.pts,      0);
    }

    #[test]
    fn video_frame_fill_color() {
        let mut f = SovereignVideoFrame::new(0, 10, 10).unwrap();
        f.fill_color(255, 0, 0).unwrap();
        let (r, g, b) = f.get_pixel(0, 0).unwrap();
        assert_eq!(r, 255);
        assert_eq!(g, 0);
        assert_eq!(b, 0);
    }

    #[test]
    fn video_frame_to_rgba() {
        let mut f = SovereignVideoFrame::new(100, 2, 2).unwrap();
        f.fill_color(0, 255, 0).unwrap(); // green
        let rgba = f.to_rgba();
        assert_eq!(rgba.len(), 2 * 2 * 4);
        assert_eq!(rgba[1], 255); // G channel
        assert_eq!(rgba[3], 255); // alpha = 255
    }

    #[test]
    fn video_frame_size_bytes() {
        let f = SovereignVideoFrame::new(0, 320, 240).unwrap();
        assert_eq!(f.size_bytes(), 320 * 240 * 3); // RGB24
    }

    #[test]
    fn frame_buffer_push_pop() {
        let mut buf = FrameBuffer::new(3);
        buf.push(SovereignVideoFrame::new(0, 10, 10).unwrap());
        buf.push(SovereignVideoFrame::new(33, 10, 10).unwrap());
        assert_eq!(buf.len(), 2);
        let f = buf.pop().unwrap();
        assert_eq!(f.pts, 0);
        assert_eq!(buf.len(), 1);
    }

    #[test]
    fn frame_buffer_evicts_oldest_when_full() {
        let mut buf = FrameBuffer::new(2);
        buf.push(SovereignVideoFrame::new(0, 10, 10).unwrap());
        buf.push(SovereignVideoFrame::new(33, 10, 10).unwrap());
        buf.push(SovereignVideoFrame::new(66, 10, 10).unwrap()); // evicts pts=0
        assert_eq!(buf.len(), 2);
        let f = buf.pop().unwrap();
        assert_eq!(f.pts, 33); // oldest remaining
    }

    #[test]
    fn frame_buffer_empty_pop_none() {
        let mut buf = FrameBuffer::new(4);
        assert!(buf.pop().is_none());
        assert!(buf.is_empty());
    }
}
