// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// haniel_prism::axbw — Sovereign .axbw binary format parser
// Implements AXBW v1 spec (W0-A6)
// Uses axon_layout primitives natively

use axon_layout::{
    LayoutNode, LayoutStyle, BoxModel, EdgeSizes,
    compute_layout, Point, Size,
};
use crate::{AxbwTree, AxbwNode, AxbwNodeKind, PrismError};

/// AXBW magic bytes
const AXBW_MAGIC: &[u8; 4] = b"AXBW";
const AXBW_VERSION: u16 = 1;

const KIND_BLOCK:     u32 = 0;
const KIND_INLINE:    u32 = 1;
const KIND_FLEX:      u32 = 2;
const KIND_TEXT:      u32 = 3;
const KIND_IMAGE:     u32 = 4;
const KIND_SOVEREIGN: u32 = 5;

/// AXBW binary parser
pub struct AxbwParser;

impl AxbwParser {
    pub fn new() -> Self { Self }

    /// Parse .axbw binary into an AxbwTree
    pub fn parse(&self, data: &[u8]) -> Result<AxbwTree, PrismError> {
        if data.len() < 16 {
            return Err(PrismError::InvalidAxbw(
                "file too short — minimum 16 byte header required".into()
            ));
        }

        // Validate magic
        if &data[0..4] != AXBW_MAGIC {
            return Err(PrismError::InvalidAxbw(
                format!("invalid magic bytes: {:?}", &data[0..4])
            ));
        }

        // Version check
        let version = u16::from_le_bytes([data[4], data[5]]);
        if version != AXBW_VERSION {
            return Err(PrismError::InvalidAxbw(
                format!("unsupported version: {}", version)
            ));
        }

        let node_count = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
        let root_id    = u32::from_le_bytes([data[12], data[13], data[14], data[15]]);

        if node_count == 0 {
            return Err(PrismError::InvalidAxbw("node_count is 0".into()));
        }

        let mut nodes  = Vec::with_capacity(node_count as usize);
        let mut cursor = 16usize;

        for _ in 0..node_count {
            let (node, consumed) = self.parse_node(&data[cursor..], cursor)?;
            nodes.push(node);
            cursor += consumed;
        }

        // Validate root exists
        if !nodes.iter().any(|n| n.id == root_id) {
            return Err(PrismError::InvalidAxbw(
                format!("root_id {} not found", root_id)
            ));
        }

        Ok(AxbwTree { root_id, nodes })
    }

    fn parse_node(&self, data: &[u8], offset: usize) -> Result<(AxbwNode, usize), PrismError> {
        if data.len() < 16 {
            return Err(PrismError::InvalidAxbw(
                format!("node truncated at offset {}", offset)
            ));
        }

        let id          = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        let kind_disc   = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
        let child_count = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
        let budget_bits = u32::from_le_bytes([data[12], data[13], data[14], data[15]]);
        let budget      = f32::from_bits(budget_bits);
        let mut cursor  = 16usize;

        let kind = match kind_disc {
            KIND_BLOCK     => AxbwNodeKind::Block,
            KIND_INLINE    => AxbwNodeKind::Inline,
            KIND_FLEX      => AxbwNodeKind::Flex,
            KIND_SOVEREIGN => AxbwNodeKind::Sovereign,
            KIND_TEXT => {
                if data.len() < cursor + 4 {
                    return Err(PrismError::InvalidAxbw("text node truncated".into()));
                }
                let text_len = u32::from_le_bytes([
                    data[cursor], data[cursor+1], data[cursor+2], data[cursor+3]
                ]) as usize;
                cursor += 4;
                if data.len() < cursor + text_len {
                    return Err(PrismError::InvalidAxbw("text payload truncated".into()));
                }
                let text = String::from_utf8_lossy(&data[cursor..cursor+text_len]).to_string();
                cursor  += text_len;
                AxbwNodeKind::Text(text)
            }
            KIND_IMAGE => {
                if data.len() < cursor + 4 {
                    return Err(PrismError::InvalidAxbw("image node truncated".into()));
                }
                let src_len = u32::from_le_bytes([
                    data[cursor], data[cursor+1], data[cursor+2], data[cursor+3]
                ]) as usize;
                cursor += 4;
                if data.len() < cursor + src_len + 4 {
                    return Err(PrismError::InvalidAxbw("image payload truncated".into()));
                }
                let src    = String::from_utf8_lossy(&data[cursor..cursor+src_len]).to_string();
                cursor    += src_len;
                let aspect = f32::from_bits(u32::from_le_bytes([
                    data[cursor], data[cursor+1], data[cursor+2], data[cursor+3]
                ]));
                cursor += 4;
                AxbwNodeKind::Image { src, aspect }
            }
            _ => return Err(PrismError::InvalidAxbw(
                format!("unknown kind discriminant: {}", kind_disc)
            )),
        };

        // Parse child IDs
        let needed = child_count as usize * 4;
        if data.len() < cursor + needed {
            return Err(PrismError::InvalidAxbw("child_ids truncated".into()));
        }
        let mut child_ids = Vec::with_capacity(child_count as usize);
        for i in 0..child_count as usize {
            let b = cursor + i * 4;
            child_ids.push(u32::from_le_bytes([data[b], data[b+1], data[b+2], data[b+3]]));
        }
        cursor += needed;

        Ok((AxbwNode { id, kind, child_ids, render_budget: budget }, cursor))
    }
}

