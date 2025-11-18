use std::collections::VecDeque;

use crate::flags::Flags;

#[rustfmt::skip]
pub mod flags {
    pub type Flags = u32;
    pub const NONE: Flags           = 0;
    pub const HOVER: Flags          = 1 << 0;
    pub const DISABLED: Flags       = 1 << 1;
    pub const ACTIVE: Flags         = 1 << 2;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ButtonState {
    Down,
    Up,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Vec2 {
    pub x: u32,
    pub y: u32,
}

impl Vec2 {
    pub fn zero() -> Self {
        Vec2 { x: 0, y: 0 }
    }

    pub fn add(a: Vec2, b: Vec2) -> Self {
        Vec2 {
            x: a.x + b.x,
            y: a.y + b.y,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Rect {
    pub top_left: Vec2,
    pub size: Vec2,
}

impl Rect {
    pub fn contains(&self, point: Vec2) -> bool {
        point.x >= self.top_left.x
            && point.x <= self.top_left.x + self.size.x
            && point.y >= self.top_left.y
            && point.y <= self.top_left.y + self.size.y
    }
}

pub enum DrawCommand {
    DrawText {
        content: String,
        top_left: Vec2,
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

    pub fn recompute(&mut self, size: Vec2) {
        match self.direction {
            LayoutDirection::Vertical => {
                self.top_left.y += size.y + self.spacing;
            }
            LayoutDirection::Horizontal => {
                self.top_left.x += size.x + self.spacing;
            }
        }
    }
}

/// Tell me how big your text is
pub trait FontInformation {
    fn compute_text_size(&self, text: &str) -> Vec2;
}

#[derive(Copy, Clone)]
/// Persistent UI state object
pub struct UIState {
    active_rect: Option<Rect>,
}

impl UIState {
    pub fn new() -> Self {
        Self { active_rect: None }
    }
}

/// Result of a computation of the UI
pub struct UIResult {
    pub new_state: UIState,
    pub commands: Vec<DrawCommand>,
}

/// Transient draw context
pub struct UIContext<'f> {
    state: UIState,
    font_info: &'f dyn FontInformation,
    mouse_position: Vec2,
    mouse_primary_button: ButtonState,

    hover_rect: Option<Rect>,

    command_buffer: VecDeque<DrawCommand>,
}

impl<'f> UIContext<'f> {
    fn draw_button_raw(
        &mut self,
        top_left: Vec2,
        text_size: Vec2,
        padding: Vec2,
        label: String,
    ) -> bool {
        let button_size = Vec2::add(text_size, padding);
        let rect = Rect {
            top_left,
            size: button_size,
        };

        let hovered = self.check_set_hover(rect);
        let active = self.is_active(rect);

        let mut flags = flags::NONE;
        if hovered {
            flags |= flags::HOVER;
        }
        if active {
            flags |= flags::ACTIVE;
        }

        self.draw_text_with_flags(
            label,
            top_left,
            flags,
        );

        hovered && self.clicked_rect(rect)
    }

    fn is_active(&self, rect: Rect) -> bool {
        self.state
            .active_rect
            .is_some_and(|active_rect| active_rect == rect)
    }

    fn clicked_rect(&self, rect: Rect) -> bool {
        self.mouse_primary_button == ButtonState::Up && self.is_active(rect)
    }

    fn check_set_hover(&mut self, rect: Rect) -> bool {
        let is_hover = rect.contains(self.mouse_position);
        if is_hover {
            self.hover_rect = Some(rect);
        }

        is_hover
    }

    pub fn new(
        state: UIState,
        font_info: &'f dyn FontInformation,
        mouse_position: Vec2,
        mouse_primary_button: ButtonState,
    ) -> Self {
        Self {
            command_buffer: VecDeque::new(),
            mouse_position,
            mouse_primary_button,
            hover_rect: None,
            state,
            font_info,
        }
    }

    pub fn draw_text(&mut self, label: String, top_left: Vec2) {
        self.command_buffer.push_back(DrawCommand::DrawText {
            content: label,
            top_left,
            flags: 0,
        });
    }

    pub fn draw_text_with_flags(&mut self, label: String, top_left: Vec2, flags: Flags) {
        self.command_buffer.push_back(DrawCommand::DrawText {
            content: label,
            top_left,
            flags: flags,
        });
    }

    pub fn draw_text_layout(&mut self, layout: &mut Layout, label: String) {
        let text_size = self.font_info.compute_text_size(&label);
        self.draw_text(label, layout.top_left);
        layout.recompute(text_size);
    }

    pub fn draw_button_layout(
        &mut self,
        layout: &mut Layout,
        padding: Vec2,
        label: String,
    ) -> bool {
        let text_size = self.font_info.compute_text_size(&label);
        let clicked = self.draw_button_raw(layout.top_left, text_size, padding, label);
        layout.recompute(Vec2::add(text_size, padding));
        clicked
    }

    /// Finalize the computation of the UI and return the resulting state and draw info
    pub fn end(mut self) -> UIResult {
        // mouse down over hover => active
        if self.mouse_primary_button == ButtonState::Down {
            self.state.active_rect = self.hover_rect;
        } else {
            self.state.active_rect = None;
        }

        UIResult {
            new_state: self.state,
            commands: self.command_buffer.into(),
        }
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
        let ui_state = UIState::new();
        let mut ctx =
            super::UIContext::new(ui_state, &font_info, Vec2 { x: 0, y: 0 }, ButtonState::Up);
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
