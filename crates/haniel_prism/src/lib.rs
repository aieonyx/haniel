// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// HANIEL PRISM — Sovereign Layout Engine
// HE-4 (rod pass) + HE-5 (cone pass) implementation target

#![forbid(unsafe_code)]

pub mod axbw;
pub mod layout;
pub mod flex;
pub mod text;
pub mod srb;

pub type NodeId = u32;

/// A node in the layout tree
#[derive(Debug, Clone)]
pub struct LayoutNode {
    pub id: NodeId,
    pub kind: NodeKind,
    pub box_model: BoxModel,
    pub children: Vec<NodeId>,
    pub render_budget: f32,
}

/// Box model dimensions
#[derive(Debug, Clone, Default)]
pub struct BoxModel {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub margin: Edges,
    pub padding: Edges,
}

/// Edge insets
#[derive(Debug, Clone, Default)]
pub struct Edges {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

/// Node display type
#[derive(Debug, Clone)]
pub enum NodeKind {
    Block,
    Inline,
    Flex,
    Text(String),
    Image { src: String, aspect: f32 },
    Sovereign,
}

/// Render pass selector
#[derive(Debug, Clone, PartialEq)]
pub enum RenderPass {
    Rod,
    Cone,
}

/// Viewport dimensions
#[derive(Debug, Clone)]
pub struct Viewport {
    pub width: u32,
    pub height: u32,
    pub device_pixel_ratio: f32,
    pub fovea_x: f32,
    pub fovea_y: f32,
}

/// The full layout tree
#[derive(Debug)]
pub struct LayoutTree {
    pub root: NodeId,
    pub nodes: Vec<LayoutNode>,
    pub viewport: Viewport,
    pub pass: RenderPass,
}

/// PRISM layout engine trait
pub trait Prism: Send + Sync {
    fn parse_axbw(&self, data: &[u8]) -> Result<LayoutTree, PrismError>;
    fn parse_html(&self, html: &str) -> Result<LayoutTree, PrismError>;
    fn rod_pass(&self, tree: &mut LayoutTree);
    fn cone_pass(&self, tree: &mut LayoutTree);
    fn update_render_budget(&self, tree: &mut LayoutTree, fovea_x: f32, fovea_y: f32);
    fn invalidate(&self, tree: &mut LayoutTree, node: NodeId);
}

/// PRISM error type
#[derive(Debug)]
pub enum PrismError {
    InvalidAxbw(String),
    InvalidHtml(String),
    LayoutOverflow,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_tree_constructs() {
        let tree = LayoutTree {
            root: 0,
            nodes: vec![LayoutNode {
                id: 0,
                kind: NodeKind::Block,
                box_model: BoxModel::default(),
                children: vec![],
                render_budget: 1.0,
            }],
            viewport: Viewport {
                width: 1280,
                height: 720,
                device_pixel_ratio: 1.0,
                fovea_x: 640.0,
                fovea_y: 360.0,
            },
            pass: RenderPass::Rod,
        };
        assert_eq!(tree.root, 0);
        assert_eq!(tree.nodes.len(), 1);
    }

    #[test]
    fn render_pass_variants_exist() {
        assert_ne!(RenderPass::Rod, RenderPass::Cone);
    }

    #[test]
    fn box_model_default_is_zero() {
        let bm = BoxModel::default();
        assert_eq!(bm.x, 0.0);
        assert_eq!(bm.width, 0.0);
    }
}
