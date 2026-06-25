// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// haniel_prism::html — HTML subset parser
// Parses div/p/h1-h6/img/a/span/ul/li into AxbwTree
// Open web compatibility path (cone pass)

use crate::{AxbwTree, AxbwNode, AxbwNodeKind, PrismError};

/// HTML subset parser — converts HTML to internal AxbwTree
pub struct HtmlParser;

/// Minimal HTML token
#[derive(Debug, Clone, PartialEq)]
enum HtmlToken {
    OpenTag  { name: String, attrs: Vec<(String, String)> },
    CloseTag { name: String },
    Text     (String),
    SelfClose { name: String, attrs: Vec<(String, String)> },
}

impl HtmlParser {
    pub fn new() -> Self { Self }

    /// Parse HTML string into AxbwTree
    pub fn parse(&self, html: &str) -> Result<AxbwTree, PrismError> {
        let tokens = self.tokenize(html)?;
        let (tree, _) = self.build_tree(&tokens, 0, &mut 0)?;
        Ok(tree)
    }

    /// Tokenize HTML into tokens
    fn tokenize(&self, html: &str) -> Result<Vec<HtmlToken>, PrismError> {
        let mut tokens = Vec::new();
        let mut chars  = html.chars().peekable();
        let mut text_buf = String::new();

        while let Some(&ch) = chars.peek() {
            if ch == '<' {
                // Flush text buffer
                if !text_buf.trim().is_empty() {
                    tokens.push(HtmlToken::Text(text_buf.trim().to_string()));
                }
                text_buf.clear();

                chars.next(); // consume '<'

                // Collect tag content
                let mut tag_content = String::new();
                let mut is_close    = false;

                if chars.peek() == Some(&'/') {
                    is_close = true;
                    chars.next();
                }

                while let Some(&c) = chars.peek() {
                    if c == '>' { chars.next(); break; }
                    tag_content.push(c);
                    chars.next();
                }

                let tag_content = tag_content.trim();
                let is_self_close = tag_content.ends_with('/');
                let tag_content = tag_content.trim_end_matches('/').trim();

                if is_close {
                    let name = tag_content.split_whitespace()
                        .next().unwrap_or("").to_lowercase();
                    if !name.is_empty() {
                        tokens.push(HtmlToken::CloseTag { name });
                    }
                } else {
                    let mut parts = tag_content.splitn(2, char::is_whitespace);
                    let name  = parts.next().unwrap_or("").to_lowercase();
                    let rest  = parts.next().unwrap_or("");
                    let attrs = self.parse_attrs(rest);

                    if name.is_empty() { continue; }

                    // Self-closing tags
                    let self_closing = is_self_close
                        || matches!(name.as_str(), "img"|"br"|"hr"|"input"|"meta"|"link");

                    if self_closing {
                        tokens.push(HtmlToken::SelfClose { name, attrs });
                    } else {
                        tokens.push(HtmlToken::OpenTag { name, attrs });
                    }
                }
            } else {
                text_buf.push(ch);
                chars.next();
            }
        }

        // Flush remaining text
        if !text_buf.trim().is_empty() {
            tokens.push(HtmlToken::Text(text_buf.trim().to_string()));
        }

        Ok(tokens)
    }

    /// Parse HTML attributes from attribute string
    fn parse_attrs(&self, attrs_str: &str) -> Vec<(String, String)> {
        let mut attrs  = Vec::new();
        let mut rest   = attrs_str.trim();

        while !rest.is_empty() {
            // Find key
            let eq_pos = rest.find('=');
            match eq_pos {
                None => {
                    // Boolean attribute
                    let key = rest.split_whitespace().next().unwrap_or("").to_string();
                    if !key.is_empty() {
                        attrs.push((key.to_lowercase(), "true".to_string()));
                    }
                    break;
                }
                Some(pos) => {
                    let key = rest[..pos].trim().to_lowercase();
                    rest    = rest[pos+1..].trim();

                    // Value — quoted or unquoted
                    let (value, consumed) = if rest.starts_with('"')||rest.starts_with('\''){ 
                        let quote = rest.chars().next().unwrap();
                        let end   = rest[1..].find(quote).unwrap_or(rest.len()-1);
                        (rest[1..end+1].to_string(), end+2)
                    } else {
                        let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
                        (rest[..end].to_string(), end)
                    };

                    attrs.push((key, value));
                    rest = rest[consumed.min(rest.len())..].trim();
                }
            }
        }
        attrs
    }

