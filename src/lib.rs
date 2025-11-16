use std::collections::VecDeque;

use crate::flags::Flags;

#[rustfmt::skip]
pub mod flags {
    pub type Flags = u32;
    pub const HOT: Flags          = 1 << 0;
    pub const DISABLED: Flags     = 1 << 1;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ButtonState {
    Down,
    Up,
}

#[derive(Debug, Clone, Copy)]
pub struct Vec2 {
    pub x: u32,
    pub y: u32,
}

impl Vec2 {
    pub fn zero() -> Self {
        Vec2 { x: 0, y: 0 }
    }
}

pub enum DrawCommand {
    DrawText {
        content: String,
        x: u32,
        y: u32,
        flags: Flags,
    },
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum LayoutDirection {
    Vertical,
    Horizontal,
}

#[derive(Debug, Copy, Clone)]
pub struct Layout {
    pub direction: LayoutDirection,
    pub spacing: u32,
    pub top_left: Vec2,
    pub size: Vec2,
}

impl Layout {
    pub fn new(direction: LayoutDirection, spacing: u32, top_left: Vec2, size: Vec2) -> Self {
        Layout {
            direction,
            spacing,
            top_left,
            size,
        }
    }

    pub fn layout(&self, direction: LayoutDirection, spacing: u32) -> Self {
        Self {
            direction,
            spacing,
            top_left: self.top_left,
            size: Vec2::zero(),
        }
    }
}

/// Tell me how big your text is
pub trait FontInformation {
    fn compute_text_size(&self, text: &str) -> Vec2;
}

pub struct UIContext<'f> {
    command_buffer: VecDeque<DrawCommand>,
    font_info: &'f dyn FontInformation,

    mouse_position: Vec2,
    mouse_primary_button: ButtonState,
}

impl<'s, 'f> UIContext<'f> {
    pub fn new(
        font_info: &'f dyn FontInformation,
        mouse_position: Vec2,
        mouse_primary_button: ButtonState,
    ) -> Self {
        UIContext {
            command_buffer: VecDeque::new(),
            font_info,
            mouse_position,
            mouse_primary_button,
        }
    }

    pub fn draw_text(&mut self, label: String, x: u32, y: u32) {
        self.command_buffer.push_back(DrawCommand::DrawText {
            content: label,
            x,
            y,
            flags: 0,
        });
    }

    pub fn draw_text_with_flags(&mut self, label: String, x: u32, y: u32, flags: Flags) {
        self.command_buffer.push_back(DrawCommand::DrawText {
            content: label,
            x,
            y,
            flags: flags,
        });
    }

    pub fn draw_text_layout(&mut self, layout: &mut Layout, label: String) {
        let text_size = self.font_info.compute_text_size(&label);
        let mut top_left = layout.top_left;
        self.draw_text(label, top_left.x, top_left.y);

        match layout.direction {
            LayoutDirection::Vertical => {
                top_left.y += text_size.y + layout.spacing;
            }
            LayoutDirection::Horizontal => {
                top_left.x += text_size.x + layout.spacing;
            }
        }
        layout.top_left = top_left;
    }

    pub fn draw_button(&mut self, layout: &mut Layout, padding: Vec2, label: String) -> bool {
        let text_size = self.font_info.compute_text_size(&label);
        let mut top_left = layout.top_left;
        let button_width = text_size.x + padding.x;
        let button_height = text_size.y + padding.y;

        let is_hovered = self.mouse_position.x >= top_left.x
            && self.mouse_position.x <= top_left.x + button_width
            && self.mouse_position.y >= top_left.y
            && self.mouse_position.y <= top_left.y + button_height;

        self.draw_text(
            label,
            top_left.x + padding.x / 2,
            top_left.y + padding.y / 2,
        );

        let clicked = is_hovered && self.mouse_primary_button == ButtonState::Down;

        match layout.direction {
            LayoutDirection::Vertical => {
                top_left.y += button_height + layout.spacing;
            }
            LayoutDirection::Horizontal => {
                top_left.x += button_width + layout.spacing;
            }
        }
        layout.top_left = top_left;

        clicked
    }
}

#[cfg(test)]

mod test {

    const MOCK_TEXT_HEIGHT: u32 = 16;
    const MOCK_TEXT_WIDTH: u32 = 8;

    use super::*;
    fn mock_font_info() -> impl FontInformation {
        struct MockFontInfo;
        impl FontInformation for MockFontInfo {
            fn compute_text_size(&self, text: &str) -> Vec2 {
                Vec2 {
                    x: text.len() as u32 * MOCK_TEXT_WIDTH,
                    y: MOCK_TEXT_HEIGHT,
                }
            }
        }
        MockFontInfo
    }

    #[test]
    fn layout() {
        const SECTION_TEXT_LEN: u32 = 9;

        let font_info = mock_font_info();
        let mut ctx = super::UIContext::new(&font_info, Vec2 { x: 0, y: 0 }, ButtonState::Up);
        // draw a horizontal group of texts, each with a vertical layout of text inside
        let mut layout = Layout {
            direction: LayoutDirection::Horizontal,
            spacing: 4,
            top_left: Vec2 { x: 0, y: 0 },
            size: Vec2 { x: 800, y: 600 },
        };

        for i in 0..3 {
            let label = format!("Section {}", i);
            assert!(
                label.len() as u32 == SECTION_TEXT_LEN,
                "broken test assertion"
            );
            ctx.draw_text_layout(&mut layout, label);

            for j in 0..2 {
                let sub_label = format!("Section {} item {}", i, j);
                let mut sub_layout = layout.layout(LayoutDirection::Vertical, 2);
                ctx.draw_text_layout(&mut sub_layout, sub_label);
                assert_eq!(sub_layout.top_left.x, layout.top_left.x);
                assert_eq!(
                    sub_layout.top_left.y,
                    layout.top_left.y + MOCK_TEXT_HEIGHT + sub_layout.spacing
                );
            }
        }
        assert_eq!(
            layout.top_left.x,
            3 * (SECTION_TEXT_LEN * MOCK_TEXT_WIDTH + layout.spacing)
        );

        assert_eq!(ctx.command_buffer.len(), 9);
    }
}
