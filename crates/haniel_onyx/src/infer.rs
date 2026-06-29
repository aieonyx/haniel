// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// haniel_onyx::infer — Sovereign on-device inference engine
// Uses axon_ai_runtime SovereignModel + ComputeGraph

use axon_ai_runtime::{Tensor, SovereignModel, DenseLayer, Activation};
use crate::{InferenceRequest, InferenceResult, OnyxError};
use std::time::Instant;

/// Sovereign inference engine
pub struct InferenceEngine {
    /// Micro model — always resident (~50MB equivalent in production)
    micro_model: SovereignModel,
}

impl InferenceEngine {
    pub fn new() -> Self {
        Self {
            micro_model: Self::build_micro_model(),
        }
    }

    /// Build the Micro model — lightweight 2-layer feedforward
    /// Production: loads from .iam file. HE-11: synthetic weights.
    fn build_micro_model() -> SovereignModel {
        let mut model = SovereignModel::new("haniel-micro");

        // Layer 1: 8 → 16 (ReLU)
        if let (Ok(w1), Ok(b1)) = (
            Tensor::zeros(vec![8, 16]),
            Tensor::zeros(vec![16]),
        ) {
            if let Ok(layer) = DenseLayer::new("layer1", w1, Some(b1), Activation::ReLU) {
                model.add_layer(layer);
            }
        }

        // Layer 2: 16 → 7 (Softmax — 7 semantic classes)
        if let (Ok(w2), Ok(b2)) = (
            Tensor::zeros(vec![16, 7]),
            Tensor::zeros(vec![7]),
        ) {
            if let Ok(layer) = DenseLayer::new("layer2", w2, Some(b2), Activation::Softmax) {
                model.add_layer(layer);
            }
        }

        model
    }

    /// Run inference on a text prompt
    pub fn infer(&self, request: InferenceRequest)
        -> Result<InferenceResult, OnyxError>
    {
        let start = Instant::now();

        // Tokenize prompt (sovereign BPE at HE-13, simple word count now)
        let token_count = request.prompt.split_whitespace().count();

        // Build input tensor from prompt features
        let input = self.prompt_to_tensor(&request.prompt)?;

        // Run forward pass through micro model
        let output = self.micro_model.infer(&input)
            .map_err(|e| OnyxError::InferenceFailed(format!("{:?}", e)))?;

        // Decode output to text
        let text = self.decode_output(&output, &request.prompt, request.max_tokens);

        let latency_ms = start.elapsed().as_millis() as u64;

        Ok(InferenceResult {
            text,
            tokens_generated: token_count.min(request.max_tokens as usize) as u32,
            latency_ms,
            model_used: request.model,
        })
    }

    /// Convert prompt to input tensor (8 features)
    fn prompt_to_tensor(&self, prompt: &str) -> Result<Tensor, OnyxError> {
        let lower    = prompt.to_lowercase();
        let len_norm = (prompt.len() as f32 / 500.0).min(1.0);
        let words    = prompt.split_whitespace().count() as f32 / 100.0;

        let features = vec![
            len_norm,
            words.min(1.0),
            if lower.contains("?") { 1.0 } else { 0.0 },
            if lower.contains("summarize") || lower.contains("explain") { 1.0 } else { 0.0 },
            if lower.contains("threat") || lower.contains("danger") { 1.0 } else { 0.0 },
            if lower.contains("sovereign") || lower.contains("aieonyx") { 1.0 } else { 0.0 },
            if lower.contains("video") || lower.contains("media") { 1.0 } else { 0.0 },
            if prompt.len() > 100 { 1.0 } else { 0.0 },
        ];

        Tensor::from_vec(features)
            .map_err(|e| OnyxError::InferenceFailed(format!("tensor: {:?}", e)))
    }

    /// Decode output tensor to response text
    fn decode_output(&self, output: &Tensor, prompt: &str, _max_tokens: u32) -> String {
        // HE-11: template-based decode
        // Full LLM decode at HE-13 (GGUF + tokenizer)
        let dominant_class = output.data.iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(i, _)| i)
            .unwrap_or(0);

