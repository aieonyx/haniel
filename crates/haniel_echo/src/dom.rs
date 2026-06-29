// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// haniel_echo::dom — Capability-gated DOM mutation pipeline

use crate::capability::{CapabilityGate, CapScope};
use crate::EchoError;

/// A single DOM mutation produced by a script
#[derive(Debug, Clone)]
pub enum DomMutation {
    SetText      { node: u32, text: String },
    SetStyle     { node: u32, property: String, value: String },
    AppendChild  { parent: u32, child: u32 },
    RemoveNode   { node: u32 },
    SetAttribute { node: u32, name: String, value: String },
    SetClass     { node: u32, class: String },
}

/// DOM mutation pipeline — collects and applies mutations
/// All mutations require DomWrite capability
pub struct DomPipeline {
    pending:  Vec<DomMutation>,
    dirty:    Vec<u32>,  // node IDs that need re-layout
}

impl DomPipeline {
    pub fn new() -> Self {
        Self { pending: Vec::new(), dirty: Vec::new() }
    }

    /// Queue a mutation — requires DomWrite capability
    pub fn queue(
        &mut self,
        mutation: DomMutation,
        gate:     &CapabilityGate,
    ) -> Result<(), EchoError> {
        if !gate.check(&CapScope::DomWrite) {
            return Err(EchoError::CapabilityDenied(CapScope::DomWrite));
        }

        // Track dirty nodes for PRISM invalidation
        let node_id = match &mutation {
            DomMutation::SetText      { node, .. } => *node,
            DomMutation::SetStyle     { node, .. } => *node,
            DomMutation::AppendChild  { parent, .. } => *parent,
            DomMutation::RemoveNode   { node }      => *node,
            DomMutation::SetAttribute { node, .. }  => *node,
            DomMutation::SetClass     { node, .. }  => *node,
        };

        if !self.dirty.contains(&node_id) {
            self.dirty.push(node_id);
        }

        self.pending.push(mutation);
        Ok(())
    }

    /// Drain all pending mutations — returns (mutations, dirty_nodes)
    /// Called by HANIEL orchestrator to feed into PRISM invalidation
    pub fn drain(&mut self) -> (Vec<DomMutation>, Vec<u32>) {
        let mutations = std::mem::take(&mut self.pending);
        let dirty     = std::mem::take(&mut self.dirty);
        (mutations, dirty)
    }

    /// Count pending mutations
    pub fn pending_count(&self) -> usize { self.pending.len() }

    /// Count dirty nodes
    pub fn dirty_count(&self) -> usize { self.dirty.len() }

    /// Check if any mutations are pending
    pub fn is_empty(&self) -> bool { self.pending.is_empty() }
}

impl Default for DomPipeline {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::ArpiCapability;

    fn trusted_gate() -> CapabilityGate {
        let cap = ArpiCapability::issue(CapScope::DomWrite, "test", 3600);
        CapabilityGate::new(vec![cap])
    }

    fn untrusted_gate() -> CapabilityGate {
        CapabilityGate::untrusted()
    }

    #[test]
    fn mutation_accepted_with_dom_write_cap() {
        let mut pipeline = DomPipeline::new();
        let gate         = trusted_gate();
        let result = pipeline.queue(
            DomMutation::SetText { node: 1, text: "Hello".into() },
            &gate,
        );
        assert!(result.is_ok());
        assert_eq!(pipeline.pending_count(), 1);
    }

    #[test]
    fn mutation_rejected_without_cap() {
        let mut pipeline = DomPipeline::new();
        let gate         = untrusted_gate();
        let result = pipeline.queue(
            DomMutation::SetText { node: 1, text: "Hello".into() },
            &gate,
        );
        assert!(matches!(result, Err(EchoError::CapabilityDenied(_))));
        assert_eq!(pipeline.pending_count(), 0);
    }

    #[test]
    fn dirty_nodes_tracked() {
        let mut pipeline = DomPipeline::new();
        let gate         = trusted_gate();
        pipeline.queue(DomMutation::SetText { node: 5, text: "a".into() }, &gate).unwrap();
        pipeline.queue(DomMutation::SetStyle { node: 7, property: "color".into(), value: "red".into() }, &gate).unwrap();
        assert_eq!(pipeline.dirty_count(), 2);
    }

    #[test]
    fn dirty_nodes_deduplicated() {
        let mut pipeline = DomPipeline::new();
        let gate         = trusted_gate();
        pipeline.queue(DomMutation::SetText { node: 3, text: "a".into() }, &gate).unwrap();
        pipeline.queue(DomMutation::SetText { node: 3, text: "b".into() }, &gate).unwrap();
        assert_eq!(pipeline.dirty_count(), 1); // same node, deduplicated
    }

    #[test]
    fn drain_clears_pipeline() {
        let mut pipeline = DomPipeline::new();
        let gate         = trusted_gate();
        pipeline.queue(DomMutation::RemoveNode { node: 2 }, &gate).unwrap();
        let (mutations, dirty) = pipeline.drain();
        assert_eq!(mutations.len(), 1);
        assert_eq!(dirty.len(), 1);
        assert!(pipeline.is_empty());
        assert_eq!(pipeline.dirty_count(), 0);
    }

    #[test]
    fn append_child_tracks_parent_dirty() {
        let mut pipeline = DomPipeline::new();
        let gate         = trusted_gate();
        pipeline.queue(DomMutation::AppendChild { parent: 10, child: 11 }, &gate).unwrap();
        assert!(pipeline.dirty.contains(&10));
    }

    #[test]
    fn set_attribute_mutation() {
        let mut pipeline = DomPipeline::new();
        let gate         = trusted_gate();
        let result = pipeline.queue(
            DomMutation::SetAttribute { node: 1, name: "href".into(), value: "awp://home".into() },
            &gate,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn pipeline_starts_empty() {
        let pipeline = DomPipeline::new();
        assert!(pipeline.is_empty());
        assert_eq!(pipeline.pending_count(), 0);
        assert_eq!(pipeline.dirty_count(), 0);
    }
}
