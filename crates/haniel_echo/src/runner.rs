// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// haniel_echo::runner — Script execution engine
// Runs AxonScript and WASM via axon_wasm

use axon_wasm::{WasmRuntime, WasmValue};
use crate::capability::CapabilityGate;
use crate::{ScriptSource, ScriptResult, EchoError};
use crate::dom::{DomPipeline, DomMutation};

/// Sovereign script runner — executes AxonScript and WASM
pub struct ScriptRunner;

impl ScriptRunner {
    pub fn new() -> Self { Self }

    /// Execute a script with given capability gate
    pub fn execute(
        &self,
        source:   &ScriptSource,
        gate:     &CapabilityGate,
        pipeline: &mut DomPipeline,
    ) -> Result<ScriptResult, EchoError> {
        match source {
            ScriptSource::Wasm(bytes) => {
                self.execute_wasm(bytes, gate, pipeline)
            }
            ScriptSource::AxonScript(bytes) => {
                self.execute_axonscript(bytes, gate, pipeline)
            }
        }
    }

    /// Execute WASM bytecode via axon_wasm runtime
    fn execute_wasm(
        &self,
        bytes:    &[u8],
        gate:     &CapabilityGate,
        pipeline: &mut DomPipeline,
    ) -> Result<ScriptResult, EchoError> {
        // Parse and validate WASM
        let runtime = WasmRuntime::instantiate(bytes)
            .map_err(|e| EchoError::RuntimeError(format!("WASM: {:?}", e)))?;

        // Execute — returns values from WASM stack
        let mut runtime = runtime;
        let results = runtime.call_by_name("main", vec![])
            .unwrap_or_default();

        // Convert WASM results to capability-gated DOM mutations
        // Full DOM bridge at HE-13 (AWP integration)
        let dom_mutations = self.wasm_results_to_mutations(&results, gate, pipeline)?;

        Ok(ScriptResult {
            dom_mutations,
            events_emitted:      vec![],
            capability_requests: vec![],
            return_value:        results.first().map(|v| format!("{:?}", v)),
        })
    }

    /// Execute AxonScript (sovereign scripting language)
    fn execute_axonscript(
        &self,
        bytes:    &[u8],
        gate:     &CapabilityGate,
        pipeline: &mut DomPipeline,
    ) -> Result<ScriptResult, EchoError> {
        // AxonScript interpreter — full axonc pipeline at HE-13
        // HE-9: parse as UTF-8 and execute simple text commands
        let script = std::str::from_utf8(bytes)
            .map_err(|_| EchoError::CompileError("invalid UTF-8".into()))?;

        let mut mutations = Vec::new();

        // Simple AxonScript interpreter for HE-9
        // Format: "set_text <node_id> <text>"
        //         "set_style <node_id> <property> <value>"
        //         "remove <node_id>"
        for line in script.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("//") { continue; }

            // Parse command word first
            let mut iter   = line.splitn(2, ' ');
            let cmd        = iter.next().unwrap_or("");
            let rest       = iter.next().unwrap_or("");

            match cmd {
                "set_text" => {
                    // set_text <node_id> <text...>
                    let mut parts = rest.splitn(2, ' ');
                    let node_str  = parts.next().unwrap_or("");
                    let text      = parts.next().unwrap_or("");
                    if let Ok(node) = node_str.parse::<u32>() {
                        let mutation = DomMutation::SetText {
                            node,
                            text: text.to_string(),
                        };
                        pipeline.queue(mutation.clone(), gate)?;
                        mutations.push(mutation);
                    }
                }
                "set_style" => {
                    // set_style <node_id> <property> <value>
                    let mut parts = rest.splitn(3, ' ');
                    let node_str  = parts.next().unwrap_or("");
                    let prop      = parts.next().unwrap_or("");
                    let val       = parts.next().unwrap_or("");
                    if let Ok(node) = node_str.parse::<u32>() {
                        let mutation = DomMutation::SetStyle {
                            node,
                            property: prop.to_string(),
                            value:    val.to_string(),
                        };
                        pipeline.queue(mutation.clone(), gate)?;
                        mutations.push(mutation);
                    }
                }
                "remove" => {
                    // remove <node_id>
                    if let Ok(node) = rest.trim().parse::<u32>() {
                        let mutation = DomMutation::RemoveNode { node };
                        pipeline.queue(mutation.clone(), gate)?;
                        mutations.push(mutation);
                    }
                }
                _ => {
                    // Unknown command — skip silently in sandbox
                }
            }
        }

        Ok(ScriptResult {
            dom_mutations:       mutations,
            events_emitted:      vec![],
            capability_requests: vec![],
            return_value:        None,
        })
    }

    /// Convert WASM return values to DOM mutations
    fn wasm_results_to_mutations(
        &self,
        results:  &[WasmValue],
        _gate:    &CapabilityGate,
        _pipeline: &mut DomPipeline,
    ) -> Result<Vec<DomMutation>, EchoError> {
        // Full WASM→DOM bridge at HE-13
        // HE-9: return empty mutations (WASM execution verified but no DOM bridge yet)
        let _ = results;
        Ok(vec![])
    }
}

