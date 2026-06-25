// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// HANIEL-ONYX — Sovereign On-Device AI Compute Layer
// HE-11 implementation target

#![forbid(unsafe_code)]

pub mod classify;
pub mod infer;
pub mod hints;
pub mod threat;

use crate::haniel_prism::{LayoutTree, NodeId};

/// Semantic content classification
#[derive(Debug, Clone, PartialEq)]
pub enum SemanticClass {
    Article,
    Navigation,
    Form,
    Media,
    Threat,
    Sovereign,
    Unknown,
}

/// Render hint for a node
#[derive(Debug, Clone)]
pub struct RenderHint {
    pub node: NodeId,
    pub class: SemanticClass,
    pub priority: f32,
    pub ai_confidence: f32,
}

/// On-device model size selector
#[derive(Debug, Clone, PartialEq)]
pub enum OnDeviceModel {
    Micro,
    Standard,
    Full,
}

/// Inference request
#[derive(Debug)]
pub struct InferenceRequest {
    pub prompt: String,
    pub max_tokens: u32,
    pub model: OnDeviceModel,
}

/// Inference result
#[derive(Debug)]
pub struct InferenceResult {
    pub text: String,
    pub tokens_generated: u32,
    pub latency_ms: u64,
    pub model_used: OnDeviceModel,
}

/// HANIEL-ONYX AI compute trait
pub trait IrisOnyx: Send + Sync {
    fn classify(&self, tree: &LayoutTree) -> Result<Vec<RenderHint>, OnyxError>;
    fn infer(&self, request: InferenceRequest) -> Result<InferenceResult, OnyxError>;
    fn render_hints(&self, tree: &LayoutTree) -> Result<Vec<RenderHint>, OnyxError>;
    fn threat_classify(&self, content: &str) -> SemanticClass;
    fn model_memory_used(&self) -> usize;
}

/// ONYX error type
#[derive(Debug)]
pub enum OnyxError {
    ModelNotLoaded,
    InferenceFailed(String),
    OutOfMemory,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semantic_class_variants_exist() {
        let classes = vec![
            SemanticClass::Article,
            SemanticClass::Navigation,
            SemanticClass::Threat,
            SemanticClass::Sovereign,
        ];
        assert_eq!(classes.len(), 4);
    }

    #[test]
    fn render_hint_priority_range() {
        let hint = RenderHint {
            node: 0,
            class: SemanticClass::Article,
            priority: 0.9,
            ai_confidence: 0.85,
        };
        assert!(hint.priority >= 0.0 && hint.priority <= 1.0);
        assert!(hint.ai_confidence >= 0.0 && hint.ai_confidence <= 1.0);
    }

    #[test]
    fn on_device_model_variants() {
        assert_ne!(OnDeviceModel::Micro, OnDeviceModel::Full);
    }
}
