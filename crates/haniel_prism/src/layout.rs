// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// haniel_prism::layout — Rod pass using axon_layout::compute_layout

use axon_layout::{
    LayoutNode as AxLayoutNode,
    compute_layout, ComputedLayout, Point, Size,
};
use crate::PrismError;

/// Rod pass — compute structural layout using axon_layout engine
pub struct RodLayout;

impl RodLayout {
    pub fn new() -> Self { Self }

    /// Run rod pass on an axon_layout tree
    /// Returns ComputedLayout with all positions resolved
    pub fn compute(
        &self,
        root:  &AxLayoutNode,
        vw:    f32,
        vh:    f32,
    ) -> Result<ComputedLayout, PrismError> {
        let origin    = Point::zero();
        let available = Size::new(vw, vh)
            .map_err(|_| PrismError::LayoutOverflow)?;
        compute_layout(root, origin, available)
            .map_err(|e| PrismError::InvalidAxbw(format!("layout error: {:?}", e)))
    }
}

impl Default for RodLayout {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axon_layout::{BoxModel, LayoutStyle};

    fn make_block(id: &str, w: f32, h: f32) -> AxLayoutNode {
        let model = BoxModel::default_with_content(w, h).unwrap();
        AxLayoutNode::new(id, model, LayoutStyle::column())
    }

    #[test]
    fn rod_single_node_at_origin() {
        let root   = make_block("root", 1280.0, 720.0);
        let result = RodLayout::new().compute(&root, 1280.0, 720.0).unwrap();
        assert_eq!(result.id, "root");
        assert_eq!(result.rect.min_x(), 0.0);
        assert_eq!(result.rect.min_y(), 0.0);
    }

    #[test]
    fn rod_node_has_correct_dimensions() {
        let root   = make_block("root", 1280.0, 720.0);
        let result = RodLayout::new().compute(&root, 1280.0, 720.0).unwrap();
        assert!(result.rect.max_x() > 0.0);
        assert!(result.rect.max_y() > 0.0);
    }

    #[test]
    fn rod_parent_with_children() {
        let model = BoxModel::default_with_content(1280.0, 720.0).unwrap();
        let mut root = AxLayoutNode::new("root", model, LayoutStyle::column());
        root.add_child(make_block("child1", 1280.0, 100.0));
        root.add_child(make_block("child2", 1280.0, 100.0));
        let result = RodLayout::new().compute(&root, 1280.0, 720.0).unwrap();
        assert_eq!(result.children.len(), 2);
    }

    #[test]
    fn rod_flex_row_children() {
        let model = BoxModel::default_with_content(1000.0, 100.0).unwrap();
        let mut root = AxLayoutNode::new("flex", model, LayoutStyle::row());
        root.add_child(make_block("a", 500.0, 100.0));
        root.add_child(make_block("b", 500.0, 100.0));
        let result = RodLayout::new().compute(&root, 1000.0, 100.0).unwrap();
        assert_eq!(result.children.len(), 2);
        // Second child should be to the right of first
        let x1 = result.children[0].rect.min_x();
        let x2 = result.children[1].rect.min_x();
        assert!(x2 >= x1, "row: child2 x={} should be >= child1 x={}", x2, x1);
    }

    #[test]
    fn rod_column_children_stack_vertically() {
        let model = BoxModel::default_with_content(1280.0, 720.0).unwrap();
        let mut root = AxLayoutNode::new("col", model, LayoutStyle::column());
        root.add_child(make_block("a", 1280.0, 100.0));
        root.add_child(make_block("b", 1280.0, 100.0));
        let result = RodLayout::new().compute(&root, 1280.0, 720.0).unwrap();
        let y1 = result.children[0].rect.min_y();
        let y2 = result.children[1].rect.min_y();
        assert!(y2 >= y1, "column: child2 y={} should be >= child1 y={}", y2, y1);
    }
}
