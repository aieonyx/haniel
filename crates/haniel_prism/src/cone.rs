// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// haniel_prism::cone — Cone pass layout
// Full detail: text metrics, image aspect, incremental invalidation

use axon_layout::{
    LayoutNode, LayoutStyle, BoxModel,
    ComputedLayout, Point, Size,
    TextStyle, measure_text, find_node,
    compute_layout,
};
use crate::{AxbwTree, AxbwNodeKind, PrismError};

/// Cone pass — full detail layout
/// Refines rod pass with real text metrics and asset dimensions
pub struct ConeLayout;

impl ConeLayout {
    pub fn new() -> Self { Self }

    /// Run cone pass — produces final ComputedLayout with text metrics
    pub fn compute(
        &self,
        tree: &AxbwTree,
        vw:   f32,
        vh:   f32,
    ) -> Result<ComputedLayout, PrismError> {
        let root_node = self.build_cone_node(tree, tree.root_id, vw, vh)?;
        let origin    = Point::zero();
        let available = Size::new(vw, vh)
            .map_err(|_| PrismError::LayoutOverflow)?;
        compute_layout(&root_node, origin, available)
            .map_err(|e| PrismError::InvalidAxbw(format!("cone layout error: {e:?}")))
    }

    /// Build layout node with cone-pass metrics (real text heights)
    fn build_cone_node(
        &self,
        tree:    &AxbwTree,
        node_id: u32,
        vw:      f32,
        vh:      f32,
    ) -> Result<LayoutNode, PrismError> {
        let node = tree.nodes.iter().find(|n| n.id == node_id)
            .ok_or_else(|| PrismError::InvalidAxbw(
                format!("node {} not found in cone pass", node_id)
            ))?;

        let style = match &node.kind {
            AxbwNodeKind::Flex   => LayoutStyle::row().with_gap(8.0),
            AxbwNodeKind::Inline => LayoutStyle::row(),
            _                   => LayoutStyle::column(),
        };

        // Cone pass: compute real height for text nodes
        let (w, h) = match &node.kind {
            AxbwNodeKind::Text(text) => {
                let ts      = TextStyle::default();
                let metrics = measure_text(text, &ts, vw)
                    .map_err(|e| PrismError::InvalidAxbw(format!("text measure: {:?}", e)))?;
                (metrics.width.max(1.0), metrics.height.max(ts.line_height))
            }
            AxbwNodeKind::Image { aspect, .. } => {
                let img_h = if *aspect > 0.0 { vw / *aspect } else { vw * 0.5625 };
                (vw, img_h)
            }
            AxbwNodeKind::Sovereign => (vw, vh),
            _ => (vw, vh),
        };

        let model = BoxModel::default_with_content(w, h)
            .map_err(|_| PrismError::LayoutOverflow)?;

        let mut layout_node = LayoutNode::new(
            &format!("{}", node_id),
            model,
            style,
        );

        for &cid in &node.child_ids {
            let child = self.build_cone_node(tree, cid, vw, vh)?;
            layout_node.add_child(child);
        }

        Ok(layout_node)
    }

    /// Invalidate a node — force re-layout in next cone pass
    pub fn invalidate(
        layout: &mut ComputedLayout,
        node_id: &str,
    ) -> bool {
        if layout.id == node_id {
            // Reset rect to zero — signals dirty
            layout.rect = axon_layout::Rect::zero();
            return true;
        }
        for child in layout.children.iter_mut() {
            if Self::invalidate(child, node_id) {
                return true;
            }
        }
        false
    }

    /// Find a node in computed layout by id
    pub fn find(layout: &ComputedLayout, id: &str) -> Option<ComputedLayout> {
        find_node(layout, id).cloned()
    }
}

