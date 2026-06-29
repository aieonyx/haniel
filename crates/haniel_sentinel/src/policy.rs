// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// haniel_sentinel::policy — Render isolation policy

use crate::RenderPolicy;

/// Policy enforcer — validates cross-PD operations
pub struct PolicyEnforcer {
    pub policy: RenderPolicy,
}

impl PolicyEnforcer {
    pub fn new(policy: RenderPolicy) -> Self { Self { policy } }

    pub fn isolated()  -> Self { Self::new(RenderPolicy::Isolated) }
    pub fn sandboxed() -> Self { Self::new(RenderPolicy::Sandboxed) }
    pub fn trusted()   -> Self { Self::new(RenderPolicy::Trusted) }

    /// Check if a cross-PD operation is allowed
    pub fn allows(&self, operation: &PolicyOp) -> bool {
        match &self.policy {
            RenderPolicy::Trusted => true, // trusted allows all
            RenderPolicy::Isolated => {
                matches!(operation,
                    PolicyOp::ReadLayout
                    | PolicyOp::WritePixels
                    | PolicyOp::ReadFont
                    | PolicyOp::SendRenderComplete
                )
            }
            RenderPolicy::Sandboxed => {
                matches!(operation,
                    PolicyOp::ReadLayout
                    | PolicyOp::SendDomMutation
                )
            }
        }
    }

    /// Generate audit log entry for a policy decision
    pub fn audit(&self, op: &PolicyOp, allowed: bool) -> AuditEntry {
        AuditEntry {
            operation: format!("{:?}", op),
            allowed,
            policy:    format!("{:?}", self.policy),
        }
    }
}

/// Policy operation types
#[derive(Debug, Clone, PartialEq)]
pub enum PolicyOp {
    ReadLayout,
    WritePixels,
    ReadFont,
    NetworkFetch,
    ScriptExecute,
    StorageRead,
    StorageWrite,
    SendRenderComplete,
    SendDomMutation,
}

/// Audit log entry
#[derive(Debug, Clone)]
pub struct AuditEntry {
    pub operation: String,
    pub allowed:   bool,
    pub policy:    String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn isolated_allows_render_ops() {
        let e = PolicyEnforcer::isolated();
        assert!(e.allows(&PolicyOp::ReadLayout));
        assert!(e.allows(&PolicyOp::WritePixels));
        assert!(e.allows(&PolicyOp::ReadFont));
    }

    #[test]
    fn isolated_denies_network() {
        let e = PolicyEnforcer::isolated();
        assert!(!e.allows(&PolicyOp::NetworkFetch));
    }

    #[test]
    fn isolated_denies_script() {
        let e = PolicyEnforcer::isolated();
        assert!(!e.allows(&PolicyOp::ScriptExecute));
    }

    #[test]
    fn sandboxed_allows_dom_read() {
        let e = PolicyEnforcer::sandboxed();
        assert!(e.allows(&PolicyOp::ReadLayout));
        assert!(e.allows(&PolicyOp::SendDomMutation));
    }

    #[test]
    fn sandboxed_denies_pixel_write() {
        let e = PolicyEnforcer::sandboxed();
        assert!(!e.allows(&PolicyOp::WritePixels));
    }

    #[test]
    fn trusted_allows_all() {
        let e = PolicyEnforcer::trusted();
        for op in [
            PolicyOp::ReadLayout, PolicyOp::WritePixels,
            PolicyOp::NetworkFetch, PolicyOp::ScriptExecute,
            PolicyOp::StorageRead, PolicyOp::StorageWrite,
        ] {
            assert!(e.allows(&op), "trusted should allow {:?}", op);
        }
    }

    #[test]
    fn audit_entry_generated() {
        let e     = PolicyEnforcer::isolated();
        let entry = e.audit(&PolicyOp::NetworkFetch, false);
        assert!(!entry.allowed);
        assert!(entry.operation.contains("NetworkFetch"));
    }
}
