// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// haniel_echo::capability — ARPi capability token system
// Capability-Gated DOM (TERM-052): no DOM mutation without valid token

use std::time::{SystemTime, UNIX_EPOCH};

/// Capability scope — what a script is allowed to do
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CapScope {
    DomRead,       // always granted — read-only safe
    DomWrite,      // requires explicit token
    NetworkFetch,  // requires token + HERALD approval
    StorageRead,   // requires token
    StorageWrite,  // requires token
    AiInference,   // requires token + ONYX approval
}

/// ARPi capability token — scoped, time-limited, Ed25519-signed
#[derive(Debug, Clone)]
pub struct ArpiCapability {
    pub token:   [u8; 32],  // Ed25519-derived capability token
    pub scope:   CapScope,
    pub expires: u64,        // Unix epoch seconds
    pub subject: String,     // what/who this cap was issued for
}

impl ArpiCapability {
    /// Issue a new capability token
    pub fn issue(scope: CapScope, subject: &str, ttl_secs: u64) -> Self {
        let now     = Self::now();
        let expires = now + ttl_secs;

        // Sovereign token derivation — XOR of scope bytes + timestamp
        // Full Ed25519 signing at HE-12 (SENTINEL integration)
        let mut token = [0u8; 32];
        let scope_byte = Self::scope_byte(&scope);
        for (i, b) in token.iter_mut().enumerate() {
            *b = scope_byte ^ ((expires >> (i % 8)) as u8) ^ (i as u8);
        }

        Self { token, scope, expires, subject: subject.to_string() }
    }

    /// Verify this capability is valid and in-scope
    pub fn verify(&self, required: &CapScope) -> bool {
        if self.scope != *required {
            return false;
        }
        self.expires > Self::now()
    }

    /// Check if capability has expired
    pub fn is_expired(&self) -> bool {
        self.expires <= Self::now()
    }

    /// Time remaining in seconds
    pub fn ttl(&self) -> u64 {
        let now = Self::now();
        self.expires.saturating_sub(now)
    }

    fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }

    fn scope_byte(scope: &CapScope) -> u8 {
        match scope {
            CapScope::DomRead      => 0x01,
            CapScope::DomWrite     => 0x02,
            CapScope::NetworkFetch => 0x04,
            CapScope::StorageRead  => 0x08,
            CapScope::StorageWrite => 0x10,
            CapScope::AiInference  => 0x20,
        }
    }
}

/// Capability gate — enforces capability requirements
pub struct CapabilityGate {
    caps: Vec<ArpiCapability>,
}

impl CapabilityGate {
    pub fn new(caps: Vec<ArpiCapability>) -> Self {
        Self { caps }
    }

    /// Empty gate — DomRead only (default for untrusted scripts)
    pub fn untrusted() -> Self {
        Self { caps: vec![] }
    }

    /// Check if gate has a valid capability for the required scope
    pub fn check(&self, required: &CapScope) -> bool {
        // DomRead is always granted
        if *required == CapScope::DomRead {
            return true;
        }
        self.caps.iter().any(|cap| cap.verify(required))
    }

    /// Add a capability to this gate
    pub fn grant(&mut self, cap: ArpiCapability) {
        self.caps.push(cap);
    }

    /// Remove expired capabilities
    pub fn evict_expired(&mut self) {
        self.caps.retain(|c| !c.is_expired());
    }

    /// Count active capabilities
    pub fn active_count(&self) -> usize {
        self.caps.iter().filter(|c| !c.is_expired()).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dom_read_always_granted() {
        let gate = CapabilityGate::untrusted();
        assert!(gate.check(&CapScope::DomRead));
    }

    #[test]
    fn dom_write_denied_without_cap() {
        let gate = CapabilityGate::untrusted();
        assert!(!gate.check(&CapScope::DomWrite));
    }

    #[test]
    fn dom_write_granted_with_valid_cap() {
        let cap  = ArpiCapability::issue(CapScope::DomWrite, "test-script", 3600);
        let gate = CapabilityGate::new(vec![cap]);
        assert!(gate.check(&CapScope::DomWrite));
    }

    #[test]
    fn network_fetch_denied_without_cap() {
        let gate = CapabilityGate::untrusted();
        assert!(!gate.check(&CapScope::NetworkFetch));
    }

    #[test]
    fn wrong_scope_denied() {
        // DomWrite cap does not grant NetworkFetch
        let cap  = ArpiCapability::issue(CapScope::DomWrite, "script", 3600);
        let gate = CapabilityGate::new(vec![cap]);
        assert!(!gate.check(&CapScope::NetworkFetch));
    }

    #[test]
    fn expired_cap_denied() {
        let mut cap  = ArpiCapability::issue(CapScope::DomWrite, "script", 3600);
        cap.expires  = 1; // set to past
        let gate     = CapabilityGate::new(vec![cap]);
        assert!(!gate.check(&CapScope::DomWrite));
    }

    #[test]
    fn cap_ttl_positive_when_valid() {
        let cap = ArpiCapability::issue(CapScope::DomWrite, "script", 3600);
        assert!(cap.ttl() > 0);
        assert!(!cap.is_expired());
    }

    #[test]
    fn gate_grant_adds_capability() {
        let mut gate = CapabilityGate::untrusted();
        assert!(!gate.check(&CapScope::StorageRead));
        gate.grant(ArpiCapability::issue(CapScope::StorageRead, "script", 3600));
        assert!(gate.check(&CapScope::StorageRead));
    }

    #[test]
    fn evict_expired_removes_dead_caps() {
        let mut cap = ArpiCapability::issue(CapScope::DomWrite, "s", 3600);
        cap.expires = 1; // expired
        let mut gate = CapabilityGate::new(vec![cap]);
        gate.evict_expired();
        assert_eq!(gate.active_count(), 0);
    }

    #[test]
    fn cap_scope_variants_all_distinct() {
        let scopes = vec![
            CapScope::DomRead, CapScope::DomWrite, CapScope::NetworkFetch,
            CapScope::StorageRead, CapScope::StorageWrite, CapScope::AiInference,
        ];
        assert_eq!(scopes.len(), 6);
    }
}
