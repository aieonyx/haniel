// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// haniel_echo::sandbox — Script execution sandbox
// Isolates scripts from OS, VAULT, HERALD
// seL4 PD isolation wired at HE-12 (SENTINEL)

use crate::capability::{CapabilityGate, CapScope};

/// Sandbox configuration
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    pub max_memory_bytes: usize,
    pub max_ops:          u64,
    pub allow_network:    bool,
    pub allow_storage:    bool,
}

impl SandboxConfig {
    /// Default sovereign sandbox — restrictive
    pub fn sovereign() -> Self {
        Self {
            max_memory_bytes: 4 * 1024 * 1024, // 4MB
            max_ops:          1_000_000,
            allow_network:    false,
            allow_storage:    false,
        }
    }

    /// Trusted sandbox — for sovereign AWP scripts
    pub fn trusted() -> Self {
        Self {
            max_memory_bytes: 16 * 1024 * 1024, // 16MB
            max_ops:          10_000_000,
            allow_network:    true,
            allow_storage:    true,
        }
    }
}

/// Script sandbox — enforces resource limits
pub struct ScriptSandbox {
    pub config: SandboxConfig,
    ops_used:   u64,
    mem_used:   usize,
}

impl ScriptSandbox {
    pub fn new(config: SandboxConfig) -> Self {
        Self { config, ops_used: 0, mem_used: 0 }
    }

    pub fn sovereign() -> Self {
        Self::new(SandboxConfig::sovereign())
    }

    /// Record an operation — returns false if limit exceeded
    pub fn tick(&mut self) -> bool {
        self.ops_used += 1;
        self.ops_used <= self.config.max_ops
    }

    /// Record memory allocation — returns false if limit exceeded
    pub fn alloc(&mut self, bytes: usize) -> bool {
        self.mem_used += bytes;
        self.mem_used <= self.config.max_memory_bytes
    }

    /// Check if network access is allowed
    pub fn check_network(&self, gate: &CapabilityGate) -> bool {
        self.config.allow_network && gate.check(&CapScope::NetworkFetch)
    }

    /// Check if storage access is allowed
    pub fn check_storage_read(&self, gate: &CapabilityGate) -> bool {
        self.config.allow_storage && gate.check(&CapScope::StorageRead)
    }

    /// Reset counters for next script execution
    pub fn reset(&mut self) {
        self.ops_used = 0;
        self.mem_used = 0;
    }

    pub fn ops_used(&self)   -> u64   { self.ops_used }
    pub fn mem_used(&self)   -> usize { self.mem_used }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sandbox_sovereign_config() {
        let sb = ScriptSandbox::sovereign();
        assert_eq!(sb.config.max_memory_bytes, 4 * 1024 * 1024);
        assert!(!sb.config.allow_network);
    }

    #[test]
    fn sandbox_tick_within_limit() {
        let mut sb = ScriptSandbox::new(SandboxConfig {
            max_ops: 5,
            ..SandboxConfig::sovereign()
        });
        for _ in 0..5 { assert!(sb.tick()); }
        assert!(!sb.tick()); // 6th tick exceeds limit
    }

    #[test]
    fn sandbox_alloc_within_limit() {
        let mut sb = ScriptSandbox::sovereign();
        assert!(sb.alloc(1024));
        assert!(sb.alloc(1024));
    }

    #[test]
    fn sandbox_network_denied_without_cap() {
        let sb   = ScriptSandbox::sovereign();
        let gate = CapabilityGate::untrusted();
        assert!(!sb.check_network(&gate));
    }

    #[test]
    fn sandbox_reset_clears_counters() {
        let mut sb = ScriptSandbox::sovereign();
        sb.tick();
        sb.alloc(100);
        sb.reset();
        assert_eq!(sb.ops_used(), 0);
        assert_eq!(sb.mem_used(), 0);
    }

    #[test]
    fn trusted_sandbox_allows_more_memory() {
        let sovereign = SandboxConfig::sovereign();
        let trusted   = SandboxConfig::trusted();
        assert!(trusted.max_memory_bytes > sovereign.max_memory_bytes);
    }
}
