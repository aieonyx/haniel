// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// haniel_lumen::abr — Adaptive Bitrate controller

use crate::stream::{StreamManifest, StreamQuality};

/// Adaptive bitrate controller
/// Selects quality tier based on available bandwidth
pub struct AbrController {
    pub target_buffer_ms: u64,
    pub safety_factor:    f32,
}

impl AbrController {
    pub fn new() -> Self {
        Self { target_buffer_ms: 5000, safety_factor: 0.8 }
    }

    /// Select best quality for available bandwidth
    pub fn select<'a>(&self, manifest: &'a StreamManifest, bandwidth_bps: u64)
        -> Option<&'a StreamQuality>
    {
        let effective = (bandwidth_bps as f32 * self.safety_factor) as u64;
        manifest.qualities.iter()
            .filter(|q| q.bitrate_bps <= effective)
            .max_by_key(|q| q.bitrate_bps)
            .or_else(|| manifest.lowest_quality())
    }
}

impl Default for AbrController {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stream::StreamQuality;
    use crate::StreamProtocol;

    fn manifest() -> StreamManifest {
        let mut m = StreamManifest::new(StreamProtocol::Hls, "stream.m3u8");
        m.add_quality(StreamQuality { bitrate_bps: 500_000,   width: 854,  height: 480,  url: "480p".into(), codecs: None });
        m.add_quality(StreamQuality { bitrate_bps: 1_500_000, width: 1280, height: 720,  url: "720p".into(), codecs: None });
        m.add_quality(StreamQuality { bitrate_bps: 4_000_000, width: 1920, height: 1080, url: "1080p".into(), codecs: None });
        m
    }

    #[test]
    fn abr_selects_highest_fitting() {
        let abr = AbrController::new();
        let m   = manifest();
        let q   = abr.select(&m, 2_000_000).unwrap();
        assert_eq!(q.height, 720);
    }

    #[test]
    fn abr_selects_lowest_when_bandwidth_too_low() {
        let abr = AbrController::new();
        let m   = manifest();
        let q   = abr.select(&m, 100_000).unwrap();
        assert_eq!(q.height, 480);
    }

    #[test]
    fn abr_selects_1080p_with_high_bandwidth() {
        let abr = AbrController::new();
        let m   = manifest();
        let q   = abr.select(&m, 10_000_000).unwrap();
        assert_eq!(q.height, 1080);
    }
}
