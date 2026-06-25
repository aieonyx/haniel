// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// HANIEL PRISM — Sovereign Layout Engine
// HE-4: .axbw parser + rod pass via axon_layout
// HE-5: cone pass + flexbox + text metrics + HTML subset

#![forbid(unsafe_code)]

pub mod axbw;
pub mod cone;
pub mod flex;
pub mod html;
pub mod layout;
pub mod srb;
pub mod text;

pub use axbw::{AxbwParser, AxbwBuilder, axbw_to_layout_node};
pub use cone::ConeLayout;
pub use flex::FlexBuilder;
pub use html::HtmlParser;
pub use layout::RodLayout;
pub use srb::SrbAllocator;
pub use text::TextLayout;

// Re-export axon_layout types used throughout HANIEL
pub use axon_layout::{
    LayoutNode, LayoutStyle, ComputedLayout,
    Rect, Align, find_node,
    TextStyle, TextMetrics, measure_text, break_lines,
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
    Cone,  // full detail — real text metrics, assets
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
    fn parse_html(&self, html: &str) -> Result<AxbwTree, PrismError>;
    fn rod_pass(&self, tree: &AxbwTree, vw: f32, vh: f32) -> Result<ComputedLayout, PrismError>;
    fn cone_pass(&self, tree: &AxbwTree, vw: f32, vh: f32) -> Result<ComputedLayout, PrismError>;
}

/// Sovereign PRISM implementation
pub struct SovereignPrism {
    parser:      AxbwParser,
    html_parser: HtmlParser,
    rod:         RodLayout,
    cone:        ConeLayout,
}

impl SovereignPrism {
    pub fn new() -> Self {
        Self {
            parser:      AxbwParser::new(),
            html_parser: HtmlParser::new(),
            rod:         RodLayout::new(),
            cone:        ConeLayout::new(),
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

    fn parse_html(&self, html: &str) -> Result<AxbwTree, PrismError> {
        self.html_parser.parse(html)
    }

    fn rod_pass(&self, tree: &AxbwTree, vw: f32, vh: f32)
        -> Result<ComputedLayout, PrismError>
    {
        let root_node = axbw_to_layout_node(tree, tree.root_id, vw, vh)?;
        self.rod.compute(&root_node, vw, vh)
    }

    fn cone_pass(&self, tree: &AxbwTree, vw: f32, vh: f32)
        -> Result<ComputedLayout, PrismError>
    {
        self.cone.compute(tree, vw, vh)
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
    fn prism_cone_pass_returns_layout() {
        let data   = AxbwBuilder::new(0).add_block(0, 1.0, &[]).build();
        let tree   = prism().parse_axbw(&data).unwrap();
        let layout = prism().cone_pass(&tree, 1280.0, 720.0).unwrap();
        assert_eq!(layout.id, "0");
    }

    #[test]
    fn prism_parse_html_div() {
        let tree = prism().parse_html("<div><p>Sovereign</p></div>").unwrap();
        assert!(!tree.nodes.is_empty());
    }

    #[test]
    fn prism_rod_then_cone_consistent() {
        let data = AxbwBuilder::new(0)
            .add_block(0, 1.0, &[1])
            .add_text(1, 0.8, "Hello")
            .build();
        let tree   = prism().parse_axbw(&data).unwrap();
        let rod    = prism().rod_pass(&tree, 1280.0, 720.0).unwrap();
        let cone   = prism().cone_pass(&tree, 1280.0, 720.0).unwrap();
        // Both should have same root id
        assert_eq!(rod.id, cone.id);
        // Both should have children
        assert_eq!(rod.children.len(), cone.children.len());
    }

    #[test]
    fn prism_parse_rejects_bad_axbw() {
        let mut data = AxbwBuilder::new(0).add_block(0, 1.0, &[]).build();
        data[0] = 0xFF;
        assert!(matches!(prism().parse_axbw(&data), Err(PrismError::InvalidAxbw(_))));
    }

    #[test]
    fn axbw_node_kinds_complete() {
        let _ = AxbwNodeKind::Block;
        let _ = AxbwNodeKind::Inline;
        let _ = AxbwNodeKind::Flex;
        let _ = AxbwNodeKind::Text("hi".into());
        let _ = AxbwNodeKind::Image { src: "x".into(), aspect: 1.78 };
        let _ = AxbwNodeKind::Sovereign;
    }
}
