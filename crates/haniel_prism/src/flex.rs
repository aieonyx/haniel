// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// haniel_prism::flex — Flexbox helpers (wraps axon_layout flex)

use axon_layout::{LayoutNode, LayoutStyle, BoxModel, Direction, Align};
use crate::PrismError;

/// Flex container builder — creates axon_layout flex nodes
pub struct FlexBuilder;

impl FlexBuilder {
    /// Create a flex row container
    pub fn row(id: &str, w: f32, h: f32, gap: f32) -> Result<LayoutNode, PrismError> {
        let model = BoxModel::default_with_content(w, h)
            .map_err(|_| PrismError::LayoutOverflow)?;
        Ok(LayoutNode::new(id, model, LayoutStyle::row().with_gap(gap)))
    }

    /// Create a flex column container
    pub fn column(id: &str, w: f32, h: f32, gap: f32) -> Result<LayoutNode, PrismError> {
        let model = BoxModel::default_with_content(w, h)
            .map_err(|_| PrismError::LayoutOverflow)?;
        Ok(LayoutNode::new(id, model, LayoutStyle::column().with_gap(gap)))
    }

    /// Create a flex row with centered alignment
    pub fn row_centered(id: &str, w: f32, h: f32) -> Result<LayoutNode, PrismError> {
        let model = BoxModel::default_with_content(w, h)
            .map_err(|_| PrismError::LayoutOverflow)?;
        Ok(LayoutNode::new(
            id, model,
            LayoutStyle::row().with_align(Align::Center)
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axon_layout::{compute_layout, Point, Size};

    #[test]
    fn flex_row_constructs() {
        let node = FlexBuilder::row("flex", 1280.0, 100.0, 8.0).unwrap();
        assert_eq!(node.id, "flex");
    }

    #[test]
    fn flex_column_constructs() {
        let node = FlexBuilder::column("col", 1280.0, 720.0, 0.0).unwrap();
        assert_eq!(node.id, "col");
    }

    #[test]
    fn flex_row_layout_correct() {
        let mut row = FlexBuilder::row("row", 1000.0, 100.0, 0.0).unwrap();
        let child1  = FlexBuilder::row("c1", 500.0, 100.0, 0.0).unwrap();
        let child2  = FlexBuilder::row("c2", 500.0, 100.0, 0.0).unwrap();
        row.add_child(child1);
        row.add_child(child2);
        let layout = compute_layout(
            &row,
            Point::zero(),
            Size::new(1000.0, 100.0).unwrap()
        ).unwrap();
        assert_eq!(layout.children.len(), 2);
        let x1 = layout.children[0].rect.min_x();
        let x2 = layout.children[1].rect.min_x();
        assert!(x2 >= x1);
    }

    #[test]
    fn flex_column_stacks_vertically() {
        let mut col = FlexBuilder::column("col", 1280.0, 200.0, 0.0).unwrap();
        col.add_child(FlexBuilder::column("r1", 1280.0, 100.0, 0.0).unwrap());
        col.add_child(FlexBuilder::column("r2", 1280.0, 100.0, 0.0).unwrap());
        let layout = compute_layout(
            &col,
            Point::zero(),
            Size::new(1280.0, 200.0).unwrap()
        ).unwrap();
        let y1 = layout.children[0].rect.min_y();
        let y2 = layout.children[1].rect.min_y();
        assert!(y2 >= y1);
    }

    #[test]
    fn flex_gap_offsets_children() {
        let mut row = FlexBuilder::row("row", 1000.0, 100.0, 20.0).unwrap();
        row.add_child(FlexBuilder::row("c1", 100.0, 100.0, 0.0).unwrap());
        row.add_child(FlexBuilder::row("c2", 100.0, 100.0, 0.0).unwrap());
        let layout = compute_layout(
            &row,
            Point::zero(),
            Size::new(1000.0, 100.0).unwrap()
        ).unwrap();
        let x1 = layout.children[0].rect.max_x();
        let x2 = layout.children[1].rect.min_x();
        // Gap of 20 between children
        assert!(x2 >= x1, "second child should start after first");
    }

    #[test]
    fn flex_row_centered_constructs() {
        let node = FlexBuilder::row_centered("centered", 800.0, 60.0).unwrap();
        assert_eq!(node.id, "centered");
    }
}
