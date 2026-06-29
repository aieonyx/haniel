// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// haniel_lumen::audio — Sovereign audio pipeline
// Wraps axon_media PCM + RTP for HANIEL audio

use axon_media::{PcmConfig, PcmFrame, RtpSession, RtpPacket};
use crate::LumenError;

/// Sovereign audio pipeline
pub struct AudioPipeline {
    config:  PcmConfig,
    buffer:  Vec<PcmFrame>,
    session: RtpSession,
}

impl AudioPipeline {
    /// Create stereo 44kHz audio pipeline (web standard)
    pub fn stereo_44() -> Self {
        Self {
            config:  PcmConfig::stereo_44(),
            buffer:  Vec::new(),
            session: RtpSession::new(0x1234, axon_media::PT_L16_MONO),
        }
    }

    /// Create mono 48kHz audio pipeline (voice/VOIP)
    pub fn mono_48() -> Self {
        Self {
            config:  PcmConfig::mono_48(),
            buffer:  Vec::new(),
            session: RtpSession::new(0x5678, axon_media::PT_PCMU),
        }
    }

    /// Create a silence frame
    pub fn silence(&self, samples: usize) -> Result<PcmFrame, LumenError> {
        PcmFrame::silence(self.config.clone(), samples)
            .map_err(|e| LumenError::DecodeError(format!("audio: {:?}", e)))
    }

    /// Create an audio frame from raw samples
    pub fn frame_from_samples(&self, samples: Vec<i16>)
        -> Result<PcmFrame, LumenError>
    {
        PcmFrame::new(self.config.clone(), samples)
            .map_err(|e| LumenError::DecodeError(format!("audio: {:?}", e)))
    }

    /// Push a decoded audio frame to buffer
    pub fn push(&mut self, frame: PcmFrame) {
        self.buffer.push(frame);
    }

    /// Pop next audio frame for playback
    pub fn pop(&mut self) -> Option<PcmFrame> {
        if self.buffer.is_empty() { None } else { Some(self.buffer.remove(0)) }
    }

    /// Mix two audio frames together
    pub fn mix(&self, a: &PcmFrame, b: &PcmFrame) -> Result<PcmFrame, LumenError> {
        a.mix(b).map_err(|e| LumenError::DecodeError(format!("mix: {:?}", e)))
    }

    /// Apply volume scaling to a frame
    pub fn scale_volume(&self, frame: &PcmFrame, factor: f32) -> PcmFrame {
        frame.amplitude_scale(factor)
    }

    /// Package audio frame as RTP packet for streaming
    pub fn to_rtp(&mut self, frame: &PcmFrame, ts_increment: u32) -> RtpPacket {
        let payload = frame.to_bytes();
        self.session.next_packet(payload, ts_increment)
    }

    /// Duration of buffered audio in milliseconds
    pub fn buffered_ms(&self) -> u64 {
        self.buffer.iter().map(|f| f.duration_ms()).sum()
    }

    /// Number of buffered frames
    pub fn buffer_len(&self) -> usize { self.buffer.len() }

    /// Is buffer empty
    pub fn is_empty(&self) -> bool { self.buffer.is_empty() }

    /// Sample rate
    pub fn sample_rate(&self) -> u32 { self.config.sample_rate }

    /// Channel count
    pub fn channels(&self) -> u8 { self.config.channels }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pipeline() -> AudioPipeline { AudioPipeline::stereo_44() }

    use axon_media::{SAMPLE_RATE_44KHZ, SAMPLE_RATE_48KHZ};

    #[test]
    fn audio_pipeline_stereo_44_config() {
        let p = pipeline();
        assert_eq!(p.sample_rate(), SAMPLE_RATE_44KHZ);
        assert_eq!(p.channels(),    2);
    }

    #[test]
    fn audio_pipeline_mono_48_config() {
        let p = AudioPipeline::mono_48();
        assert_eq!(p.sample_rate(), SAMPLE_RATE_48KHZ);
        assert_eq!(p.channels(),    1);
    }

    #[test]
    fn silence_frame_constructs() {
        let p     = pipeline();
        let frame = p.silence(1024).unwrap();
        // stereo: duration = samples / (rate * channels)
        assert_eq!(frame.duration_ms(), 1024 * 1000 / (SAMPLE_RATE_44KHZ as u64 * 2));
    }

    #[test]
    fn frame_from_samples() {
        let p       = pipeline();
        let samples = vec![0i16; 1024];
        let frame   = p.frame_from_samples(samples).unwrap();
        assert!(frame.duration_ms() > 0);
    }

    #[test]
    fn push_and_pop_frame() {
        let mut p = pipeline();
        let frame = p.silence(512).unwrap();
        p.push(frame);
        assert_eq!(p.buffer_len(), 1);
        let popped = p.pop();
        assert!(popped.is_some());
        assert!(p.is_empty());
    }

    #[test]
    fn buffered_ms_accumulates() {
        let mut p = pipeline();
        let f1    = p.silence(512).unwrap();
        let f2    = p.silence(512).unwrap();
        p.push(f1);
        p.push(f2);
        assert!(p.buffered_ms() > 0);
    }

    #[test]
    fn mix_two_frames() {
        let p  = pipeline();
        let a  = p.silence(256).unwrap();
        let b  = p.silence(256).unwrap();
        let ab = p.mix(&a, &b).unwrap();
        assert_eq!(ab.duration_ms(), a.duration_ms());
    }

    #[test]
    fn scale_volume() {
        let p     = pipeline();
        let frame = p.silence(256).unwrap();
        let _loud = p.scale_volume(&frame, 2.0);
        // Should not panic
    }

    #[test]
    fn rtp_packet_from_frame() {
        let mut p   = pipeline();
        let frame   = p.silence(160).unwrap();
        let packet  = p.to_rtp(&frame, 160);
        assert!(packet.total_bytes() > 12); // > RTP header size
    }

    #[test]
    fn empty_pipeline_pop_none() {
        let mut p = pipeline();
        assert!(p.pop().is_none());
    }
}
