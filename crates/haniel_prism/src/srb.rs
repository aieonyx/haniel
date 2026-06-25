// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// haniel_prism::srb — Sovereign Render Budget (TERM-051)

use axon_layout::{ComputedLayout, Point};

/// Sovereign Render Budget allocator
/// Weights render budget by proximity to fovea (TERM-051)
pub struct SrbAllocator {
    pub fovea: Point,
    pub viewport_w: f32,
    pub viewport_h: f32,
}

impl SrbAllocator {
    pub fn new(fovea_x: f32, fovea_y: f32, vw: f32, vh: f32) -> Self {
        Self {
            fovea: Point::new(fovea_x, fovea_y),
            viewport_w: vw,
            viewport_h: vh,
        }
    }

    /// Compute SRB weight for a computed layout node
    /// Returns value 0.1 (far) to 1.0 (fovea center)
    pub fn weight(&self, layout: &ComputedLayout) -> f32 {
        let center  = layout.rect.center();
        let dist    = center.distance_to(&self.fovea);
        let max_d   = (self.viewport_w * self.viewport_w
            + self.viewport_h * self.viewport_h).sqrt();
        let budget  = 1.0 - (dist / max_d).min(0.9);
        budget.max(0.1)
    }

    /// Compute SRB weights for all nodes in a computed layout tree
    pub fn weights_for(&self, layout: &ComputedLayout) -> Vec<(String, f32)> {
        let mut result = Vec::new();
        self.collect_weights(layout, &mut result);
        result
    }

    fn collect_weights(&self, layout: &ComputedLayout, out: &mut Vec<(String, f32)>) {
        out.push((layout.id.clone(), self.weight(layout)));
        for child in &layout.children {
            self.collect_weights(child, out);
        }
    }

    /// Update fovea position
    pub fn set_fovea(&mut self, x: f32, y: f32) {
        self.fovea = Point::new(x, y);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axon_layout::{Rect, Point};

    fn make_layout(id: &str, x: f32, y: f32, w: f32, h: f32) -> ComputedLayout {
        ComputedLayout {
            id:       id.to_string(),
            rect:     Rect::new(x, y, w, h).unwrap(),
            children: vec![],
        }
    }

    #[test]
    fn srb_fovea_center_gets_high_budget() {
        let srb    = SrbAllocator::new(500.0, 500.0, 1000.0, 1000.0);
        let layout = make_layout("center", 490.0, 490.0, 20.0, 20.0);
        let w      = srb.weight(&layout);
        assert!(w > 0.9, "fovea node should get >0.9 budget, got {}", w);
    }

    #[test]
    fn srb_far_node_gets_low_budget() {
        let srb    = SrbAllocator::new(0.0, 0.0, 1000.0, 1000.0);
        let layout = make_layout("far", 900.0, 900.0, 50.0, 50.0);
        let w      = srb.weight(&layout);
        assert!(w < 0.5, "far node should get <0.5 budget, got {}", w);
    }

    #[test]
    fn srb_budget_always_in_range() {
        let srb = SrbAllocator::new(500.0, 500.0, 1000.0, 1000.0);
        for (x, y) in [(0.0,0.0),(500.0,500.0),(999.0,999.0),(250.0,750.0)] {
            let layout = make_layout("n", x, y, 10.0, 10.0);
            let w = srb.weight(&layout);
            assert!(w >= 0.1 && w <= 1.0, "budget {} out of range", w);
        }
    }

    #[test]
    fn srb_weights_for_collects_all() {
        let srb = SrbAllocator::new(500.0, 500.0, 1000.0, 1000.0);
        let layout = ComputedLayout {
            id:   "root".to_string(),
            rect: Rect::new(0.0, 0.0, 1000.0, 1000.0).unwrap(),
            children: vec![
                make_layout("child1", 100.0, 100.0, 50.0, 50.0),
                make_layout("child2", 800.0, 800.0, 50.0, 50.0),
            ],
        };
        let weights = srb.weights_for(&layout);
        assert_eq!(weights.len(), 3);
        assert_eq!(weights[0].0, "root");
    }

    #[test]
    fn srb_set_fovea_updates() {
        let mut srb = SrbAllocator::new(0.0, 0.0, 1000.0, 1000.0);
        srb.set_fovea(500.0, 500.0);
        assert_eq!(srb.fovea.x, 500.0);
        assert_eq!(srb.fovea.y, 500.0);
    }
}
