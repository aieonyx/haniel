// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// HANIEL SENTINEL — seL4 Formally Isolated Renderer
// HE-12: PD spec, IPC channels, policy enforcement
// seL4 kernel binding at HE-15 (ASL mKernel integration)

#![forbid(unsafe_code)]

pub mod ipc;
pub mod pd;
pub mod policy;
pub mod renderer;

pub use ipc::{IpcChannel, IpcDirection};
pub use pd::{ProtectionDomain, PdCapabilities, PdId};
pub use policy::{PolicyEnforcer, PolicyOp, AuditEntry};
pub use renderer::{IsolatedRenderer, RenderRequest, RenderResult};

/// Render isolation policy
#[derive(Debug, Clone, PartialEq)]
pub enum RenderPolicy {
    Isolated,   // renderer in separate PD — cannot touch OS
    Sandboxed,  // script in sandbox — limited DOM access
    Trusted,    // sovereign AWP content — full trust
}

/// IPC message types between PDs
#[derive(Debug, Clone)]
pub enum SentinelMessage {
    Ping,
    RenderFrame     { frame_id: u64 },
    RenderComplete  { frame_id: u64, pixel_count: usize },
    DomMutation     { node: u32, mutation_type: String },
    PolicyViolation { pd_id: u32, operation: String },
    Shutdown,
}

/// SENTINEL error type
#[derive(Debug)]
pub enum SentinelError {
    PdAlreadyActive(u32),
    PdNotFound(u32),
    IpcFull,
    PolicyViolation(String),
    SeL4Error(String),
}

/// SENTINEL sovereign isolation trait
pub trait Sentinel: Send + Sync {
    fn create_renderer_pd(&self) -> Result<ProtectionDomain, SentinelError>;
    fn policy_check(&self, pd_id: u32, op: &PolicyOp) -> bool;
    fn audit_log(&self) -> Vec<AuditEntry>;
}

/// Sovereign SENTINEL implementation
pub struct SovereignSentinel {
    enforcer:   PolicyEnforcer,
    audit:      std::sync::Mutex<Vec<AuditEntry>>,
}

impl SovereignSentinel {
    pub fn new() -> Self {
        Self {
            enforcer: PolicyEnforcer::isolated(),
            audit:    std::sync::Mutex::new(Vec::new()),
        }
    }

    /// Create an isolated renderer instance
    pub fn renderer(&self) -> IsolatedRenderer {
        IsolatedRenderer::new()
    }

    /// Log a policy decision
    fn log(&self, op: &PolicyOp, allowed: bool) {
        let entry = self.enforcer.audit(op, allowed);
        self.audit.lock().unwrap().push(entry);
    }
}

impl Default for SovereignSentinel {
    fn default() -> Self { Self::new() }
}

impl Sentinel for SovereignSentinel {
    fn create_renderer_pd(&self) -> Result<ProtectionDomain, SentinelError> {
        let mut pd = ProtectionDomain::renderer();
        pd.activate()?;
        Ok(pd)
    }

    fn policy_check(&self, _pd_id: u32, op: &PolicyOp) -> bool {
        let allowed = self.enforcer.allows(op);
        self.log(op, allowed);
        allowed
    }

    fn audit_log(&self) -> Vec<AuditEntry> {
        self.audit.lock().unwrap().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sentinel() -> SovereignSentinel { SovereignSentinel::new() }

    #[test]
    fn sentinel_constructs() {
        let _ = sentinel();
    }

    #[test]
    fn sentinel_creates_renderer_pd() {
        let s  = sentinel();
        let pd = s.create_renderer_pd().unwrap();
        assert!(pd.active);
        assert_eq!(pd.id, PdId::renderer());
    }

    #[test]
    fn sentinel_policy_allows_render_ops() {
        let s = sentinel();
        assert!(s.policy_check(1, &PolicyOp::ReadLayout));
        assert!(s.policy_check(1, &PolicyOp::WritePixels));
        assert!(s.policy_check(1, &PolicyOp::ReadFont));
    }

    #[test]
    fn sentinel_policy_denies_network() {
        let s = sentinel();
        assert!(!s.policy_check(1, &PolicyOp::NetworkFetch));
    }

    #[test]
    fn sentinel_policy_denies_script() {
        let s = sentinel();
        assert!(!s.policy_check(1, &PolicyOp::ScriptExecute));
    }

    #[test]
    fn sentinel_audit_log_populated() {
        let s = sentinel();
        s.policy_check(1, &PolicyOp::ReadLayout);
        s.policy_check(1, &PolicyOp::NetworkFetch);
        let log = s.audit_log();
        assert_eq!(log.len(), 2);
        assert!(log[0].allowed);
        assert!(!log[1].allowed);
    }

    #[test]
    fn sentinel_renderer_submit_process_poll() {
        let s = sentinel();
        let r = s.renderer();
        r.submit(RenderRequest {
            frame_id: 1, width: 1280, height: 720, dirty_only: false,
        }).unwrap();
        r.process_one();
        let result = r.poll().unwrap();
        assert_eq!(result.frame_id, 1);
        assert!(result.success);
    }

    #[test]
    fn sentinel_message_variants() {
        let _ = SentinelMessage::Ping;
        let _ = SentinelMessage::RenderFrame { frame_id: 1 };
        let _ = SentinelMessage::RenderComplete { frame_id: 1, pixel_count: 100 };
        let _ = SentinelMessage::Shutdown;
    }

    #[test]
    fn render_policy_variants() {
        assert_ne!(RenderPolicy::Isolated, RenderPolicy::Trusted);
        assert_ne!(RenderPolicy::Sandboxed, RenderPolicy::Isolated);
    }
}