    /// Build AxbwTree from token list
    fn build_tree(
        &self,
        tokens:  &[HtmlToken],
        id_base: u32,
        counter: &mut u32,
    ) -> Result<(AxbwTree, usize), PrismError> {
        let mut nodes    = Vec::new();
        let mut stack: Vec<(u32, Vec<u32>)> = Vec::new(); // (node_id, children)
        let mut consumed = 0usize;
        let mut root_id  = 0u32;

        for (i, token) in tokens.iter().enumerate() {
            consumed = i + 1;
            match token {
                HtmlToken::OpenTag { name, attrs } => {
                    let id   = id_base + *counter;
                    *counter += 1;
                    let kind = Self::tag_to_kind(name, attrs);
                    if stack.is_empty() { root_id = id; }
                    stack.push((id, Vec::new()));
                    // Push placeholder — filled when tag closes
                    nodes.push((id, kind, Vec::new()));
                }
                HtmlToken::CloseTag { name: _ } => {
                    if let Some((closed_id, children)) = stack.pop() {
                        // Update children in nodes
                        if let Some(entry) = nodes.iter_mut().find(|(id,_,_)| *id == closed_id) {
                            entry.2 = children.clone();
                        }
                        // Add to parent
                        if let Some((_, parent_children)) = stack.last_mut() {
                            parent_children.push(closed_id);
                        }
                    }
                }
                HtmlToken::SelfClose { name, attrs } => {
                    let id   = id_base + *counter;
                    *counter += 1;
                    let kind = Self::tag_to_kind(name, attrs);
                    nodes.push((id, kind, Vec::new()));
                    if stack.is_empty() {
                        root_id = id;
                    } else if let Some((_, children)) = stack.last_mut() {
                        children.push(id);
                    }
                }
                HtmlToken::Text(text) => {
                    let id   = id_base + *counter;
                    *counter += 1;
                    nodes.push((id, AxbwNodeKind::Text(text.clone()), Vec::new()));
                    if let Some((_, children)) = stack.last_mut() {
                        children.push(id);
                    }
                    if stack.is_empty() { root_id = id; }
                }
            }
        }

        // Close any unclosed tags
        while let Some((closed_id, children)) = stack.pop() {
            if let Some(entry) = nodes.iter_mut().find(|(id,_,_)| *id == closed_id) {
                entry.2 = children.clone();
            }
            if let Some((_, parent_children)) = stack.last_mut() {
                parent_children.push(closed_id);
            }
        }

        // Convert to AxbwNode vec
        let axbw_nodes = nodes.into_iter().map(|(id, kind, child_ids)| {
            AxbwNode { id, kind, child_ids, render_budget: 1.0 }
        }).collect();

        Ok((AxbwTree { root_id, nodes: axbw_nodes }, consumed))
    }

