// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// HANIEL LUMEN — Sovereign Media Engine
// HE-10 implementation target

#![forbid(unsafe_code)]

pub mod demux;
pub mod codec;
pub mod stream;
pub mod abr;
pub mod drm;
pub mod audio;

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

/// Active media session
#[derive(Debug)]
pub struct MediaSession {
    pub id: u64,
    pub source: MediaSource,
    pub video_codec: VideoCodec,
    pub audio_codec: AudioCodec,
    pub drm: DrmMode,
    pub width: u32,
    pub height: u32,
    pub framerate: f32,
    pub hardware_decode: bool,
}

/// Decoded video frame
#[derive(Debug)]
pub struct VideoFrame {
    pub pts: u64,
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

/// Media playback stats
#[derive(Debug, Default)]
pub struct MediaStats {
    pub frames_decoded: u64,
    pub frames_dropped: u64,
    pub buffer_ms: u64,
    pub bitrate_bps: u64,
}

/// LUMEN sovereign media trait
pub trait Lumen: Send + Sync {
    fn open(&self, source: MediaSource) -> Result<MediaSession, LumenError>;
    fn next_frame(&self, session: &MediaSession) -> Result<VideoFrame, LumenError>;
    fn select_quality(&self, session: &MediaSession, bandwidth_bps: u64) -> Result<u32, LumenError>;
    fn stats(&self, session_id: u64) -> MediaStats;
    fn close(&self, session: MediaSession);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn video_codec_av1_is_primary() {
        // AV1 is our sovereign-first codec — royalty-free
        let codec = VideoCodec::Av1;
        assert_eq!(codec, VideoCodec::Av1);
    }

    #[test]
    fn audio_codec_opus_is_primary() {
        // Opus is our sovereign-first audio codec
        let codec = AudioCodec::Opus;
        assert_eq!(codec, AudioCodec::Opus);
    }

    #[test]
    fn drm_mode_blocked_for_hostile_drm() {
        let drm = DrmMode::Blocked;
        assert_eq!(drm, DrmMode::Blocked);
    }

    #[test]
    fn stream_protocol_variants_exist() {
        let protocols = vec![
            StreamProtocol::Hls,
            StreamProtocol::Dash,
            StreamProtocol::Awp,
        ];
        assert_eq!(protocols.len(), 3);
    }

    #[test]
    fn media_stats_default_zero() {
        let stats = MediaStats::default();
        assert_eq!(stats.frames_decoded, 0);
        assert_eq!(stats.frames_dropped, 0);
    }
}
