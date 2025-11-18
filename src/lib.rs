// Stupidly simple render-agnostic immediate mode UI lib

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

/// f32 color
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
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

    pub fn sub(a: Vec2, b: Vec2) -> Self {
        Vec2 {
            x: a.x - b.x,
            y: a.y - b.y,
        }
    }

    pub fn div(a: Vec2, b: u32) -> Self {
        Vec2 {
            x: a.x / b,
            y: a.y / b,
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

/// The output of a reimui ui run
#[derive(Debug, Clone)]
pub enum DrawCommand {
    DrawText {
        content: String,
        top_left: Vec2,
        flags: Flags,
        role: UIDrawRole,
    },
    DrawRect {
        top_left: Vec2,
        size: Vec2,
        flags: Flags,
        role: UIDrawRole,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Hint about what a draw command is for
pub enum UIDrawRole {
    Text,
    ButtonText,
    ButtonBackground,
}

#[derive(Copy, Clone)]
/// Persistent UI state object
pub struct UIState {
    active_rect: Option<Rect>,
}

impl Default for UIState {
    fn default() -> Self {
        Self::new()
    }
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

    pub fn draw_rect_raw(&mut self, top_left: Vec2, size: Vec2, flags: Flags, role: UIDrawRole) {
        self.command_buffer.push_back(DrawCommand::DrawRect {
            top_left,
            size,
            flags,
            role,
        });
    }

    pub fn draw_text_raw(&mut self, label: String, top_left: Vec2, flags: Flags, role: UIDrawRole) {
        self.command_buffer.push_back(DrawCommand::DrawText {
            content: label,
            top_left,
            flags,
            role,
        });
    }


    pub fn draw_button_raw(
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

        let half_padding = Vec2::div(
            Vec2::sub(rect.size, text_size),
            2,
        );
        let centered_text_pos = Vec2::add(
            rect.top_left,
            half_padding,
        );

        self.draw_rect_raw(rect.top_left, rect.size, flags, UIDrawRole::ButtonBackground);
        self.draw_text_raw(label, centered_text_pos, flags, UIDrawRole::ButtonText);

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
            role: UIDrawRole::Text,
        });
    }


    pub fn draw_text_layout(&mut self, layout: &mut Layout, label: String) {
        let text_size = self.font_info.compute_text_size(&label);
        self.draw_text(label, layout.top_left);
        layout.recompute(text_size);
    }

    pub fn draw_button(&mut self, top_left: Vec2, padding: Vec2, label: String) -> bool {
        let text_size = self.font_info.compute_text_size(&label);
        self.draw_button_raw(top_left, text_size, padding, label)
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

    #[test]
    fn button_click() {
        let font_info = mock_font_info();
        let ui_state = UIState::new();

        // first frame: mouse down over button
        let mut ctx = super::UIContext::new(
            ui_state,
            &font_info,
            Vec2 { x: 10, y: 10 },
            ButtonState::Down,
        );
        let clicked = ctx.draw_button(
            Vec2 { x: 0, y: 0 },
            Vec2 { x: 8, y: 4 },
            "Click me".into(),
        );
        assert!(!clicked, "button should not register click on mouse down");
        let result = ctx.end();

        // second frame: mouse up over button
        let mut ctx = super::UIContext::new(
            result.new_state,
            &font_info,
            Vec2 { x: 10, y: 10 },
            ButtonState::Up,
        );
        let clicked = ctx.draw_button(
            Vec2 { x: 0, y: 0 },
            Vec2 { x: 8, y: 4 },
            "Click me".into(),
        );
        assert!(clicked, "button should register click on mouse up");
    }

    #[test]
    fn button_click_outside() {
        let font_info = mock_font_info();
        let ui_state = UIState::new();

        // first frame: mouse down outside button
        let mut ctx = super::UIContext::new(
            ui_state,
            &font_info,
            Vec2 { x: 100, y: 100 },
            ButtonState::Down,
        );
        let clicked = ctx.draw_button(
            Vec2 { x: 0, y: 0 },
            Vec2 { x: 8, y: 4 },
            "Click me".into(),
        );
        assert!(!clicked, "button should not register click on mouse down outside");
        let result = ctx.end();

        // second frame: mouse up outside button
        let mut ctx = super::UIContext::new(
            result.new_state,
            &font_info,
            Vec2 { x: 100, y: 100 },
            ButtonState::Up,
        );
        let clicked = ctx.draw_button(
            Vec2 { x: 0, y: 0 },
            Vec2 { x: 8, y: 4 },
            "Click me".into(),
        );
        assert!(!clicked, "button should not register click on mouse up outside");
    }
}