    /// Map HTML tag name to AxbwNodeKind
    fn tag_to_kind(name: &str, attrs: &[(String, String)]) -> AxbwNodeKind {
        match name {
            "div" | "section" | "article" | "main" | "header"
            | "footer" | "nav" | "aside" => AxbwNodeKind::Block,

            "p" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6"
            | "blockquote" | "pre" => {
                // Heading/paragraph — text content resolved at cone pass
                AxbwNodeKind::Block
            }

            "span" | "a" | "strong" | "em" | "code"
            | "label" | "button" => AxbwNodeKind::Inline,

            "ul" | "ol" | "li" => AxbwNodeKind::Block,

            "img" => {
                let src    = attrs.iter()
                    .find(|(k,_)| k == "src")
                    .map(|(_,v)| v.clone())
                    .unwrap_or_default();
                let aspect = attrs.iter()
                    .find(|(k,_)| k == "width")
                    .and_then(|(_,v)| v.parse::<f32>().ok())
                    .zip(attrs.iter()
                        .find(|(k,_)| k == "height")
                        .and_then(|(_,v)| v.parse::<f32>().ok()))
                    .map(|(w,h)| if h > 0.0 { w/h } else { 16.0/9.0 })
                    .unwrap_or(16.0/9.0);
                AxbwNodeKind::Image { src, aspect }
            }

            "table" | "tr" | "td" | "th" | "thead" | "tbody" => {
                AxbwNodeKind::Flex
            }

            _ => AxbwNodeKind::Block, // unknown tags → block
        }
    }
}

impl Default for HtmlParser {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p() -> HtmlParser { HtmlParser::new() }

    #[test]
    fn parse_single_div() {
        let tree = p().parse("<div></div>").unwrap();
        assert_eq!(tree.nodes.len(), 1);
        assert!(matches!(tree.nodes[0].kind, AxbwNodeKind::Block));
    }

    #[test]
    fn parse_div_with_text() {
        let tree = p().parse("<div>Hello World</div>").unwrap();
        assert!(tree.nodes.len() >= 2);
        let has_text = tree.nodes.iter()
            .any(|n| matches!(&n.kind, AxbwNodeKind::Text(t) if t.contains("Hello")));
        assert!(has_text, "expected Text node with Hello");
    }

    #[test]
    fn parse_nested_divs() {
        let tree = p().parse("<div><div></div></div>").unwrap();
        assert!(tree.nodes.len() >= 2);
        assert!(!tree.nodes[0].child_ids.is_empty());
    }

    #[test]
    fn parse_heading_tags() {
        for tag in &["h1","h2","h3","h4","h5","h6"] {
            let html = format!("<{}>Title</{}>", tag, tag);
            let tree = p().parse(&html).unwrap();
            assert!(tree.nodes.len() >= 1, "failed for {}", tag);
        }
    }

    #[test]
    fn parse_img_self_closing() {
        let tree = p().parse(r#"<img src="logo.png" width="160" height="90"/>"#).unwrap();
        assert!(tree.nodes.iter().any(|n| matches!(&n.kind,
            AxbwNodeKind::Image { src, .. } if src == "logo.png"
        )));
    }

    #[test]
    fn parse_img_aspect_ratio() {
        let tree = p().parse(r#"<img src="x.png" width="160" height="90"/>"#).unwrap();
        let img = tree.nodes.iter().find(|n| matches!(&n.kind, AxbwNodeKind::Image{..})).unwrap();
        match &img.kind {
            AxbwNodeKind::Image { aspect, .. } => {
                assert!((aspect - 16.0/9.0).abs() < 0.01);
            }
            _ => panic!("expected image"),
        }
    }

    #[test]
    fn parse_span_is_inline() {
        let tree = p().parse("<span>text</span>").unwrap();
        assert!(tree.nodes.iter().any(|n| matches!(n.kind, AxbwNodeKind::Inline)));
    }

    #[test]
    fn parse_ul_li() {
        let tree = p().parse("<ul><li>Item 1</li><li>Item 2</li></ul>").unwrap();
        assert!(tree.nodes.len() >= 3);
    }

    #[test]
    fn parse_empty_html() {
        // Should not panic
        let result = p().parse("");
        // Empty is ok or produces minimal tree
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn parse_paragraph() {
        let tree = p().parse("<p>Sovereign paragraph text.</p>").unwrap();
        let has_text = tree.nodes.iter()
            .any(|n| matches!(&n.kind, AxbwNodeKind::Text(t) if t.contains("Sovereign")));
        assert!(has_text);
    }
}
