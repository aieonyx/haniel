// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// haniel_lumen::stream — HLS/DASH stream handling (HE-14 full impl)

use crate::StreamProtocol;

/// Stream manifest — HLS or DASH
#[derive(Debug, Clone)]
pub struct StreamManifest {
    pub protocol:  StreamProtocol,
    pub url:       String,
    pub qualities: Vec<StreamQuality>,
}

/// Quality level in adaptive stream
#[derive(Debug, Clone)]
pub struct StreamQuality {
    pub bitrate_bps: u64,
    pub width:       u32,
    pub height:      u32,
    pub url:         String,
}

impl StreamManifest {
    pub fn new(protocol: StreamProtocol, url: &str) -> Self {
        Self { protocol, url: url.to_string(), qualities: Vec::new() }
    }

    pub fn add_quality(&mut self, q: StreamQuality) {
        self.qualities.push(q);
    }

    pub fn best_quality(&self) -> Option<&StreamQuality> {
        self.qualities.iter().max_by_key(|q| q.bitrate_bps)
    }

    pub fn lowest_quality(&self) -> Option<&StreamQuality> {
        self.qualities.iter().min_by_key(|q| q.bitrate_bps)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_add_quality() {
        let mut m = StreamManifest::new(StreamProtocol::Hls, "https://example.com/stream.m3u8");
        m.add_quality(StreamQuality { bitrate_bps: 1_000_000, width: 1280, height: 720, url: "720p.m3u8".into() });
        m.add_quality(StreamQuality { bitrate_bps: 3_000_000, width: 1920, height: 1080, url: "1080p.m3u8".into() });
        assert_eq!(m.qualities.len(), 2);
    }

    #[test]
    fn manifest_best_quality() {
        let mut m = StreamManifest::new(StreamProtocol::Dash, "stream.mpd");
        m.add_quality(StreamQuality { bitrate_bps: 500_000,   width: 854,  height: 480,  url: "480p".into() });
        m.add_quality(StreamQuality { bitrate_bps: 2_000_000, width: 1920, height: 1080, url: "1080p".into() });
        let best = m.best_quality().unwrap();
        assert_eq!(best.height, 1080);
    }

    #[test]
    fn manifest_lowest_quality() {
        let mut m = StreamManifest::new(StreamProtocol::Hls, "stream.m3u8");
        m.add_quality(StreamQuality { bitrate_bps: 500_000,   width: 854,  height: 480,  url: "480p".into() });
        m.add_quality(StreamQuality { bitrate_bps: 2_000_000, width: 1920, height: 1080, url: "1080p".into() });
        let lowest = m.lowest_quality().unwrap();
        assert_eq!(lowest.height, 480);
    }
}
