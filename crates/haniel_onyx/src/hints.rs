// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// haniel_onyx::hints — Render hint generation
// Feeds SRB (Sovereign Render Budget) weights from AI classification

use crate::{RenderHint, SemanticClass};

/// Render hint generator — maps classifications to SRB weights
pub struct HintGenerator;

impl HintGenerator {
    pub fn new() -> Self { Self }

    /// Generate render hints from node classifications
    pub fn generate(
        &self,
        nodes: &[(u32, SemanticClass)],
    ) -> Vec<RenderHint> {
        nodes.iter().map(|(node, class)| {
            let (priority, confidence) = self.priority_for(class);
            RenderHint {
                node:          *node,
                class:         class.clone(),
                priority,
                ai_confidence: confidence,
            }
        }).collect()
    }

    /// Priority and confidence for each semantic class
    fn priority_for(&self, class: &SemanticClass) -> (f32, f32) {
        match class {
            SemanticClass::Sovereign  => (1.0, 0.99),
            SemanticClass::Article    => (0.9, 0.85),
            SemanticClass::Media      => (0.85, 0.80),
            SemanticClass::Form       => (0.7, 0.75),
            SemanticClass::Navigation => (0.5, 0.70),
            SemanticClass::Unknown    => (0.5, 0.50),
            SemanticClass::Threat     => (0.1, 0.95),
        }
    }

    /// Find highest priority hint
    pub fn highest_priority<'a>(&self, hints: &'a [RenderHint]) -> Option<&'a RenderHint> {
        hints.iter().max_by(|a, b| {
            a.priority.partial_cmp(&b.priority).unwrap()
        })
    }

    /// Find threat hints
    pub fn threats<'a>(&self, hints: &'a [RenderHint]) -> Vec<&'a RenderHint> {
        hints.iter().filter(|h| h.class == SemanticClass::Threat).collect()
    }

    /// Total render budget (sum of all priorities)
    pub fn total_budget(&self, hints: &[RenderHint]) -> f32 {
        hints.iter().map(|h| h.priority).sum()
    }
}

impl Default for HintGenerator {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gen() -> HintGenerator { HintGenerator::new() }

    #[test]
    fn generate_hints_count() {
        let g = gen();
        let hints = g.generate(&[
            (1, SemanticClass::Article),
            (2, SemanticClass::Navigation),
            (3, SemanticClass::Threat),
        ]);
        assert_eq!(hints.len(), 3);
    }

    #[test]
    fn sovereign_gets_highest_priority() {
        let g = gen();
        let hints = g.generate(&[(1, SemanticClass::Sovereign)]);
        assert_eq!(hints[0].priority, 1.0);
        assert_eq!(hints[0].ai_confidence, 0.99);
    }

    #[test]
    fn threat_gets_lowest_priority() {
        let g = gen();
        let hints = g.generate(&[(1, SemanticClass::Threat)]);
        assert_eq!(hints[0].priority, 0.1);
    }

    #[test]
    fn highest_priority_finds_sovereign() {
        let g = gen();
        let hints = g.generate(&[
            (1, SemanticClass::Navigation),
            (2, SemanticClass::Sovereign),
            (3, SemanticClass::Article),
        ]);
        let top = g.highest_priority(&hints).unwrap();
        assert_eq!(top.class, SemanticClass::Sovereign);
    }

    #[test]
    fn threats_filter_correctly() {
        let g = gen();
        let hints = g.generate(&[
            (1, SemanticClass::Article),
            (2, SemanticClass::Threat),
            (3, SemanticClass::Threat),
        ]);
        assert_eq!(g.threats(&hints).len(), 2);
    }

    #[test]
    fn total_budget_sums() {
        let g = gen();
        let hints = g.generate(&[
            (1, SemanticClass::Article),    // 0.9
            (2, SemanticClass::Navigation), // 0.5
        ]);
        let total = g.total_budget(&hints);
        assert!((total - 1.4).abs() < 0.01);
    }
}
