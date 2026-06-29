// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// haniel_onyx::classify — Semantic page classification
// Uses axon_ai_runtime tensor ops for content classification

use axon_ai_runtime::{Tensor, ops};
use crate::{SemanticClass, RenderHint};

/// Sovereign semantic classifier
/// Classifies page nodes into semantic categories
/// Uses lightweight feedforward network
pub struct SemanticClassifier;

impl SemanticClassifier {
    pub fn new() -> Self { Self }

    /// Classify text content into semantic category
    /// HE-11: heuristic classifier — full neural at HE-11 completion
    pub fn classify_text(&self, text: &str) -> SemanticClass {
        let lower = text.to_lowercase();

        // Navigation patterns
        if lower.contains("menu") || lower.contains("nav")
            || lower.contains("home") || lower.contains("about")
            || lower.contains("contact") || lower.len() < 20
        {
            return SemanticClass::Navigation;
        }

        // Threat patterns
        if lower.contains("connect wallet") || lower.contains("verify now")
            || lower.contains("claim your") || lower.contains("limited time")
            || lower.contains("act now") || lower.contains("winner")
        {
            return SemanticClass::Threat;
        }

        // Sovereign content
        if lower.contains("awp://") || lower.contains("aieonyx")
            || lower.contains("sovereign")
        {
            return SemanticClass::Sovereign;
        }

        // Media patterns
        if lower.contains("video") || lower.contains("audio")
            || lower.contains("watch") || lower.contains("play")
            || lower.contains("stream")
        {
            return SemanticClass::Media;
        }

        // Form patterns
        if lower.contains("submit") || lower.contains("email")
            || lower.contains("password") || lower.contains("login")
            || lower.contains("sign up") || lower.contains("register")
        {
            return SemanticClass::Form;
        }

        // Article — long text content
        if text.len() > 200 {
            return SemanticClass::Article;
        }

        SemanticClass::Unknown
    }

    /// Extract feature vector from text for neural classification
    pub fn extract_features(&self, text: &str) -> Tensor {
        let lower    = text.to_lowercase();
        let len_norm = (text.len() as f32 / 1000.0).min(1.0);

        let features = vec![
            len_norm,
            if lower.contains("nav") || lower.contains("menu") { 1.0 } else { 0.0 },
            if lower.contains("article") || lower.contains("post") { 1.0 } else { 0.0 },
            if lower.contains("video") || lower.contains("play") { 1.0 } else { 0.0 },
            if lower.contains("form") || lower.contains("input") { 1.0 } else { 0.0 },
            if lower.contains("wallet") || lower.contains("claim") { 1.0 } else { 0.0 },
            if lower.contains("sovereign") || lower.contains("awp") { 1.0 } else { 0.0 },
            if text.len() > 200 { 1.0 } else { 0.0 },
        ];

        Tensor::from_vec(features).unwrap_or_else(|_| Tensor::zeros(vec![8]).unwrap())
    }

    /// Compute confidence score for a classification
    pub fn confidence(&self, features: &Tensor, class: &SemanticClass) -> f32 {
        // Apply softmax to features to get probability distribution
        if let Ok(probs) = ops::softmax(features) {
            let class_idx = match class {
                SemanticClass::Navigation => 1,
                SemanticClass::Article    => 2,
                SemanticClass::Media      => 3,
                SemanticClass::Form       => 4,
                SemanticClass::Threat     => 5,
                SemanticClass::Sovereign  => 6,
                SemanticClass::Unknown    => 0,
            };
            probs.data.get(class_idx % probs.numel()).copied().unwrap_or(0.0).abs()
        } else {
            0.5
        }
    }

    /// Generate render hints for a list of node classifications
    pub fn to_render_hints(
        &self,
        classifications: Vec<(u32, SemanticClass)>,
    ) -> Vec<RenderHint> {
        classifications.into_iter().map(|(node, class)| {
            let priority = match &class {
                SemanticClass::Article   => 0.9,
                SemanticClass::Media     => 0.85,
                SemanticClass::Sovereign => 1.0,
                SemanticClass::Form      => 0.7,
                SemanticClass::Navigation => 0.5,
                SemanticClass::Threat    => 0.1,
                SemanticClass::Unknown   => 0.5,
            };
            RenderHint { node, class, priority, ai_confidence: 0.75 }
        }).collect()
    }
}

impl Default for SemanticClassifier {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clf() -> SemanticClassifier { SemanticClassifier::new() }

    #[test]
    fn classify_navigation_text() {
        let c = clf();
        assert_eq!(c.classify_text("Home About Contact Menu"), SemanticClass::Navigation);
    }

    #[test]
    fn classify_threat_text() {
        let c = clf();
        assert_eq!(c.classify_text("Connect wallet now to claim your prize!"), SemanticClass::Threat);
    }

    #[test]
    fn classify_sovereign_text() {
        let c = clf();
        assert_eq!(c.classify_text("awp://aegis sovereign dashboard"), SemanticClass::Sovereign);
    }

    #[test]
    fn classify_media_text() {
        let c = clf();
        assert_eq!(c.classify_text("Watch the latest video stream now"), SemanticClass::Media);
    }

    #[test]
    fn classify_form_text() {
        let c = clf();
        assert_eq!(c.classify_text("Enter your email and password to login"), SemanticClass::Form);
    }

    #[test]
    fn classify_article_long_text() {
        let c    = clf();
        let long = "a ".repeat(120); // 240 chars
        assert_eq!(c.classify_text(&long), SemanticClass::Article);
    }

    #[test]
    fn extract_features_correct_dim() {
        let c        = clf();
        let features = c.extract_features("Hello sovereign world");
        assert_eq!(features.numel(), 8);
    }

    #[test]
    fn extract_features_sovereign_flag() {
        let c = clf();
        let f = c.extract_features("sovereign content");
        // index 6 = sovereign flag
        assert_eq!(f.data[6], 1.0);
    }

    #[test]
    fn extract_features_threat_flag() {
        let c = clf();
        let f = c.extract_features("connect wallet claim");
        // index 5 = threat flag
        assert_eq!(f.data[5], 1.0);
    }

    #[test]
    fn render_hints_priority_sovereign_highest() {
        let c     = clf();
        let hints = c.to_render_hints(vec![
            (1, SemanticClass::Sovereign),
            (2, SemanticClass::Threat),
        ]);
        assert_eq!(hints[0].priority, 1.0);
        assert_eq!(hints[1].priority, 0.1);
    }

    #[test]
    fn render_hints_count_matches() {
        let c = clf();
        let hints = c.to_render_hints(vec![
            (1, SemanticClass::Article),
            (2, SemanticClass::Navigation),
            (3, SemanticClass::Media),
        ]);
        assert_eq!(hints.len(), 3);
    }
}
