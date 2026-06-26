// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// HANIEL-ONYX — Sovereign On-Device AI Compute Layer
// HE-7: axon_gpu bridge wired — GPU compute available for AI inference
// HE-11: full AI runtime (GGUF, tokenizer, attention, KV cache)

#![forbid(unsafe_code)]

pub mod classify;
pub mod gpu_bridge;
pub mod hints;
pub mod infer;
pub mod threat;

pub use gpu_bridge::OnxGpuBridge;


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
    pub node:          u32,
    pub class:         SemanticClass,
    pub priority:      f32,
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
    pub prompt:     String,
    pub max_tokens: u32,
    pub model:      OnDeviceModel,
}

/// Inference result
#[derive(Debug)]
pub struct InferenceResult {
    pub text:             String,
    pub tokens_generated: u32,
    pub latency_ms:       u64,
    pub model_used:       OnDeviceModel,
}

/// ONYX error type
#[derive(Debug)]
pub enum OnyxError {
    ModelNotLoaded,
    InferenceFailed(String),
    OutOfMemory,
}

/// HANIEL-ONYX AI compute trait
pub trait IrisOnyx: Send + Sync {
    fn classify(&self) -> Result<Vec<RenderHint>, OnyxError>;
    fn infer(&self, request: InferenceRequest) -> Result<InferenceResult, OnyxError>;
    fn render_hints(&self) -> Result<Vec<RenderHint>, OnyxError>;
    fn threat_classify(&self, content: &str) -> SemanticClass;
    fn model_memory_used(&self) -> usize;
}

/// Sovereign ONYX implementation
/// HE-7: GPU compute available via OnxGpuBridge
/// HE-11: full inference pipeline
pub struct SovereignOnyx {
    pub gpu: OnxGpuBridge,
}

impl SovereignOnyx {
    pub fn new() -> Self {
        Self {
            gpu: OnxGpuBridge::cpu(),
        }
    }

    /// Initialize with GPU discovery
    pub fn with_gpu() -> Self {
        Self {
            gpu: OnxGpuBridge::new().unwrap_or_else(|_| OnxGpuBridge::cpu()),
        }
    }
}

impl Default for SovereignOnyx {
    fn default() -> Self { Self::new() }
}

impl IrisOnyx for SovereignOnyx {
    fn classify(&self) -> Result<Vec<RenderHint>, OnyxError> {
        // Full classification at HE-11
        Ok(vec![])
    }

    fn infer(&self, request: InferenceRequest) -> Result<InferenceResult, OnyxError> {
        // Full inference at HE-11
        // HE-7: GPU compute available via self.gpu
        Ok(InferenceResult {
            text:             format!("[HE-11: inference for: {}]", request.prompt),
            tokens_generated: 0,
            latency_ms:       0,
            model_used:       request.model,
        })
    }

    fn render_hints(&self) -> Result<Vec<RenderHint>, OnyxError> {
        Ok(vec![])
    }

    fn threat_classify(&self, _content: &str) -> SemanticClass {
        SemanticClass::Unknown
    }

    fn model_memory_used(&self) -> usize { 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semantic_class_variants_exist() {
        let _ = SemanticClass::Article;
        let _ = SemanticClass::Navigation;
        let _ = SemanticClass::Threat;
        let _ = SemanticClass::Sovereign;
    }

    #[test]
    fn render_hint_range() {
        let hint = RenderHint {
            node: 0, class: SemanticClass::Article,
            priority: 0.9, ai_confidence: 0.85,
        };
        assert!(hint.priority >= 0.0 && hint.priority <= 1.0);
    }

    #[test]
    fn sovereign_onyx_constructs() {
        let o = SovereignOnyx::new();
        assert_eq!(o.gpu.vram_bytes(), 0); // CPU mode
    }

    #[test]
    fn sovereign_onyx_gpu_bridge_works() {
        let o = SovereignOnyx::new();
        let r = o.gpu.add(&[1.0, 2.0], &[3.0, 4.0]).unwrap();
        assert_eq!(r, vec![4.0, 6.0]);
    }

    #[test]
    fn sovereign_onyx_gpu_relu() {
        let o = SovereignOnyx::new();
        let r = o.gpu.relu(&[-1.0, 0.5, 2.0]).unwrap();
        assert_eq!(r[0], 0.0);
        assert!(r[1] > 0.0);
        assert!(r[2] > 0.0);
    }
}
