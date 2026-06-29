// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// haniel_sentinel::pd — Protection Domain descriptor
// seL4 PD specification for HANIEL renderer isolation
// Full seL4 binding at HE-15 (SENTINEL integration with ASL mKernel)

use crate::{SentinelError, RenderPolicy};

/// Protection Domain capability flags
#[derive(Debug, Clone, PartialEq)]
pub struct PdCapabilities {
    pub can_read_dom:    bool,   // read layout tree
    pub can_write_frame: bool,   // write pixel buffer
    pub can_access_font: bool,   // access font cache
    pub can_network:     bool,   // network access (NEVER for renderer)
    pub can_exec_script: bool,   // script execution (NEVER for renderer)
}

impl PdCapabilities {
    /// Renderer PD — minimal capabilities
    pub fn renderer() -> Self {
        Self {
            can_read_dom:    true,
            can_write_frame: true,
            can_access_font: true,
            can_network:     false,  // renderer never touches network
            can_exec_script: false,  // renderer never executes scripts
        }
    }

    /// Script PD — ECHO sandbox capabilities
    pub fn script() -> Self {
        Self {
            can_read_dom:    true,
            can_write_frame: false, // scripts cannot write pixels directly
            can_access_font: false,
            can_network:     false, // requires explicit ARPi capability
            can_exec_script: true,
        }
    }

    /// Null PD — no capabilities (default safe state)
    pub fn null() -> Self {
        Self {
            can_read_dom:    false,
            can_write_frame: false,
            can_access_font: false,
            can_network:     false,
            can_exec_script: false,
        }
    }
}

/// seL4 Protection Domain identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PdId(pub u32);

impl PdId {
    pub fn new(id: u32) -> Self { Self(id) }
    pub fn renderer() -> Self { Self(1) }
    pub fn script()   -> Self { Self(2) }
    pub fn network()  -> Self { Self(3) }
    pub fn trusted()  -> Self { Self(0) }
}

/// Protection Domain descriptor
#[derive(Debug, Clone)]
pub struct ProtectionDomain {
    pub id:         PdId,
    pub name:       String,
    pub caps:       PdCapabilities,
    pub policy:     RenderPolicy,
    pub memory_mb:  u32,
    pub active:     bool,
}

impl ProtectionDomain {
    /// Create renderer PD
    pub fn renderer() -> Self {
        Self {
            id:        PdId::renderer(),
            name:      "haniel-renderer".to_string(),
            caps:      PdCapabilities::renderer(),
            policy:    RenderPolicy::Isolated,
            memory_mb: 64,
            active:    false,
        }
    }

    /// Create script sandbox PD
    pub fn script_sandbox() -> Self {
        Self {
            id:        PdId::script(),
            name:      "haniel-echo".to_string(),
            caps:      PdCapabilities::script(),
            policy:    RenderPolicy::Sandboxed,
            memory_mb: 32,
            active:    false,
        }
    }

    /// Activate this PD (seL4 binding at HE-15)
    pub fn activate(&mut self) -> Result<(), SentinelError> {
        if self.active {
            return Err(SentinelError::PdAlreadyActive(self.id.0));
        }
        // seL4 PD creation: sel4::pd_create(self.id.0, self.memory_mb)
        // Stub: mark active
        self.active = true;
        Ok(())
    }

    /// Deactivate this PD
    pub fn deactivate(&mut self) {
        self.active = false;
    }

    /// Check if capability is granted
    pub fn has_cap(&self, cap: &str) -> bool {
        match cap {
            "dom_read"    => self.caps.can_read_dom,
            "frame_write" => self.caps.can_write_frame,
            "font_access" => self.caps.can_access_font,
            "network"     => self.caps.can_network,
            "script_exec" => self.caps.can_exec_script,
            _             => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renderer_pd_caps() {
        let pd = ProtectionDomain::renderer();
        assert!(pd.caps.can_read_dom);
        assert!(pd.caps.can_write_frame);
        assert!(!pd.caps.can_network,   "renderer must not have network");
        assert!(!pd.caps.can_exec_script, "renderer must not exec scripts");
    }

    #[test]
    fn script_pd_caps() {
        let pd = ProtectionDomain::script_sandbox();
        assert!(pd.caps.can_exec_script);
        assert!(!pd.caps.can_write_frame, "script must not write pixels");
        assert!(!pd.caps.can_network);
    }

    #[test]
    fn null_caps_deny_all() {
        let caps = PdCapabilities::null();
        assert!(!caps.can_read_dom);
        assert!(!caps.can_write_frame);
        assert!(!caps.can_network);
        assert!(!caps.can_exec_script);
    }

    #[test]
    fn pd_activate_deactivate() {
        let mut pd = ProtectionDomain::renderer();
        assert!(!pd.active);
        pd.activate().unwrap();
        assert!(pd.active);
        pd.deactivate();
        assert!(!pd.active);
    }

    #[test]
    fn pd_double_activate_errors() {
        let mut pd = ProtectionDomain::renderer();
        pd.activate().unwrap();
        assert!(matches!(pd.activate(), Err(SentinelError::PdAlreadyActive(_))));
    }

    #[test]
    fn pd_has_cap_renderer() {
        let pd = ProtectionDomain::renderer();
        assert!(pd.has_cap("dom_read"));
        assert!(pd.has_cap("frame_write"));
        assert!(!pd.has_cap("network"));
        assert!(!pd.has_cap("script_exec"));
    }

    #[test]
    fn pd_id_constants() {
        assert_eq!(PdId::renderer(), PdId(1));
        assert_eq!(PdId::script(),   PdId(2));
        assert_eq!(PdId::trusted(),  PdId(0));
    }
}
