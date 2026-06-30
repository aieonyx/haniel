// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// HANIEL LUMEN — Sovereign Media Engine
// HE-10: axon_media wired — video frames, PCM audio, DRM sovereignty
// HE-14: HLS/DASH manifest + segment-list parsing, AV1/codec-string negotiation

#![forbid(unsafe_code)]

pub mod abr;
pub mod audio;
pub mod codec;
pub mod demux;
pub mod drm;
pub mod session;
pub mod stream;
pub mod video;

pub use audio::AudioPipeline;
pub use codec::{detect_video_codec, detect_audio_codec, parse_codec_string, first_video_codec, first_audio_codec, CodecToken};
pub use demux::{Segment, SegmentList, SegmentParseError};
pub use drm::{DrmResolver, DrmVerdict, SovereigntyNotice};
pub use session::{MediaSession, MediaStats, SessionState, SessionIdGen};
pub use stream::{StreamManifest, StreamQuality, ManifestParseError};
pub use abr::AbrController;
pub use video::{SovereignVideoFrame, FrameBuffer};

/// Media source descriptor
#[derive(Debug, Clone)]
pub enum MediaSource {
    Url(String),
    AwpStream(String),
    LocalFile(String),
    AdaptiveStream { manifest: String, protocol: StreamProtocol },
}

/// Adaptive stream protocol
#[derive(Debug, Clone, PartialEq)]
pub enum StreamProtocol {
    Hls,
    Dash,
    Rtmp,
    Awp,
}

/// Video codec
#[derive(Debug, Clone, PartialEq)]
pub enum VideoCodec {
    Av1,
    Vp9,
    H264,
    Hevc,
}

/// Audio codec
#[derive(Debug, Clone, PartialEq)]
pub enum AudioCodec {
    Opus,
    Aac,
    Flac,
    Vorbis,
}

/// DRM mode
#[derive(Debug, Clone, PartialEq)]
pub enum DrmMode {
    None,
    SovereignKey(Vec<u8>),
    CompatibilityShim,
    Blocked,
}

/// LUMEN error type
#[derive(Debug)]
pub enum LumenError {
    UnsupportedCodec(String),
    DrmBlocked(String),
    NetworkError(String),
    DecodeError(String),
    EndOfStream,
}

/// LUMEN sovereign media trait
pub trait Lumen: Send + Sync {
    fn open(&self, source: MediaSource) -> Result<MediaSession, LumenError>;
    fn stats(&self, session_id: u64) -> MediaStats;
}

/// Sovereign LUMEN implementation
pub struct SovereignLumen {
    drm:  DrmResolver,
    abr:  AbrController,
    id_gen: std::sync::Mutex<SessionIdGen>,
}

impl SovereignLumen {
    pub fn new() -> Self {
        Self {
            drm:    DrmResolver::new(),
            abr:    AbrController::new(),
            id_gen: std::sync::Mutex::new(SessionIdGen::new()),
        }
    }

    /// Check if a source can be played
    pub fn can_play(&self, drm: &DrmMode) -> bool {
        self.drm.can_play(drm)
    }

    /// Get sovereignty notice for DRM content
    pub fn sovereignty_notice(&self, drm: &DrmMode) -> Option<SovereigntyNotice> {
        self.drm.notice(drm)
    }

    /// Select stream quality for bandwidth
    pub fn select_quality<'a>(
        &self,
        manifest:      &'a StreamManifest,
        bandwidth_bps: u64,
    ) -> Option<&'a StreamQuality> {
        self.abr.select(manifest, bandwidth_bps)
    }
}

impl Default for SovereignLumen {
    fn default() -> Self { Self::new() }
}

impl Lumen for SovereignLumen {
    fn open(&self, source: MediaSource) -> Result<MediaSession, LumenError> {
        let id = self.id_gen.lock().unwrap().generate();
        Ok(MediaSession::new(id, source))
    }

    fn stats(&self, _session_id: u64) -> MediaStats {
        MediaStats::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lumen() -> SovereignLumen { SovereignLumen::new() }

    #[test]
    fn lumen_open_url_session() {
        let l = lumen();
        let s = l.open(MediaSource::Url("https://example.com/video.mp4".into())).unwrap();
        assert_eq!(s.id, 1);
    }

    #[test]
    fn lumen_open_awp_stream() {
        let l = lumen();
        let s = l.open(MediaSource::AwpStream("awp://media/test".into())).unwrap();
        assert_eq!(s.id, 1);
    }

    #[test]
    fn lumen_session_ids_increment() {
        let l  = lumen();
        let s1 = l.open(MediaSource::Url("a".into())).unwrap();
        let s2 = l.open(MediaSource::Url("b".into())).unwrap();
        assert_eq!(s1.id, 1);
        assert_eq!(s2.id, 2);
    }

    #[test]
    fn lumen_open_content_can_play() {
        let l = lumen();
        assert!(l.can_play(&DrmMode::None));
    }

    #[test]
    fn lumen_drm_content_blocked() {
        let l = lumen();
        assert!(!l.can_play(&DrmMode::CompatibilityShim));
    }

    #[test]
    fn lumen_sovereignty_notice_for_drm() {
        let l      = lumen();
        let notice = l.sovereignty_notice(&DrmMode::CompatibilityShim);
        assert!(notice.is_some());
    }

    #[test]
    fn lumen_no_notice_for_open_content() {
        let l = lumen();
        assert!(l.sovereignty_notice(&DrmMode::None).is_none());
    }

    #[test]
    fn lumen_abr_selects_quality() {
        let l = lumen();
        let mut m = StreamManifest::new(StreamProtocol::Hls, "stream.m3u8");
        m.add_quality(StreamQuality { bitrate_bps: 500_000, width: 854, height: 480, url: "480p".into(), codecs: None });
        m.add_quality(StreamQuality { bitrate_bps: 2_000_000, width: 1280, height: 720, url: "720p".into(), codecs: None });
        let q = l.select_quality(&m, 1_000_000).unwrap();
        assert_eq!(q.height, 480);
    }

    #[test]
    fn video_codec_variants_exist() {
        let _ = VideoCodec::Av1;
        let _ = VideoCodec::Vp9;
        let _ = VideoCodec::H264;
        let _ = VideoCodec::Hevc;
    }

    #[test]
    fn audio_codec_variants_exist() {
        let _ = AudioCodec::Opus;
        let _ = AudioCodec::Aac;
        let _ = AudioCodec::Flac;
        let _ = AudioCodec::Vorbis;
    }
}
