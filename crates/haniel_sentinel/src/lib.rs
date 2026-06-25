// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// HANIEL SENTINEL — seL4 Formally Isolated Renderer
// HE-12 implementation target

#![forbid(unsafe_code)]

pub mod pd;
pub mod caps;
pub mod audit;
pub mod recovery;

/// seL4 PD configuration
#[derive(Debug, Clone)]
pub struct PdConfig {
    pub name: String,
    pub memory_pages: u32,
    pub caps: Vec<SentinelCap>,
}

/// Capability grants for renderer PD
#[derive(Debug, Clone, PartialEq)]
pub enum SentinelCap {
    DisplaySurface,
    FontRead,
    NetworkNone,
    EdisonDbNoise,
}

/// PD handle (opaque)
#[derive(Debug)]
pub struct PdHandle {
    pub name: String,
    pub pid: u32,
}

/// Isolation report
#[derive(Debug)]
pub struct IsolationReport {
    pub pd_name: String,
    pub memory_used: u32,
    pub caps_active: Vec<SentinelCap>,
    pub violations: Vec<IsolationViolation>,
}

/// Isolation violation
#[derive(Debug, Clone)]
pub enum IsolationViolation {
    UnauthorizedCap(SentinelCap),
    MemoryOverflow { requested: u32, limit: u32 },
    NetworkAttempt { uri: String },
}

/// SENTINEL isolation trait
pub trait Sentinel: Send + Sync {
    fn spawn_renderer(&self, config: PdConfig) -> Result<PdHandle, SentinelError>;
    fn report(&self, pd: &PdHandle) -> IsolationReport;
    fn terminate(&self, pd: PdHandle) -> Result<(), SentinelError>;
    fn audit_caps(&self, pd: &PdHandle) -> Vec<IsolationViolation>;
}

/// SENTINEL error type
#[derive(Debug)]
pub enum SentinelError {
    PdSpawnFailed(String),
    CapabilityLeak,
    PdNotFound,
    IsolationBreach(IsolationViolation),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sentinel_cap_variants_exist() {
        let caps = vec![
            SentinelCap::DisplaySurface,
            SentinelCap::FontRead,
            SentinelCap::NetworkNone,
            SentinelCap::EdisonDbNoise,
        ];
        assert_eq!(caps.len(), 4);
    }

    #[test]
    fn pd_config_constructs() {
        let cfg = PdConfig {
            name: "haniel-renderer".to_string(),
            memory_pages: 256,
            caps: vec![SentinelCap::DisplaySurface, SentinelCap::FontRead],
        };
        assert_eq!(cfg.caps.len(), 2);
        assert_eq!(cfg.memory_pages, 256);
    }

    #[test]
    fn isolation_violation_network_attempt() {
        let v = IsolationViolation::NetworkAttempt {
            uri: "https://evil.com".to_string(),
        };
        match v {
            IsolationViolation::NetworkAttempt { uri } => {
                assert!(uri.contains("evil"));
            }
            _ => panic!("wrong variant"),
        }
    }
}
