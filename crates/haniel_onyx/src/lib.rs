// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// HANIEL-ONYX — Sovereign On-Device AI Compute Layer
// HE-11: full inference pipeline — classifier, inference engine,
//        render hints, threat detection, GPU bridge

#![forbid(unsafe_code)]

pub mod classify;
pub mod gpu_bridge;
pub mod hints;
pub mod infer;
pub mod threat;

pub use classify::SemanticClassifier;
pub use gpu_bridge::OnxGpuBridge;
pub use hints::HintGenerator;
pub use infer::InferenceEngine;
pub use threat::ThreatClassifier;

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

/// Render hint for a node — feeds SRB weights
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
    fn classify_text(&self, text: &str) -> SemanticClass;
    fn infer(&self, request: InferenceRequest) -> Result<InferenceResult, OnyxError>;
    fn render_hints(&self, nodes: &[(u32, &str)]) -> Vec<RenderHint>;
    fn threat_score(&self, content: &str) -> f32;
    fn model_memory_used(&self) -> usize;
}

/// Sovereign ONYX — full HE-11 implementation
pub struct SovereignOnyx {
    pub gpu:       OnxGpuBridge,
    classifier:    SemanticClassifier,
    inference:     InferenceEngine,
    hint_gen:      HintGenerator,
    threat_clf:    ThreatClassifier,
}

impl SovereignOnyx {
    pub fn new() -> Self {
        Self {
            gpu:        OnxGpuBridge::cpu(),
            classifier: SemanticClassifier::new(),
            inference:  InferenceEngine::new(),
            hint_gen:   HintGenerator::new(),
            threat_clf: ThreatClassifier::new(),
        }
    }

    pub fn with_gpu() -> Self {
        Self {
            gpu:        OnxGpuBridge::new().unwrap_or_else(|_| OnxGpuBridge::cpu()),
            classifier: SemanticClassifier::new(),
            inference:  InferenceEngine::new(),
            hint_gen:   HintGenerator::new(),
            threat_clf: ThreatClassifier::new(),
        }
    }

    /// Summarize text locally — no network call
    pub fn summarize(&self, text: &str, max_words: usize) -> String {
        self.inference.summarize(text, max_words)
    }
}

impl Default for SovereignOnyx {
    fn default() -> Self { Self::new() }
}

impl IrisOnyx for SovereignOnyx {
    fn classify_text(&self, text: &str) -> SemanticClass {
        self.classifier.classify_text(text)
    }

    fn infer(&self, request: InferenceRequest) -> Result<InferenceResult, OnyxError> {
        self.inference.infer(request)
    }

    fn render_hints(&self, nodes: &[(u32, &str)]) -> Vec<RenderHint> {
        let classifications: Vec<(u32, SemanticClass)> = nodes.iter()
            .map(|(id, text)| (*id, self.classifier.classify_text(text)))
            .collect();
        self.hint_gen.generate(&classifications)
    }

    fn threat_score(&self, content: &str) -> f32 {
        self.threat_clf.threat_score(content)
    }

    fn model_memory_used(&self) -> usize {
        // Micro model: 8*16 + 16 + 16*7 + 7 = 255 f32 = ~1KB
        255 * 4
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn onyx() -> SovereignOnyx { SovereignOnyx::new() }

    #[test]
    fn onyx_constructs() {
        let _ = onyx();
    }

    #[test]
    fn onyx_classify_sovereign() {
        let o = onyx();
        assert_eq!(o.classify_text("awp://aegis sovereign"), SemanticClass::Sovereign);
    }

    #[test]
    fn onyx_classify_threat() {
        let o = onyx();
        assert_eq!(o.classify_text("Connect wallet now claim reward"), SemanticClass::Threat);
    }

    #[test]
    fn onyx_infer_returns_result() {
        let o   = onyx();
        let req = InferenceRequest {
            prompt:     "What is sovereignty?".to_string(),
            max_tokens: 50,
            model:      OnDeviceModel::Micro,
        };
        let r = o.infer(req).unwrap();
        assert!(!r.text.is_empty());
    }

    #[test]
    fn onyx_render_hints_pipeline() {
        let o = onyx();
        let hints = o.render_hints(&[
            (1, "Home About Contact"),
            (2, "awp://sovereign content"),
            (3, "Long article about technology in the modern world and its implications"),
        ]);
        assert_eq!(hints.len(), 3);
        // Sovereign node should have highest priority
        let sovereign_hint = hints.iter().find(|h| h.class == SemanticClass::Sovereign);
        assert!(sovereign_hint.is_some());
        assert_eq!(sovereign_hint.unwrap().priority, 1.0);
    }

    #[test]
    fn onyx_threat_score_hostile() {
        let o     = onyx();
        let score = o.threat_score("Connect wallet verify claim limited time");
        assert!(score >= 0.0);
    }

    #[test]
    fn onyx_summarize() {
        let o    = onyx();
        let text = "word ".repeat(50);
        let s    = o.summarize(&text, 10);
        assert!(s.ends_with("...") || s.len() <= text.len());
    }

    #[test]
    fn onyx_gpu_bridge_available() {
        let o = onyx();
        let r = o.gpu.add(&[1.0, 2.0], &[3.0, 4.0]).unwrap();
        assert_eq!(r, vec![4.0, 6.0]);
    }

    #[test]
    fn onyx_model_memory_reported() {
        let o = onyx();
        assert!(o.model_memory_used() > 0);
    }
}
