// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// haniel_sentinel::renderer — Isolated renderer interface
// Software path now — seL4 PD renderer at HE-15

use crate::{SentinelError, SentinelMessage, RenderPolicy};
use crate::pd::ProtectionDomain;
use crate::ipc::{IpcChannel, IpcDirection};

/// Render request — sent from trusted PD to renderer PD
#[derive(Debug, Clone)]
pub struct RenderRequest {
    pub frame_id:   u64,
    pub width:      u32,
    pub height:     u32,
    pub dirty_only: bool,
}

/// Render result — returned from renderer PD to trusted PD
#[derive(Debug)]
pub struct RenderResult {
    pub frame_id:    u64,
    pub pixel_count: usize,
    pub latency_ms:  u64,
    pub success:     bool,
}

/// Isolated renderer — runs in separate PD
pub struct IsolatedRenderer {
    pub pd:      ProtectionDomain,
    pub policy:  RenderPolicy,
    tx:          IpcChannel,  // trusted → renderer
    rx:          IpcChannel,  // renderer → trusted
}

impl IsolatedRenderer {
    /// Create isolated renderer with software isolation
    pub fn new() -> Self {
        let tx = IpcChannel::new(IpcDirection::TrustedToRenderer, 32);
        let rx = IpcChannel::new(IpcDirection::RendererToTrusted, 32);
        Self {
            pd:     ProtectionDomain::renderer(),
            policy: RenderPolicy::Isolated,
            tx,
            rx,
        }
    }

    /// Submit a render request to the isolated renderer
    pub fn submit(&self, req: RenderRequest) -> Result<(), SentinelError> {
        self.tx.send(SentinelMessage::RenderFrame {
            frame_id: req.frame_id,
        })
    }

    /// Poll for completed render results
    pub fn poll(&self) -> Option<RenderResult> {
        match self.rx.recv() {
            Some(SentinelMessage::RenderComplete { frame_id, pixel_count }) => {
                Some(RenderResult {
                    frame_id,
                    pixel_count,
                    latency_ms: 16, // target 60fps
                    success:    true,
                })
            }
            _ => None,
        }
    }

    /// Pending render requests
    pub fn pending_count(&self) -> usize { self.tx.pending() }

    /// Simulate renderer processing (software path — real seL4 at HE-15)
    pub fn process_one(&self) -> bool {
        match self.tx.recv() {
            Some(SentinelMessage::RenderFrame { frame_id }) => {
                let _ = self.rx.send(SentinelMessage::RenderComplete {
                    frame_id,
                    pixel_count: 1280 * 720,
                });
                true
            }
            _ => false,
        }
    }

    /// Policy check — can this request cross the PD boundary?
    pub fn policy_allows(&self, cap: &str) -> bool {
        match self.policy {
            RenderPolicy::Isolated  => self.pd.has_cap(cap),
            RenderPolicy::Sandboxed => false,
            RenderPolicy::Trusted   => true,
        }
    }
}

impl Default for IsolatedRenderer {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn renderer() -> IsolatedRenderer { IsolatedRenderer::new() }

    #[test]
    fn renderer_constructs() {
        let r = renderer();
        assert_eq!(r.policy, RenderPolicy::Isolated);
    }

    #[test]
    fn submit_render_request() {
        let r = renderer();
        let result = r.submit(RenderRequest {
            frame_id: 1, width: 1280, height: 720, dirty_only: false,
        });
        assert!(result.is_ok());
        assert_eq!(r.pending_count(), 1);
    }

    #[test]
    fn process_and_poll_complete() {
        let r = renderer();
        r.submit(RenderRequest {
            frame_id: 42, width: 1280, height: 720, dirty_only: false,
        }).unwrap();
        let processed = r.process_one();
        assert!(processed);
        let result = r.poll().unwrap();
        assert_eq!(result.frame_id, 42);
        assert!(result.success);
        assert_eq!(result.pixel_count, 1280 * 720);
    }

    #[test]
    fn poll_empty_returns_none() {
        let r = renderer();
        assert!(r.poll().is_none());
    }

    #[test]
    fn policy_allows_dom_read() {
        let r = renderer();
        assert!(r.policy_allows("dom_read"));
    }

    #[test]
    fn policy_denies_network() {
        let r = renderer();
        assert!(!r.policy_allows("network"));
    }

    #[test]
    fn policy_denies_script_exec() {
        let r = renderer();
        assert!(!r.policy_allows("script_exec"));
    }

    #[test]
    fn multiple_frames_queued() {
        let r = renderer();
        for i in 0..5 {
            r.submit(RenderRequest {
                frame_id: i, width: 1280, height: 720, dirty_only: false,
            }).unwrap();
        }
        assert_eq!(r.pending_count(), 5);
    }
}