impl Default for AxbwParser {
    fn default() -> Self { Self::new() }
}

/// Build axon_layout::LayoutNode tree from parsed AxbwTree
/// This is the bridge between .axbw binary and axon_layout compute engine
pub fn axbw_to_layout_node(
    tree:    &AxbwTree,
    node_id: u32,
    vw:      f32,
    vh:      f32,
) -> Result<LayoutNode, PrismError> {
    let node = tree.nodes.iter().find(|n| n.id == node_id)
        .ok_or_else(|| PrismError::InvalidAxbw(format!("node {} not found", node_id)))?;

    let style = match &node.kind {
        AxbwNodeKind::Block | AxbwNodeKind::Text(_)
        | AxbwNodeKind::Image { .. } | AxbwNodeKind::Sovereign =>
            LayoutStyle::column(),
        AxbwNodeKind::Inline =>
            LayoutStyle::row(),
        AxbwNodeKind::Flex =>
            LayoutStyle::row().with_gap(8.0),
    };

    let model = BoxModel::default_with_content(vw, vh)
        .map_err(|_| PrismError::LayoutOverflow)?;

    let mut layout_node = LayoutNode::new(
        &format!("{}", node_id),
        model,
        style,
    );

    for &cid in &node.child_ids {
        let child = axbw_to_layout_node(tree, cid, vw, vh)?;
        layout_node.add_child(child);
    }

    Ok(layout_node)
}

/// AXBW binary builder — for tests
pub struct AxbwBuilder {
    nodes:   Vec<Vec<u8>>,
    root_id: u32,
}

impl AxbwBuilder {
    pub fn new(root_id: u32) -> Self {
        Self { nodes: Vec::new(), root_id }
    }

    pub fn add_block(&mut self, id: u32, budget: f32, children: &[u32]) -> &mut Self {
        self.nodes.push(Self::node(id, KIND_BLOCK, budget, &[], children));
        self
    }

    pub fn add_inline(&mut self, id: u32, budget: f32, children: &[u32]) -> &mut Self {
        self.nodes.push(Self::node(id, KIND_INLINE, budget, &[], children));
        self
    }

    pub fn add_flex(&mut self, id: u32, budget: f32, children: &[u32]) -> &mut Self {
        self.nodes.push(Self::node(id, KIND_FLEX, budget, &[], children));
        self
    }

    pub fn add_sovereign(&mut self, id: u32, budget: f32) -> &mut Self {
        self.nodes.push(Self::node(id, KIND_SOVEREIGN, budget, &[], &[]));
        self
    }

    pub fn add_text(&mut self, id: u32, budget: f32, text: &str) -> &mut Self {
        let tb = text.as_bytes();
        let mut p = (tb.len() as u32).to_le_bytes().to_vec();
        p.extend_from_slice(tb);
        self.nodes.push(Self::node(id, KIND_TEXT, budget, &p, &[]));
        self
    }

    pub fn add_image(&mut self, id: u32, budget: f32, src: &str, aspect: f32) -> &mut Self {
        let sb = src.as_bytes();
        let mut p = (sb.len() as u32).to_le_bytes().to_vec();
        p.extend_from_slice(sb);
        p.extend_from_slice(&aspect.to_bits().to_le_bytes());
        self.nodes.push(Self::node(id, KIND_IMAGE, budget, &p, &[]));
        self
    }

    pub fn build(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(AXBW_MAGIC);
        out.extend_from_slice(&AXBW_VERSION.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&(self.nodes.len() as u32).to_le_bytes());
        out.extend_from_slice(&self.root_id.to_le_bytes());
        for node in &self.nodes { out.extend_from_slice(node); }
        out
    }

