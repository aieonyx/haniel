// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// HANIEL ECHO — Script Runtime (AxonScript + WASM, capability-gated DOM)
// HE-9 implementation target

#![forbid(unsafe_code)]

pub mod capability;
pub mod dom;
pub mod runner;
pub mod sandbox;

// u32 wired at HE-9

/// ARPi capability token
#[derive(Debug, Clone)]
pub struct ArpiCapability {
    pub token: [u8; 32],
    pub scope: CapScope,
    pub expires: u64,
}

/// Capability scope
#[derive(Debug, Clone, PartialEq)]
pub enum CapScope {
    DomRead,
    DomWrite,
    NetworkFetch,
    StorageRead,
    StorageWrite,
    AiInference,
}

/// Script source type
#[derive(Debug)]
pub enum ScriptSource {
    AxonScript(Vec<u8>),
    Wasm(Vec<u8>),
}

/// DOM mutation produced by script
#[derive(Debug, Clone)]
pub enum DomMutation {
    SetText   { node: u32, text: String },
    SetStyle  { node: u32, property: String, value: String },
    AppendChild { parent: u32, child: u32 },
    RemoveNode  { node: u32 },
}

/// Script execution result
#[derive(Debug)]
pub struct ScriptResult {
    pub dom_mutations: Vec<DomMutation>,
    pub events_emitted: Vec<String>,
    pub capability_requests: Vec<CapScope>,
}

/// ECHO script runtime trait
pub trait Echo: Send + Sync {
    fn execute(
        &self,
        source: ScriptSource,
        caps: &[ArpiCapability],
    ) -> Result<ScriptResult, EchoError>;
    fn verify_capability(&self, cap: &ArpiCapability, scope: CapScope) -> bool;
    fn compile(&self, source: &[u8]) -> Result<Vec<u8>, EchoError>;
    fn memory_used(&self) -> usize;
}

/// ECHO error type
#[derive(Debug)]
pub enum EchoError {
    CapabilityDenied(CapScope),
    CompileError(String),
    RuntimeError(String),
    Timeout,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cap_scope_variants_exist() {
        let _r = CapScope::DomRead;
        let _w = CapScope::DomWrite;
        assert_ne!(CapScope::DomRead, CapScope::DomWrite);
    }

    #[test]
    fn dom_mutation_set_text() {
        let m = DomMutation::SetText { node: 1, text: "Hello".into() };
        match m {
            DomMutation::SetText { node, text } => {
                assert_eq!(node, 1);
                assert_eq!(text, "Hello");
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn arpi_capability_token_size() {
        let cap = ArpiCapability {
            token: [0u8; 32],
            scope: CapScope::DomRead,
            expires: 9999999999,
        };
        assert_eq!(cap.token.len(), 32);
    }
}
