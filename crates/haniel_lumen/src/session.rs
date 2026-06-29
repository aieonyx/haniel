// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// haniel_lumen::session — Sovereign media session management

use crate::{MediaSource, VideoCodec, AudioCodec, DrmMode};

/// Active media session
#[derive(Debug)]
pub struct MediaSession {
    pub id:              u64,
    pub source:          MediaSource,
    pub video_codec:     VideoCodec,
    pub audio_codec:     AudioCodec,
    pub drm:             DrmMode,
    pub width:           u32,
    pub height:          u32,
    pub framerate:       f32,
    pub hardware_decode: bool,
    pub state:           SessionState,
}

/// Session playback state
#[derive(Debug, Clone, PartialEq)]
pub enum SessionState {
    Idle,
    Playing,
    Paused,
    Ended,
    Error(String),
}

impl MediaSession {
    pub fn new(id: u64, source: MediaSource) -> Self {
        Self {
            id,
            source,
            video_codec:     VideoCodec::Av1,
            audio_codec:     AudioCodec::Opus,
            drm:             DrmMode::None,
            width:           1280,
            height:          720,
            framerate:       30.0,
            hardware_decode: false,
            state:           SessionState::Idle,
        }
    }

    pub fn is_playing(&self) -> bool {
        self.state == SessionState::Playing
    }

    pub fn play(&mut self) {
        if self.state == SessionState::Idle || self.state == SessionState::Paused {
            self.state = SessionState::Playing;
        }
    }

    pub fn pause(&mut self) {
        if self.state == SessionState::Playing {
            self.state = SessionState::Paused;
        }
    }

    pub fn end(&mut self) {
        self.state = SessionState::Ended;
    }

    pub fn with_drm(mut self, drm: DrmMode) -> Self {
        self.drm = drm;
        self
    }

    pub fn with_resolution(mut self, w: u32, h: u32) -> Self {
        self.width  = w;
        self.height = h;
        self
    }

    pub fn with_framerate(mut self, fps: f32) -> Self {
        self.framerate = fps;
        self
    }
}

/// Media playback statistics
#[derive(Debug, Default, Clone)]
pub struct MediaStats {
    pub frames_decoded: u64,
    pub frames_dropped: u64,
    pub buffer_ms:      u64,
    pub bitrate_bps:    u64,
    pub audio_ms:       u64,
}

impl MediaStats {
    pub fn new() -> Self { Self::default() }

    pub fn record_frame(&mut self) {
        self.frames_decoded += 1;
    }

    pub fn drop_frame(&mut self) {
        self.frames_dropped += 1;
    }

    pub fn drop_ratio(&self) -> f32 {
        let total = self.frames_decoded + self.frames_dropped;
        if total == 0 { 0.0 } else { self.frames_dropped as f32 / total as f32 }
    }
}

/// Session ID generator
pub struct SessionIdGen {
    counter: u64,
}

impl SessionIdGen {
    pub fn new() -> Self { Self { counter: 1 } }
    pub fn generate(&mut self) -> u64 {
        let id = self.counter;
        self.counter += 1;
        id
    }
}

impl Default for SessionIdGen {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MediaSource;

    fn session() -> MediaSession {
        MediaSession::new(1, MediaSource::Url("awp://test".into()))
    }

    #[test]
    fn session_starts_idle() {
        let s = session();
        assert_eq!(s.state, SessionState::Idle);
        assert!(!s.is_playing());
    }

    #[test]
    fn session_play_transitions() {
        let mut s = session();
        s.play();
        assert_eq!(s.state, SessionState::Playing);
        assert!(s.is_playing());
    }

    #[test]
    fn session_pause_transitions() {
        let mut s = session();
        s.play();
        s.pause();
        assert_eq!(s.state, SessionState::Paused);
        assert!(!s.is_playing());
    }

    #[test]
    fn session_end_transitions() {
        let mut s = session();
        s.play();
        s.end();
        assert_eq!(s.state, SessionState::Ended);
    }

    #[test]
    fn session_builder_methods() {
        let s = MediaSession::new(1, MediaSource::Url("test".into()))
            .with_resolution(1920, 1080)
            .with_framerate(60.0)
            .with_drm(DrmMode::None);
        assert_eq!(s.width,     1920);
        assert_eq!(s.height,    1080);
        assert_eq!(s.framerate, 60.0);
    }

    #[test]
    fn stats_record_frames() {
        let mut stats = MediaStats::new();
        stats.record_frame();
        stats.record_frame();
        stats.drop_frame();
        assert_eq!(stats.frames_decoded, 2);
        assert_eq!(stats.frames_dropped, 1);
    }

    #[test]
    fn stats_drop_ratio() {
        let mut stats = MediaStats::new();
        stats.record_frame();
        stats.record_frame();
        stats.drop_frame();
        let ratio = stats.drop_ratio();
        assert!((ratio - 1.0/3.0).abs() < 0.01);
    }

    #[test]
    fn stats_drop_ratio_zero_when_no_drops() {
        let mut stats = MediaStats::new();
        stats.record_frame();
        assert_eq!(stats.drop_ratio(), 0.0);
    }

    #[test]
    fn session_id_gen_increments() {
        let mut gen = SessionIdGen::new();
        assert_eq!(gen.generate(), 1);
        assert_eq!(gen.generate(), 2);
        assert_eq!(gen.generate(), 3);
    }
}