    fn node(id: u32, kind: u32, budget: f32, payload: &[u8], children: &[u32]) -> Vec<u8> {
        let mut n = Vec::new();
        n.extend_from_slice(&id.to_le_bytes());
        n.extend_from_slice(&kind.to_le_bytes());
        n.extend_from_slice(&(children.len() as u32).to_le_bytes());
        n.extend_from_slice(&budget.to_bits().to_le_bytes());
        n.extend_from_slice(payload);
        for &cid in children { n.extend_from_slice(&cid.to_le_bytes()); }
        n
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p() -> AxbwParser { AxbwParser::new() }

    #[test]
    fn parse_single_block() {
        let data = AxbwBuilder::new(0).add_block(0, 1.0, &[]).build();
        let tree = p().parse(&data).unwrap();
        assert_eq!(tree.root_id, 0);
        assert_eq!(tree.nodes.len(), 1);
        assert!(matches!(tree.nodes[0].kind, AxbwNodeKind::Block));
    }

    #[test]
    fn parse_parent_child() {
        let data = AxbwBuilder::new(0)
            .add_block(0, 1.0, &[1, 2])
            .add_inline(1, 0.5, &[])
            .add_inline(2, 0.5, &[])
            .build();
        let tree = p().parse(&data).unwrap();
        assert_eq!(tree.nodes.len(), 3);
        assert_eq!(tree.nodes[0].child_ids, vec![1, 2]);
    }

    #[test]
    fn parse_text_node() {
        let data = AxbwBuilder::new(0)
            .add_text(0, 1.0, "Hello, Sovereign World!")
            .build();
        let tree = p().parse(&data).unwrap();
        match &tree.nodes[0].kind {
            AxbwNodeKind::Text(s) => assert_eq!(s, "Hello, Sovereign World!"),
            _ => panic!("expected Text"),
        }
    }

    #[test]
    fn parse_image_node() {
        let data = AxbwBuilder::new(0)
            .add_image(0, 1.0, "awp://logo.png", 16.0/9.0)
            .build();
        let tree = p().parse(&data).unwrap();
        match &tree.nodes[0].kind {
            AxbwNodeKind::Image { src, aspect } => {
                assert_eq!(src, "awp://logo.png");
                assert!((aspect - 16.0/9.0).abs() < 0.001);
            }
            _ => panic!("expected Image"),
        }
    }

    #[test]
    fn parse_flex_node() {
        let data = AxbwBuilder::new(0).add_flex(0, 1.0, &[]).build();
        let tree = p().parse(&data).unwrap();
        assert!(matches!(tree.nodes[0].kind, AxbwNodeKind::Flex));
    }

    #[test]
    fn parse_sovereign_node() {
        let data = AxbwBuilder::new(0).add_sovereign(0, 1.0).build();
        let tree = p().parse(&data).unwrap();
        assert!(matches!(tree.nodes[0].kind, AxbwNodeKind::Sovereign));
    }

    #[test]
    fn parse_budget_preserved() {
        let data = AxbwBuilder::new(0).add_block(0, 0.75, &[]).build();
        let tree = p().parse(&data).unwrap();
        assert!((tree.nodes[0].render_budget - 0.75).abs() < 0.001);
    }

    #[test]
    fn parse_rejects_bad_magic() {
        let mut data = AxbwBuilder::new(0).add_block(0, 1.0, &[]).build();
        data[0] = 0xFF;
        assert!(matches!(p().parse(&data), Err(PrismError::InvalidAxbw(_))));
    }

    #[test]
    fn parse_rejects_wrong_version() {
        let mut data = AxbwBuilder::new(0).add_block(0, 1.0, &[]).build();
        data[4] = 0xFF;
        assert!(matches!(p().parse(&data), Err(PrismError::InvalidAxbw(_))));
    }

    #[test]
    fn parse_rejects_too_short() {
        assert!(matches!(p().parse(&[0u8; 8]), Err(PrismError::InvalidAxbw(_))));
    }

    #[test]
    fn parse_rejects_zero_nodes() {
        let mut data = AxbwBuilder::new(0).add_block(0, 1.0, &[]).build();
        data[8] = 0; data[9] = 0; data[10] = 0; data[11] = 0;
        assert!(matches!(p().parse(&data), Err(PrismError::InvalidAxbw(_))));
    }

    #[test]
    fn parse_deep_tree() {
        let data = AxbwBuilder::new(0)
            .add_block(0, 1.0, &[1])
            .add_block(1, 0.9, &[2])
            .add_block(2, 0.8, &[3])
            .add_block(3, 0.7, &[4])
            .add_block(4, 0.6, &[])
            .build();
        let tree = p().parse(&data).unwrap();
        assert_eq!(tree.nodes.len(), 5);
        assert!((tree.nodes[4].render_budget - 0.6).abs() < 0.001);
    }

    #[test]
    fn axbw_to_layout_node_block() {
        let data = AxbwBuilder::new(0).add_block(0, 1.0, &[]).build();
        let tree = p().parse(&data).unwrap();
        let ln   = axbw_to_layout_node(&tree, 0, 1280.0, 720.0).unwrap();
        assert_eq!(ln.id, "0");
    }

    #[test]
    fn axbw_to_layout_node_with_children() {
        let data = AxbwBuilder::new(0)
            .add_block(0, 1.0, &[1])
            .add_text(1, 0.8, "hello")
            .build();
        let tree = p().parse(&data).unwrap();
        let ln   = axbw_to_layout_node(&tree, 0, 1280.0, 720.0).unwrap();
        assert_eq!(ln.child_count(), 1);
    }
}