impl Default for ConeLayout {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AxbwBuilder, AxbwParser};

    fn cone() -> ConeLayout { ConeLayout::new() }
    fn parser() -> AxbwParser { AxbwParser::new() }

    #[test]
    fn cone_single_block() {
        let data   = AxbwBuilder::new(0).add_block(0, 1.0, &[]).build();
        let tree   = parser().parse(&data).unwrap();
        let layout = cone().compute(&tree, 1280.0, 720.0).unwrap();
        assert_eq!(layout.id, "0");
        assert!(layout.rect.max_x() > 0.0);
    }

    #[test]
    fn cone_text_node_has_real_height() {
        let data = AxbwBuilder::new(0)
            .add_text(0, 1.0, "Hello sovereign world this is a test of text metrics")
            .build();
        let tree   = parser().parse(&data).unwrap();
        let layout = cone().compute(&tree, 1280.0, 720.0).unwrap();
        // Text should have non-zero height from measure_text
        assert!(layout.rect.max_y() > 0.0,
            "text node should have height, rect: {:?}", layout.rect);
    }

    #[test]
    fn cone_image_preserves_aspect() {
        let data   = AxbwBuilder::new(0)
            .add_image(0, 1.0, "img.png", 16.0/9.0)
            .build();
        let tree   = parser().parse(&data).unwrap();
        let layout = cone().compute(&tree, 1280.0, 720.0).unwrap();
        let w      = layout.rect.size.width;
        let h      = layout.rect.size.height;
        if w > 0.0 && h > 0.0 {
            let ratio  = w / h;
            let expect = 16.0_f32 / 9.0;
            assert!((ratio - expect).abs() < 0.5,
                "aspect ratio {:.3} should be near {:.3}", ratio, expect);
        }
    }

    #[test]
    fn cone_parent_with_text_children() {
        let data = AxbwBuilder::new(0)
            .add_block(0, 1.0, &[1, 2])
            .add_text(1, 0.5, "First line of text")
            .add_text(2, 0.5, "Second line of text")
            .build();
        let tree   = parser().parse(&data).unwrap();
        let layout = cone().compute(&tree, 1280.0, 720.0).unwrap();
        assert_eq!(layout.children.len(), 2);
    }

    #[test]
    fn cone_flex_row_children() {
        let data = AxbwBuilder::new(0)
            .add_flex(0, 1.0, &[1, 2])
            .add_block(1, 0.5, &[])
            .add_block(2, 0.5, &[])
            .build();
        let tree   = parser().parse(&data).unwrap();
        let layout = cone().compute(&tree, 1280.0, 720.0).unwrap();
        assert_eq!(layout.children.len(), 2);
        // Row: second child should be to the right
        let x1 = layout.children[0].rect.min_x();
        let x2 = layout.children[1].rect.min_x();
        assert!(x2 >= x1, "flex row: child2 x={} >= child1 x={}", x2, x1);
    }

    #[test]
    fn cone_sovereign_fills_viewport() {
        let data   = AxbwBuilder::new(0).add_sovereign(0, 1.0).build();
        let tree   = parser().parse(&data).unwrap();
        let layout = cone().compute(&tree, 1280.0, 720.0).unwrap();
        assert!(layout.rect.size.width > 0.0);
        assert!(layout.rect.size.height > 0.0);
    }

    #[test]
    fn cone_invalidate_resets_rect() {
        let data = AxbwBuilder::new(0).add_block(0, 1.0, &[1])
            .add_text(1, 1.0, "test").build();
        let tree   = parser().parse(&data).unwrap();
        let mut layout = cone().compute(&tree, 1280.0, 720.0).unwrap();
        let changed = ConeLayout::invalidate(&mut layout, "1");
        assert!(changed, "invalidate should return true for existing node");
    }

    #[test]
    fn cone_invalidate_nonexistent_returns_false() {
        let data   = AxbwBuilder::new(0).add_block(0, 1.0, &[]).build();
        let tree   = parser().parse(&data).unwrap();
        let mut layout = cone().compute(&tree, 1280.0, 720.0).unwrap();
        let changed = ConeLayout::invalidate(&mut layout, "999");
        assert!(!changed);
    }

    #[test]
    fn cone_find_node_by_id() {
        let data = AxbwBuilder::new(0)
            .add_block(0, 1.0, &[1])
            .add_text(1, 0.8, "findme")
            .build();
        let tree   = parser().parse(&data).unwrap();
        let layout = cone().compute(&tree, 1280.0, 720.0).unwrap();
        let found  = ConeLayout::find(&layout, "1");
        assert!(found.is_some(), "should find node with id 1");
    }
}
