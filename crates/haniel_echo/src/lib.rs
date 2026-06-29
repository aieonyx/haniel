// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// HANIEL ECHO — Script Runtime (AxonScript + WASM, capability-gated DOM)
// HE-9: axon_wasm wired, capability gate live, DOM mutation pipeline active

#![forbid(unsafe_code)]

pub mod capability;
pub mod dom;
pub mod runner;
pub mod sandbox;

pub use capability::{ArpiCapability, CapabilityGate, CapScope};
pub use dom::{DomMutation, DomPipeline};
pub use runner::ScriptRunner;
pub use sandbox::{ScriptSandbox, SandboxConfig};

/// Script source type
#[derive(Debug, Clone)]
pub enum ScriptSource {
    AxonScript(Vec<u8>),  // .ax sovereign script
    Wasm(Vec<u8>),        // WASM binary
}

/// Script execution result
#[derive(Debug)]
pub struct ScriptResult {
    pub dom_mutations:       Vec<DomMutation>,
    pub events_emitted:      Vec<String>,
    pub capability_requests: Vec<CapScope>,
    pub return_value:        Option<String>,
}

/// ECHO error type
#[derive(Debug)]
pub enum EchoError {
    CapabilityDenied(CapScope),
    CompileError(String),
    RuntimeError(String),
    SandboxViolation(String),
    Timeout,
}

/// ECHO sovereign script runtime trait
pub trait Echo: Send + Sync {
    fn execute(
        &self,
        source:   &ScriptSource,
        gate:     &CapabilityGate,
        pipeline: &mut DomPipeline,
    ) -> Result<ScriptResult, EchoError>;

    fn memory_used(&self) -> usize;
}

/// Sovereign ECHO implementation
pub struct SovereignEcho {
    runner:  ScriptRunner,
    sandbox: ScriptSandbox,
}

impl SovereignEcho {
    pub fn new() -> Self {
        Self {
            runner:  ScriptRunner::new(),
            sandbox: ScriptSandbox::sovereign(),
        }
    }
}

impl Default for SovereignEcho {
    fn default() -> Self { Self::new() }
}

impl Echo for SovereignEcho {
    fn execute(
        &self,
        source:   &ScriptSource,
        gate:     &CapabilityGate,
        pipeline: &mut DomPipeline,
    ) -> Result<ScriptResult, EchoError> {
        self.runner.execute(source, gate, pipeline)
    }

    fn memory_used(&self) -> usize {
        self.sandbox.mem_used()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn echo() -> SovereignEcho { SovereignEcho::new() }

    fn trusted_gate() -> CapabilityGate {
        let cap = ArpiCapability::issue(CapScope::DomWrite, "test", 3600);
        CapabilityGate::new(vec![cap])
    }

    #[test]
    fn echo_constructs() {
        let _ = echo();
    }

    #[test]
    fn echo_executes_axonscript() {
        let e    = echo();
        let gate = trusted_gate();
        let mut pipeline = DomPipeline::new();
        let src  = ScriptSource::AxonScript(b"set_text 1 Sovereign".to_vec());
        let result = e.execute(&src, &gate, &mut pipeline).unwrap();
        assert_eq!(result.dom_mutations.len(), 1);
    }

    #[test]
    fn echo_executes_empty_wasm() {
        let e    = echo();
        let gate = trusted_gate();
        let mut pipeline = DomPipeline::new();
        let wasm = vec![0x00,0x61,0x73,0x6d,0x01,0x00,0x00,0x00];
        let src  = ScriptSource::Wasm(wasm);
        let result = e.execute(&src, &gate, &mut pipeline);
        assert!(result.is_ok());
    }

    #[test]
    fn echo_dom_write_blocked_without_cap() {
        let e    = echo();
        let gate = CapabilityGate::untrusted();
        let mut pipeline = DomPipeline::new();
        let src  = ScriptSource::AxonScript(b"set_text 1 Hello".to_vec());
        let result = e.execute(&src, &gate, &mut pipeline);
        assert!(matches!(result, Err(EchoError::CapabilityDenied(_))));
    }

    #[test]
    fn echo_pipeline_tracks_dirty_nodes() {
        let e    = echo();
        let gate = trusted_gate();
        let mut pipeline = DomPipeline::new();
        let src  = ScriptSource::AxonScript(b"set_text 5 Hello
set_style 7 color red".to_vec());
        e.execute(&src, &gate, &mut pipeline).unwrap();
        assert_eq!(pipeline.dirty_count(), 2);
    }

    #[test]
    fn cap_scope_dom_read_always_granted() {
        let gate = CapabilityGate::untrusted();
        assert!(gate.check(&CapScope::DomRead));
    }

    #[test]
    fn script_result_has_mutations() {
        let e    = echo();
        let gate = trusted_gate();
        let mut pipeline = DomPipeline::new();
        let script = b"set_text 1 a
set_text 2 b
remove 3";
        let src  = ScriptSource::AxonScript(script.to_vec());
        let result = e.execute(&src, &gate, &mut pipeline).unwrap();
        assert_eq!(result.dom_mutations.len(), 3);
    }
}
