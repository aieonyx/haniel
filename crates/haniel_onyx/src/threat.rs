// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// haniel_onyx::threat — AI-powered threat content classifier

use crate::SemanticClass;

/// Threat classifier — detects hostile AI-generated content
pub struct ThreatClassifier;

impl ThreatClassifier {
    pub fn new() -> Self { Self }

    /// Classify content — returns SemanticClass::Threat if hostile
    pub fn classify(&self, content: &str) -> SemanticClass {
        let score = self.threat_score(content);
        if score > 0.3 {
            SemanticClass::Threat
        } else {
            SemanticClass::Unknown
        }
    }

    /// Compute threat score 0.0-1.0
    /// Each hostile keyword hit = 0.2, capped at 1.0
    pub fn threat_score(&self, content: &str) -> f32 {
        let lower = content.to_lowercase();
        let hostile_keywords = [
            "connect wallet", "verify account", "claim reward",
            "limited time", "act now", "you have been selected",
            "send crypto", "seed phrase", "private key",
            "urgent action", "account suspended", "winner",
        ];
        let mut hits = 0usize;
        for kw in &hostile_keywords {
            if lower.contains(kw) { hits += 1; }
        }
        (hits as f32 * 0.2).min(1.0)
    }
}

impl Default for ThreatClassifier {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clf() -> ThreatClassifier { ThreatClassifier::new() }

    #[test]
    fn classify_hostile_wallet() {
        let c = clf();
        assert_eq!(
            c.classify("Connect wallet now to claim reward"),
            SemanticClass::Threat
        );
    }

    #[test]
    fn classify_clean_article() {
        let c = clf();
        let result = c.classify("This is a regular news article about technology.");
        assert_ne!(result, SemanticClass::Threat);
    }

    #[test]
    fn threat_score_hostile_high() {
        let c     = clf();
        let score = c.threat_score("Connect wallet verify account claim reward limited time act now");
        assert!(score > 0.0, "hostile content should have positive score");
    }

    #[test]
    fn threat_score_clean_low() {
        let c     = clf();
        let score = c.threat_score("A thoughtful article about distributed systems.");
        assert!(score <= 0.3, "clean content score should be low");
    }
}