        match dominant_class {
            0 => format!("[HANIEL-ONYX] Processing: {}", &prompt[..prompt.len().min(50)]),
            1 => "[HANIEL-ONYX] Navigation content detected.".to_string(),
            2 => "[HANIEL-ONYX] Article content — sovereign reading mode active.".to_string(),
            3 => "[HANIEL-ONYX] Media content detected — LUMEN engaged.".to_string(),
            4 => "[HANIEL-ONYX] Form detected — ARPi verification recommended.".to_string(),
            5 => "[HANIEL-ONYX] THREAT DETECTED — STS gate active.".to_string(),
            6 => "[HANIEL-ONYX] Sovereign content — full trust granted.".to_string(),
            _ => "[HANIEL-ONYX] Analysis complete.".to_string(),
        }
    }

    /// Summarize text content locally
    pub fn summarize(&self, text: &str, max_words: usize) -> String {
        // Extractive summarization — first N words
        // Full abstractive at HE-13
        let words: Vec<&str> = text.split_whitespace().collect();
        if words.len() <= max_words {
            return text.to_string();
        }
        format!("{}...", words[..max_words].join(" "))
    }

    /// Detect AI-generated hostile content
    pub fn detect_hostile(&self, content: &str) -> f32 {
        let lower = content.to_lowercase();
        let mut score = 0.0f32;

        let hostile_patterns = [
            "connect wallet", "verify now", "claim your",
            "limited time offer", "act now", "you have won",
            "send crypto", "seed phrase", "private key",
        ];

        for pattern in &hostile_patterns {
            if lower.contains(pattern) {
                score += 0.2;
            }
        }

        score.min(1.0)
    }
}

impl Default for InferenceEngine {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::OnDeviceModel;

    fn engine() -> InferenceEngine { InferenceEngine::new() }

    #[test]
    fn inference_engine_constructs() {
        let _ = engine();
    }

    #[test]
    fn infer_returns_result() {
        let e = engine();
        let req = InferenceRequest {
            prompt:     "What is sovereign computing?".to_string(),
            max_tokens: 50,
            model:      OnDeviceModel::Micro,
        };
        let result = e.infer(req).unwrap();
        assert!(!result.text.is_empty());
        assert!(result.latency_ms < 1000);
    }

    #[test]
    fn infer_sovereign_prompt() {
        let e = engine();
        let req = InferenceRequest {
            prompt:     "sovereign aieonyx system".to_string(),
            max_tokens: 20,
            model:      OnDeviceModel::Micro,
        };
        let result = e.infer(req).unwrap();
        assert!(result.tokens_generated > 0);
    }

    #[test]
    fn infer_question_prompt() {
        let e = engine();
        let req = InferenceRequest {
            prompt:     "What is the weather today?".to_string(),
            max_tokens: 30,
            model:      OnDeviceModel::Micro,
        };
        let result = e.infer(req);
        assert!(result.is_ok());
    }

    #[test]
    fn summarize_short_text_unchanged() {
        let e    = engine();
        let text = "Short text.";
        let s    = e.summarize(text, 100);
        assert_eq!(s, text);
    }

    #[test]
    fn summarize_long_text_truncated() {
        let e    = engine();
        let text = "word ".repeat(200);
        let s    = e.summarize(&text, 10);
        assert!(s.ends_with("..."));
        assert!(s.split_whitespace().count() <= 11);
    }

    #[test]
    fn detect_hostile_wallet_connect() {
        let e = engine();
        let score = e.detect_hostile("Please connect wallet to verify now");
        assert!(score > 0.0, "should detect hostile content");
    }

    #[test]
    fn detect_hostile_clean_content() {
        let e     = engine();
        let score = e.detect_hostile("This is a regular article about technology.");
        assert_eq!(score, 0.0, "clean content should score 0");
    }

    #[test]
    fn prompt_to_tensor_correct_shape() {
        let e = engine();
        let t = e.prompt_to_tensor("test prompt").unwrap();
        assert_eq!(t.numel(), 8);
    }

    #[test]
    fn model_used_preserved_in_result() {
        let e   = engine();
        let req = InferenceRequest {
            prompt:     "test".to_string(),
            max_tokens: 10,
            model:      OnDeviceModel::Micro,
        };
        let result = e.infer(req).unwrap();
        assert_eq!(result.model_used, OnDeviceModel::Micro);
    }
}
