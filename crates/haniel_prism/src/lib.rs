// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// HANIEL PRISM — Sovereign Layout Engine
// HE-4: .axbw parser + rod pass via axon_layout

#![forbid(unsafe_code)]

pub mod axbw;
pub mod layout;
pub mod flex;
pub mod text;
pub mod srb;

pub use axbw::{AxbwParser, AxbwBuilder, axbw_to_layout_node};
pub use layout::RodLayout;
pub use srb::SrbAllocator;

// Re-export axon_layout types used throughout HANIEL
pub use axon_layout::{
    LayoutNode, LayoutStyle, ComputedLayout,
    BoxModel, EdgeSizes, Point, Rect, Size,
    Direction, Align, compute_layout,
    TextStyle, TextMetrics, measure_text,
};

/// AXBW parsed node
#[derive(Debug, Clone)]
pub struct AxbwNode {
    pub id:            u32,
    pub kind:          AxbwNodeKind,
    pub child_ids:     Vec<u32>,
    pub render_budget: f32,
}

/// AXBW node kind
#[derive(Debug, Clone)]
pub enum AxbwNodeKind {
    Block,
    Inline,
    Flex,
    Text(String),
    Image { src: String, aspect: f32 },
    Sovereign,
}

/// Parsed AXBW tree (binary representation)
#[derive(Debug)]
pub struct AxbwTree {
    pub root_id: u32,
    pub nodes:   Vec<AxbwNode>,
}

/// Render pass selector
#[derive(Debug, Clone, PartialEq)]
pub enum RenderPass {
    Rod,   // structural skeleton — immediate
    Cone,  // full detail — cone pass at HE-5
}

/// PRISM error type
#[derive(Debug)]
pub enum PrismError {
    InvalidAxbw(String),
    InvalidHtml(String),
    LayoutOverflow,
}

/// PRISM sovereign layout trait
pub trait Prism: Send + Sync {
    fn parse_axbw(&self, data: &[u8]) -> Result<AxbwTree, PrismError>;
    fn rod_pass(
        &self,
        tree: &AxbwTree,
        vw:   f32,
        vh:   f32,
    ) -> Result<ComputedLayout, PrismError>;
}

/// Sovereign PRISM implementation
pub struct SovereignPrism {
    parser: AxbwParser,
    rod:    RodLayout,
}

impl SovereignPrism {
    pub fn new() -> Self {
        Self {
            parser: AxbwParser::new(),
            rod:    RodLayout::new(),
        }
    }
}

impl Default for SovereignPrism {
    fn default() -> Self { Self::new() }
}

impl Prism for SovereignPrism {
    fn parse_axbw(&self, data: &[u8]) -> Result<AxbwTree, PrismError> {
        self.parser.parse(data)
    }

    fn rod_pass(
        &self,
        tree: &AxbwTree,
        vw:   f32,
        vh:   f32,
    ) -> Result<ComputedLayout, PrismError> {
        let root_node = axbw_to_layout_node(tree, tree.root_id, vw, vh)?;
        self.rod.compute(&root_node, vw, vh)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn prism() -> SovereignPrism { SovereignPrism::new() }

    #[test]
    fn prism_parse_axbw_single_block() {
        let data = AxbwBuilder::new(0).add_block(0, 1.0, &[]).build();
        let tree = prism().parse_axbw(&data).unwrap();
        assert_eq!(tree.root_id, 0);
        assert_eq!(tree.nodes.len(), 1);
    }

    #[test]
    fn prism_rod_pass_returns_layout() {
        let data   = AxbwBuilder::new(0).add_block(0, 1.0, &[]).build();
        let tree   = prism().parse_axbw(&data).unwrap();
        let layout = prism().rod_pass(&tree, 1280.0, 720.0).unwrap();
        assert_eq!(layout.id, "0");
    }

    #[test]
    fn prism_rod_pass_origin_correct() {
        let data   = AxbwBuilder::new(0).add_block(0, 1.0, &[]).build();
        let tree   = prism().parse_axbw(&data).unwrap();
        let layout = prism().rod_pass(&tree, 1280.0, 720.0).unwrap();
        assert_eq!(layout.rect.min_x(), 0.0);
        assert_eq!(layout.rect.min_y(), 0.0);
    }

    #[test]
    fn prism_rod_pass_with_children() {
        let data = AxbwBuilder::new(0)
            .add_block(0, 1.0, &[1, 2])
            .add_block(1, 0.5, &[])
            .add_block(2, 0.5, &[])
            .build();
        let tree   = prism().parse_axbw(&data).unwrap();
        let layout = prism().rod_pass(&tree, 1280.0, 720.0).unwrap();
        assert_eq!(layout.children.len(), 2);
    }

    #[test]
    fn prism_parse_rejects_bad_magic() {
        let mut data = AxbwBuilder::new(0).add_block(0, 1.0, &[]).build();
        data[0] = 0xFF;
        assert!(matches!(prism().parse_axbw(&data), Err(PrismError::InvalidAxbw(_))));
    }

    #[test]
    fn axbw_node_kinds_exist() {
        let _ = AxbwNodeKind::Block;
        let _ = AxbwNodeKind::Flex;
        let _ = AxbwNodeKind::Text("hi".into());
        let _ = AxbwNodeKind::Image { src: "x".into(), aspect: 1.0 };
        let _ = AxbwNodeKind::Sovereign;
    }
}
