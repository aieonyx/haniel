// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// haniel_lumen::codec — Codec detection and negotiation
// Full AV1/VP9/H264 decode at HE-14 (open web phase)

use crate::{VideoCodec, AudioCodec};

/// Detect video codec from file extension or MIME type
pub fn detect_video_codec(mime: &str) -> VideoCodec {
    let lower = mime.to_lowercase();
    if lower.contains("av1") || lower.contains("av01") {
        VideoCodec::Av1
    } else if lower.contains("vp9") || lower.contains("vp09") {
        VideoCodec::Vp9
    } else if lower.contains("h264") || lower.contains("avc") {
        VideoCodec::H264
    } else if lower.contains("hevc") || lower.contains("h265") {
        VideoCodec::Hevc
    } else {
        VideoCodec::Av1 // sovereign default
    }
}

/// Detect audio codec from MIME type
pub fn detect_audio_codec(mime: &str) -> AudioCodec {
    let lower = mime.to_lowercase();
    if lower.contains("opus") {
        AudioCodec::Opus
    } else if lower.contains("aac") || lower.contains("mp4a") {
        AudioCodec::Aac
    } else if lower.contains("flac") {
        AudioCodec::Flac
    } else if lower.contains("vorbis") {
        AudioCodec::Vorbis
    } else {
        AudioCodec::Opus // sovereign default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_av1() {
        assert_eq!(detect_video_codec("video/mp4; codecs=av01"), VideoCodec::Av1);
    }

    #[test]
    fn detect_vp9() {
        assert_eq!(detect_video_codec("video/webm; codecs=vp9"), VideoCodec::Vp9);
    }

    #[test]
    fn detect_h264() {
        assert_eq!(detect_video_codec("video/mp4; codecs=avc1"), VideoCodec::H264);
    }

    #[test]
    fn detect_opus() {
        assert_eq!(detect_audio_codec("audio/ogg; codecs=opus"), AudioCodec::Opus);
    }

    #[test]
    fn detect_aac() {
        assert_eq!(detect_audio_codec("audio/mp4; codecs=mp4a.40.2"), AudioCodec::Aac);
    }

    #[test]
    fn default_video_is_av1() {
        assert_eq!(detect_video_codec("video/unknown"), VideoCodec::Av1);
    }

    #[test]
    fn default_audio_is_opus() {
        assert_eq!(detect_audio_codec("audio/unknown"), AudioCodec::Opus);
    }
}
