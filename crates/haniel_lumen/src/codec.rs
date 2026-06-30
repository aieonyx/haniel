// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// haniel_lumen::codec — Codec detection and negotiation
// HE-14: manifest codec-string parsing (HLS CODECS=, DASH codecs=) added

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

/// Parse a manifest-level codec string (HLS CODECS="..." or DASH codecs="...")
/// into one or more codec tokens. Manifest codec strings are comma-separated
/// RFC 6381 fragments, e.g. "avc1.4d001f,mp4a.40.2" or "av01.0.05M.08,opus".
pub fn parse_codec_string(codecs: &str) -> Vec<CodecToken> {
    codecs
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(classify_codec_token)
        .collect()
}

/// A single classified codec token from a manifest codec string
#[derive(Debug, Clone, PartialEq)]
pub enum CodecToken {
    Video(VideoCodec),
    Audio(AudioCodec),
    Unknown(String),
}

fn classify_codec_token(token: &str) -> CodecToken {
    let lower = token.to_lowercase();

    // Video codec prefixes (RFC 6381 codec fragments)
    if lower.starts_with("av01") || lower.starts_with("av1") {
        return CodecToken::Video(VideoCodec::Av1);
    }
    if lower.starts_with("vp09") || lower.starts_with("vp9") {
        return CodecToken::Video(VideoCodec::Vp9);
    }
    if lower.starts_with("avc1") || lower.starts_with("avc3") || lower.starts_with("h264") {
        return CodecToken::Video(VideoCodec::H264);
    }
    if lower.starts_with("hvc1") || lower.starts_with("hev1") || lower.starts_with("h265") {
        return CodecToken::Video(VideoCodec::Hevc);
    }

    // Audio codec prefixes
    if lower.starts_with("opus") {
        return CodecToken::Audio(AudioCodec::Opus);
    }
    if lower.starts_with("mp4a") || lower.starts_with("aac") {
        return CodecToken::Audio(AudioCodec::Aac);
    }
    if lower.starts_with("flac") || lower.starts_with("fla1") {
        return CodecToken::Audio(AudioCodec::Flac);
    }
    if lower.starts_with("vorbis") {
        return CodecToken::Audio(AudioCodec::Vorbis);
    }

    CodecToken::Unknown(token.to_string())
}

/// Extract the first video codec found in a manifest codec string, if any
pub fn first_video_codec(codecs: &str) -> Option<VideoCodec> {
    parse_codec_string(codecs).into_iter().find_map(|t| match t {
        CodecToken::Video(v) => Some(v),
        _ => None,
    })
}

/// Extract the first audio codec found in a manifest codec string, if any
pub fn first_audio_codec(codecs: &str) -> Option<AudioCodec> {
    parse_codec_string(codecs).into_iter().find_map(|t| match t {
        CodecToken::Audio(a) => Some(a),
        _ => None,
    })
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

    // ── parse_codec_string — manifest CODECS= / codecs= attribute parsing ──

    #[test]
    fn parse_codec_string_avc_and_aac() {
        let tokens = parse_codec_string("avc1.4d001f,mp4a.40.2");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], CodecToken::Video(VideoCodec::H264));
        assert_eq!(tokens[1], CodecToken::Audio(AudioCodec::Aac));
    }

    #[test]
    fn parse_codec_string_av1_and_opus() {
        let tokens = parse_codec_string("av01.0.05M.08,opus");
        assert_eq!(tokens[0], CodecToken::Video(VideoCodec::Av1));
        assert_eq!(tokens[1], CodecToken::Audio(AudioCodec::Opus));
    }

    #[test]
    fn parse_codec_string_vp9() {
        let tokens = parse_codec_string("vp09.00.10.08");
        assert_eq!(tokens[0], CodecToken::Video(VideoCodec::Vp9));
    }

    #[test]
    fn parse_codec_string_hevc() {
        let tokens = parse_codec_string("hvc1.1.6.L93.B0");
        assert_eq!(tokens[0], CodecToken::Video(VideoCodec::Hevc));
    }

    #[test]
    fn parse_codec_string_flac() {
        let tokens = parse_codec_string("flac");
        assert_eq!(tokens[0], CodecToken::Audio(AudioCodec::Flac));
    }

    #[test]
    fn parse_codec_string_vorbis() {
        let tokens = parse_codec_string("vorbis");
        assert_eq!(tokens[0], CodecToken::Audio(AudioCodec::Vorbis));
    }

    #[test]
    fn parse_codec_string_unknown_token() {
        let tokens = parse_codec_string("xyz123");
        assert_eq!(tokens[0], CodecToken::Unknown("xyz123".to_string()));
    }

    #[test]
    fn parse_codec_string_handles_whitespace() {
        let tokens = parse_codec_string("avc1.4d001f, mp4a.40.2");
        assert_eq!(tokens.len(), 2);
    }

    #[test]
    fn parse_codec_string_empty_returns_empty() {
        let tokens = parse_codec_string("");
        assert!(tokens.is_empty());
    }

    #[test]
    fn first_video_codec_finds_av1() {
        assert_eq!(first_video_codec("av01.0.05M.08,opus"), Some(VideoCodec::Av1));
    }

    #[test]
    fn first_video_codec_none_when_audio_only() {
        assert_eq!(first_video_codec("opus"), None);
    }

    #[test]
    fn first_audio_codec_finds_aac() {
        assert_eq!(first_audio_codec("avc1.4d001f,mp4a.40.2"), Some(AudioCodec::Aac));
    }

    #[test]
    fn first_audio_codec_none_when_video_only() {
        assert_eq!(first_audio_codec("avc1.4d001f"), None);
    }
}
