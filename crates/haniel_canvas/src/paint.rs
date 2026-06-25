// Copyright (c) 2026 Edison Lepین / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// haniel_canvas::paint — Sovereign paint command pipeline

/// RGBA color
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self { Self { r, g, b, a } }
    pub const fn rgb(r: u8, g: u8, b: u8)          -> Self { Self { r, g, b, a: 255 } }
    pub const fn transparent()                       -> Self { Self { r: 0, g: 0, b: 0, a: 0 } }

    // Sovereign palette
    pub const SOVEREIGN_BLACK:  Self = Self::rgb(10,  10,  10);
    pub const SOVEREIGN_WHITE:  Self = Self::rgb(245, 245, 245);
    pub const SOVEREIGN_ACCENT: Self = Self::rgb(0,   180, 255);
    pub const SOVEREIGN_BG:     Self = Self::rgb(18,  18,  24);
    pub const SOVEREIGN_BORDER: Self = Self::rgb(50,  50,  70);
    pub const THREAT_RED:       Self = Self::rgb(220, 50,  50);
    pub const VERIFIED_GREEN:   Self = Self::rgb(50,  200, 100);
    pub const GUARDED_AMBER:    Self = Self::rgb(255, 165, 0);
}

/// A single paint instruction
#[derive(Debug, Clone)]
pub enum PaintCommand {
    /// Fill a rectangle with a solid color
    FillRect {
        x: f32, y: f32,
        w: f32, h: f32,
        color: Color,
    },
    /// Draw a 1px border around a rectangle
    StrokeRect {
        x: f32, y: f32,
        w: f32, h: f32,
        color: Color,
    },
    /// Render text (glyph rendering at HE-8, placeholder box now)
    Text {
        x: f32, y: f32,
        content: String,
        size: f32,
        color: Color,
    },
    /// Render an image placeholder (full image at HE-10)
    Image {
        x: f32, y: f32,
        w: f32, h: f32,
        src: String,
    },
    /// Clear the entire buffer to a color
    Clear(Color),
    /// Blit a sub-buffer at position
    Blit {
        src_id: String,
        dx: f32, dy: f32,
    },
}

/// Display list — ordered sequence of paint commands
#[derive(Debug, Default, Clone)]
pub struct DisplayList {
    commands: Vec<PaintCommand>,
}

impl DisplayList {
    pub fn new() -> Self { Self { commands: Vec::new() } }

    pub fn push(&mut self, cmd: PaintCommand) {
        self.commands.push(cmd);
    }

    pub fn commands(&self) -> &[PaintCommand] {
        &self.commands
    }

    pub fn len(&self) -> usize { self.commands.len() }

    pub fn is_empty(&self) -> bool { self.commands.is_empty() }

    pub fn clear(&mut self) { self.commands.clear(); }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_sovereign_palette_defined() {
        let _ = Color::SOVEREIGN_BLACK;
        let _ = Color::SOVEREIGN_WHITE;
        let _ = Color::SOVEREIGN_ACCENT;
        let _ = Color::SOVEREIGN_BG;
        let _ = Color::THREAT_RED;
        let _ = Color::VERIFIED_GREEN;
    }

    #[test]
    fn color_transparent_alpha_zero() {
        assert_eq!(Color::transparent().a, 0);
    }

    #[test]
    fn color_rgb_alpha_255() {
        assert_eq!(Color::rgb(255, 0, 0).a, 255);
    }

    #[test]
    fn display_list_push_and_len() {
        let mut dl = DisplayList::new();
        dl.push(PaintCommand::Clear(Color::SOVEREIGN_BG));
        dl.push(PaintCommand::FillRect {
            x: 0.0, y: 0.0, w: 100.0, h: 50.0,
            color: Color::SOVEREIGN_ACCENT,
        });
        assert_eq!(dl.len(), 2);
    }

    #[test]
    fn display_list_clear() {
        let mut dl = DisplayList::new();
        dl.push(PaintCommand::Clear(Color::SOVEREIGN_BG));
        dl.clear();
        assert!(dl.is_empty());
    }

    #[test]
    fn display_list_commands_slice() {
        let mut dl = DisplayList::new();
        dl.push(PaintCommand::FillRect {
            x: 10.0, y: 10.0, w: 50.0, h: 50.0,
            color: Color::VERIFIED_GREEN,
        });
        assert_eq!(dl.commands().len(), 1);
    }

    #[test]
    fn paint_command_variants_exist() {
        let _ = PaintCommand::Clear(Color::SOVEREIGN_BG);
        let _ = PaintCommand::FillRect { x:0.0, y:0.0, w:1.0, h:1.0, color: Color::SOVEREIGN_WHITE };
        let _ = PaintCommand::StrokeRect { x:0.0, y:0.0, w:1.0, h:1.0, color: Color::SOVEREIGN_BORDER };
        let _ = PaintCommand::Text { x:0.0, y:0.0, content:"test".into(), size:16.0, color: Color::SOVEREIGN_WHITE };
        let _ = PaintCommand::Image { x:0.0, y:0.0, w:100.0, h:100.0, src:"img.png".into() };
    }
}