impl Default for ScriptRunner {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::{ArpiCapability, CapScope};

    fn trusted_gate() -> CapabilityGate {
        let cap = ArpiCapability::issue(CapScope::DomWrite, "test", 3600);
        CapabilityGate::new(vec![cap])
    }

    fn untrusted_gate() -> CapabilityGate {
        CapabilityGate::untrusted()
    }

    fn runner() -> ScriptRunner { ScriptRunner::new() }

    // Minimal valid WASM binary — empty module
    fn empty_wasm() -> Vec<u8> {
        vec![
            0x00, 0x61, 0x73, 0x6d, // magic: \0asm
            0x01, 0x00, 0x00, 0x00, // version: 1
        ]
    }

    #[test]
    fn wasm_empty_module_executes() {
        let r   = runner();
        let mut pipeline = DomPipeline::new();
        let gate = trusted_gate();
        let src  = ScriptSource::Wasm(empty_wasm());
        let result = r.execute(&src, &gate, &mut pipeline);
        assert!(result.is_ok(), "empty WASM should execute: {:?}", result);
    }

    #[test]
    fn axonscript_set_text_with_cap() {
        let r    = runner();
        let mut pipeline = DomPipeline::new();
        let gate = trusted_gate();
        let script = b"set_text 1 Hello Sovereign World";
        let src  = ScriptSource::AxonScript(script.to_vec());
        let result = r.execute(&src, &gate, &mut pipeline).unwrap();
        assert_eq!(result.dom_mutations.len(), 1);
        match &result.dom_mutations[0] {
            DomMutation::SetText { node, text } => {
                assert_eq!(*node, 1);
                assert!(text.contains("Hello"));
            }
            _ => panic!("expected SetText"),
        }
    }

    #[test]
    fn axonscript_set_text_without_cap_fails() {
        let r    = runner();
        let mut pipeline = DomPipeline::new();
        let gate = untrusted_gate();
        let script = b"set_text 1 Hello";
        let src  = ScriptSource::AxonScript(script.to_vec());
        let result = r.execute(&src, &gate, &mut pipeline);
        assert!(matches!(result, Err(EchoError::CapabilityDenied(_))));
    }

    #[test]
    fn axonscript_set_style_with_cap() {
        let r    = runner();
        let mut pipeline = DomPipeline::new();
        let gate = trusted_gate();
        let script = b"set_style 2 color red";
        let src  = ScriptSource::AxonScript(script.to_vec());
        let result = r.execute(&src, &gate, &mut pipeline).unwrap();
        assert_eq!(result.dom_mutations.len(), 1);
        match &result.dom_mutations[0] {
            DomMutation::SetStyle { node, property, value } => {
                assert_eq!(*node, 2);
                assert_eq!(property, "color");
                assert_eq!(value, "red");
            }
            _ => panic!("expected SetStyle"),
        }
    }

    #[test]
    fn axonscript_remove_node() {
        let r    = runner();
        let mut pipeline = DomPipeline::new();
        let gate = trusted_gate();
        let script = b"remove 5";
        let src  = ScriptSource::AxonScript(script.to_vec());
        let result = r.execute(&src, &gate, &mut pipeline).unwrap();
        assert_eq!(result.dom_mutations.len(), 1);
        assert!(matches!(result.dom_mutations[0], DomMutation::RemoveNode { node: 5 }));
    }

    #[test]
    fn axonscript_comments_ignored() {
        let r    = runner();
        let mut pipeline = DomPipeline::new();
        let gate = trusted_gate();
        let script = b"// this is a comment

set_text 1 Hello";
        let src  = ScriptSource::AxonScript(script.to_vec());
        let result = r.execute(&src, &gate, &mut pipeline).unwrap();
        assert_eq!(result.dom_mutations.len(), 1);
    }

    #[test]
    fn axonscript_unknown_command_skipped() {
        let r    = runner();
        let mut pipeline = DomPipeline::new();
        let gate = trusted_gate();
        let script = b"unknown_command 1 2 3";
        let src  = ScriptSource::AxonScript(script.to_vec());
        let result = r.execute(&src, &gate, &mut pipeline).unwrap();
        assert_eq!(result.dom_mutations.len(), 0);
    }

    #[test]
    fn axonscript_multiple_mutations() {
        let r    = runner();
        let mut pipeline = DomPipeline::new();
        let gate = trusted_gate();
        let script = b"set_text 1 Hello
set_style 2 color blue
remove 3";
        let src  = ScriptSource::AxonScript(script.to_vec());
        let result = r.execute(&src, &gate, &mut pipeline).unwrap();
        assert_eq!(result.dom_mutations.len(), 3);
        assert_eq!(pipeline.pending_count(), 3);
    }
}
