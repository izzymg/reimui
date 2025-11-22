// Stupidly simple render-agnostic immediate mode UI lib

use crate::flags::Flags;
use std::{collections::VecDeque, ops::Range};

pub mod prelude {
    pub use super::{
        ButtonState, FontInformation, Layout, LayoutDirection, Rect, UIContext, UIState, Vec2,
    };
}

#[rustfmt::skip]
pub mod flags {
    pub type Flags = u32;
    pub const NONE: Flags           = 0;
    pub const HOVER: Flags          = 1 << 0;
    pub const DISABLED: Flags       = 1 << 1;
    pub const ACTIVE: Flags         = 1 << 2;
}

/// Something that can be used as a slider value.
/// Primitive numerical values are already implemented.
pub trait SliderValue: Copy {
    fn percentage(value: Self, min: Self, max: Self) -> f32;
    fn increment(value: Self, step: Self, min: Self, max: Self) -> Self;
    fn decrement(value: Self, step: Self, min: Self, max: Self) -> Self;
    fn clamp_value(value: Self, min: Self, max: Self) -> Self;
    fn step_percentage(step: Self, min: Self, max: Self) -> f32;
}

pub struct SliderState<T> {
    pub value: T,
    pub max: T,
    pub min: T,
    pub step: T,
}

impl<T> SliderState<T> {
    pub fn new_range(bounds: Range<T>, initial: T, step: T) -> Self {
        Self {
            value: initial,
            max: bounds.end,
            min: bounds.start,
            step,
        }
    }

    pub fn new(min: T, max: T, initial: T, step: T) -> Self {
        Self {
            value: initial,
            max,
            min,
            step,
        }
    }
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
    pub const fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }

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

impl From<Layout> for Rect {
    fn from(value: Layout) -> Self {
        Rect {
            size: value.size,
            top_left: value.top_left,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DrawData {
    pub rect: Rect,
    pub flags: Flags,
    pub role: UIDrawRole,
}

/// The output of a reimui ui run
#[derive(Debug, Clone)]
pub enum DrawCommand {
    DrawText {
        content: String,
        draw_data: DrawData,
    },
    DrawRect {
        draw_data: DrawData,
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

    pub fn recompute(&mut self, size: Vec2) {
        match self.direction {
            LayoutDirection::Vertical => {
                // add spacing if not first element
                if self.size.y != 0 {
                    self.size.y = self.size.y.saturating_add(self.spacing);
                }
                self.size.y = self.size.y.saturating_add(size.y);
                self.size.x = self.size.x.max(size.x);
                self.top_left.y = self
                    .top_left
                    .y
                    .saturating_add(size.y.saturating_add(self.spacing));
            }
            LayoutDirection::Horizontal => {
                if self.size.x != 0 {
                    self.size.x = self.size.x.saturating_add(self.spacing);
                }
                self.size.x = self.size.x.saturating_add(size.x);
                self.size.y = self.size.y.max(size.y);
                self.top_left.x = self
                    .top_left
                    .x
                    .saturating_add(size.x.saturating_add(self.spacing));
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
    SliderRect,
    SliderKnob,
    LayoutBackground,
}

#[derive(Copy, Clone)]
/// Persistent UI state object
pub struct UIState {
    active_rect: Option<Rect>,
    last_mouse_position: Vec2,
    active_drag_amt: f32,
}

impl Default for UIState {
    fn default() -> Self {
        Self::new()
    }
}

impl UIState {
    pub fn new() -> Self {
        Self {
            active_rect: None,
            last_mouse_position: Vec2::zero(),
            active_drag_amt: 0.0,
        }
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

    layout_stack: Vec<Layout>,
}

impl<'f> UIContext<'f> {
    pub fn new(
        state: UIState,
        font_info: &'f dyn FontInformation,
        mouse_position: Vec2,
        mouse_primary_button: ButtonState,
    ) -> Self {
        let initial_layout_stack = vec![Layout::new(
            LayoutDirection::Horizontal,
            0,
            Vec2::zero(),
            Vec2::zero(),
        )];

        Self {
            command_buffer: VecDeque::new(),
            mouse_position,
            mouse_primary_button,
            hover_rect: None,
            state,
            font_info,
            layout_stack: initial_layout_stack,
        }
    }

    pub fn new_layout_init(
        state: UIState,
        font_info: &'f dyn FontInformation,
        mouse_position: Vec2,
        mouse_primary_button: ButtonState,

        position: Vec2,
        spacing: u32,
    ) -> Self {
        let initial_layout_stack = vec![Layout::new(
            LayoutDirection::Horizontal,
            spacing,
            position,
            Vec2::zero(),
        )];

        Self {
            command_buffer: VecDeque::new(),
            mouse_position,
            mouse_primary_button,
            hover_rect: None,
            state,
            font_info,
            layout_stack: initial_layout_stack,
        }
    }

    pub fn get_current_layout(&self) -> &Layout {
        self.layout_stack
            .last()
            .expect("get layout: should always have a root layout")
    }

    pub fn recompute_current_layout(&mut self, size: Vec2) {
        self.layout_stack
            .last_mut()
            .expect("compute layout: should always have a root layout")
            .recompute(size);
    }

    /// Returns the index into the command buffer of this draw
    pub fn rect_raw(&mut self, rect: Rect, flags: Flags, role: UIDrawRole) -> usize {
        let idx = self.command_buffer.len();
        self.command_buffer.push_back(DrawCommand::DrawRect {
            draw_data: DrawData { rect, flags, role },
        });
        idx
    }

    pub fn text_raw(&mut self, label: String, rect: Rect, flags: Flags, role: UIDrawRole) {
        self.command_buffer.push_back(DrawCommand::DrawText {
            content: label,
            draw_data: DrawData { rect, flags, role },
        });
    }

    pub fn button_raw(
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

        let half_padding = Vec2::div(Vec2::sub(rect.size, text_size), 2);
        let centered_text_pos = Vec2::add(rect.top_left, half_padding);

        self.rect_raw(rect, flags, UIDrawRole::ButtonBackground);
        self.text_raw(
            label,
            Rect {
                top_left: centered_text_pos,
                size: text_size,
            },
            flags,
            UIDrawRole::ButtonText,
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

    pub fn text(&mut self, label: String, rect: Rect) {
        self.command_buffer.push_back(DrawCommand::DrawText {
            content: label,
            draw_data: DrawData {
                rect,
                flags: flags::NONE,
                role: UIDrawRole::Text,
            },
        });
    }

    pub fn text_layout(&mut self, label: String) {
        let layout = self.get_current_layout();
        let text_size = self.font_info.compute_text_size(&label);
        self.text(
            label,
            Rect {
                size: text_size,
                top_left: layout.top_left,
            },
        );
        self.recompute_current_layout(text_size);
    }

    pub fn button(&mut self, top_left: Vec2, padding: Vec2, label: String) -> bool {
        let text_size = self.font_info.compute_text_size(&label);
        self.button_raw(top_left, text_size, padding, label)
    }

    pub fn button_layout(&mut self, padding: Vec2, label: String) -> bool {
        let layout = self.get_current_layout();
        let text_size = self.font_info.compute_text_size(&label);
        let clicked = self.button_raw(layout.top_left, text_size, padding, label);
        self.recompute_current_layout(Vec2::add(text_size, padding));
        clicked
    }

    pub fn slider<T: SliderValue>(&mut self, rect: Rect, state: &mut SliderState<T>) {
        let hovered = self.check_set_hover(rect);
        let is_active = self.is_active(rect);
        let knob_size = Vec2::new(10, rect.size.y);

        // by how many pixels does each step of the slider correspond to
        let slider_span = rect.size.x.saturating_sub(knob_size.x);
        let pixels_per_step = if slider_span == 0 {
            0.0
        } else {
            slider_span as f32 * T::step_percentage(state.step, state.min, state.max)
        };

        if is_active {
            // build drag value over this draw
            let delta_x = self.mouse_position.x as f32 - self.state.last_mouse_position.x as f32;
            self.state.active_drag_amt += delta_x;

            if pixels_per_step > 0.0 {
                // increment n steps based on the amount dragged
                let steps = (self.state.active_drag_amt / pixels_per_step).trunc();
                let steps_i = steps as i32;
                if steps_i != 0 {
                    if steps_i > 0 {
                        for _ in 0..steps_i {
                            state.value =
                                T::increment(state.value, state.step, state.min, state.max);
                        }
                    } else {
                        for _ in 0..(-steps_i) {
                            state.value =
                                T::decrement(state.value, state.step, state.min, state.max);
                        }
                    }

                    // keep remainder of drag
                    self.state.active_drag_amt -= (steps as f32) * pixels_per_step;
                }
            } else {
                // increment once on any drag
                if self.state.active_drag_amt > 0.0 {
                    state.value = T::increment(state.value, state.step, state.min, state.max);
                } else if self.state.active_drag_amt < 0.0 {
                    state.value = T::decrement(state.value, state.step, state.min, state.max);
                }
                self.state.active_drag_amt = 0.0;
            }
        }
        state.value = T::clamp_value(state.value, state.min, state.max);
        let value_percentage =
            T::percentage(state.value, state.min, state.max).clamp(0.0_f32, 1.0_f32);
        let mut flags = flags::NONE;
        if hovered {
            flags |= flags::HOVER;
        }

        // move the knob by the percentage it is into the slider rect
        let knob_top_left = Vec2::add(
            rect.top_left,
            Vec2::new(
                ((rect.size.x.saturating_sub(knob_size.x)) as f32 * value_percentage) as u32,
                0,
            ),
        );

        self.rect_raw(rect, flags, UIDrawRole::SliderRect);
        self.rect_raw(
            Rect {
                size: knob_size,
                top_left: knob_top_left,
            },
            flags,
            UIDrawRole::SliderKnob,
        );
    }

    pub fn slider_layout<T: SliderValue>(&mut self, size: Vec2, state: &mut SliderState<T>) {
        let layout = self.get_current_layout();
        self.slider(
            Rect {
                top_left: layout.top_left,
                size,
            },
            state,
        );
        self.recompute_current_layout(size);
    }

    /// Runs `F` inside a layout, using the current or root layout.
    /// If `spacing` is `None` it will use the current layout.
    pub fn layout<F, T>(
        &mut self,
        direction: LayoutDirection,
        spacing: Option<u32>,
        with_bg: bool,
        draw: F,
    ) -> T
    where
        F: FnOnce(&mut Self) -> T,
    {
        // this layout should go wherever the current layout is
        // @TODO izzy: add a "layout_at" fn

        // ensure background is drawn first
        let mut bg_idx = None;
        if with_bg {
            let idx = self.rect_raw(
                Rect {
                    top_left: self.get_current_layout().top_left,
                    size: Vec2::zero(), // temp
                },
                flags::NONE,
                UIDrawRole::LayoutBackground,
            );
            bg_idx = Some(idx);
        }

        // push a new layout based on the current layout position
        let layout = self.get_current_layout();
        self.layout_stack.push(Layout {
            direction,
            size: Vec2::zero(),
            spacing: spacing.unwrap_or(layout.spacing),
            top_left: layout.top_left,
        });
        // do the draw, then pop the layout off and recompute the prev layout
        let ret = draw(self);
        let layout = self
            .layout_stack
            .pop()
            .expect("layout: should have popped a layout");
        self.recompute_current_layout(layout.size);

        // update the background with the now-known size
        if let Some(bg_idx) = bg_idx {
            let draw_cmd = self
                .command_buffer
                .get_mut(bg_idx)
                .expect("layout: expected command buffer idx to be valid");
            match draw_cmd {
                DrawCommand::DrawRect { draw_data } => {
                    draw_data.rect.size = layout.size;
                }
                _ => unreachable!("layout: expected bg_idx to point to a rect draw"),
            }
        }

        ret
    }

    /// Draws a rectange the size of the current layout
    pub fn layout_rect(&mut self) {
        let layout = self.get_current_layout();
        let rect: Rect = (*layout).into();
        self.rect_raw(rect, flags::NONE, UIDrawRole::LayoutBackground);
    }

    /// Finalize the computation of the UI and return the resulting state and draw info
    pub fn end(mut self) -> UIResult {
        // mouse down over hover => active
        if self.mouse_primary_button == ButtonState::Down {
            if self.state.active_rect != self.hover_rect {
                self.state.active_drag_amt = 0.0;
            }
            self.state.active_rect = self.hover_rect;
        } else {
            self.state.active_rect = None;
            self.state.active_drag_amt = 0.0;
        }

        self.state.last_mouse_position = self.mouse_position;
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
        ctx.layout(LayoutDirection::Horizontal, Some(4), false, |ctx| {
            let main_layout = *ctx.get_current_layout();
            for i in 0..3 {
                let label = format!("Section {}", i);
                assert!(
                    label.len() as u32 == SECTION_TEXT_LEN,
                    "broken test assertion"
                );

                ctx.text_layout(label);

                for j in 0..2 {
                    let sub_label = format!("Section {} item {}", i, j);

                    ctx.layout(LayoutDirection::Vertical, Some(2), false, |ctx| {
                        ctx.text_layout(sub_label);

                        let sub_layout = ctx.get_current_layout();
                        assert_eq!(
                            sub_layout.top_left.y,
                            main_layout.top_left.y + MOCK_TEXT_HEIGHT + sub_layout.spacing
                        );
                    });
                }
            }
        });

        println!("layout {:?}", ctx.get_current_layout());

        assert_eq!(ctx.command_buffer.len(), 9);
    }

    #[test]
    fn nested_layout_size_propagates() {
        let font_info = mock_font_info();
        let ui_state = UIState::new();
        let mut ctx =
            super::UIContext::new(ui_state, &font_info, Vec2 { x: 0, y: 0 }, ButtonState::Up);

        ctx.layout(LayoutDirection::Horizontal, Some(4), false, |ctx| {
            let parent_before = *ctx.get_current_layout();
            let child_layout = ctx.layout(LayoutDirection::Vertical, Some(3), false, |ctx| {
                ctx.text_layout("Hi".into());
                ctx.text_layout("WiderText".into());
                *ctx.get_current_layout()
            });
            assert_eq!(child_layout.size.x, MOCK_TEXT_WIDTH * 9);
            assert_eq!(child_layout.size.y, MOCK_TEXT_HEIGHT * 2 + 3);

            let parent_after = *ctx.get_current_layout();

            assert_eq!(
                parent_after.top_left.x,
                parent_before.top_left.x + child_layout.size.x + parent_after.spacing
            );
            assert_eq!(parent_after.size.x, child_layout.size.x);
            assert_eq!(parent_after.size.y, child_layout.size.y);
        });
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
        let clicked = ctx.button(Vec2 { x: 0, y: 0 }, Vec2 { x: 8, y: 4 }, "Click me".into());
        assert!(!clicked, "button should not register click on mouse down");
        let result = ctx.end();

        // second frame: mouse up over button
        let mut ctx = super::UIContext::new(
            result.new_state,
            &font_info,
            Vec2 { x: 10, y: 10 },
            ButtonState::Up,
        );
        let clicked = ctx.button(Vec2 { x: 0, y: 0 }, Vec2 { x: 8, y: 4 }, "Click me".into());
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
        let clicked = ctx.button(Vec2 { x: 0, y: 0 }, Vec2 { x: 8, y: 4 }, "Click me".into());
        assert!(
            !clicked,
            "button should not register click on mouse down outside"
        );
        let result = ctx.end();

        // second frame: mouse up outside button
        let mut ctx = super::UIContext::new(
            result.new_state,
            &font_info,
            Vec2 { x: 100, y: 100 },
            ButtonState::Up,
        );
        let clicked = ctx.button(Vec2 { x: 0, y: 0 }, Vec2 { x: 8, y: 4 }, "Click me".into());
        assert!(
            !clicked,
            "button should not register click on mouse up outside"
        );
    }

    #[test]
    fn slider_updates_direction_and_clamps() {
        let font_info = mock_font_info();
        let rect = Rect {
            top_left: Vec2 { x: 0, y: 0 },
            size: Vec2 { x: 100, y: 12 },
        };
        let mut slider_state = SliderState::new(0_u32, 10_u32, 5_u32, 1_u32);

        // prime the slider to become active
        let mut ctx = UIContext::new(
            UIState::new(),
            &font_info,
            Vec2 { x: 10, y: 6 },
            ButtonState::Down,
        );
        ctx.slider(rect, &mut slider_state);
        let mut state = ctx.end().new_state;

        // small motions should not cause a step yet
        let mut ctx = UIContext::new(state, &font_info, Vec2 { x: 14, y: 6 }, ButtonState::Down);
        ctx.slider(rect, &mut slider_state);
        state = ctx.end().new_state;
        assert_eq!(slider_state.value, 5);

        // accumulate enough motion to register a single step
        let mut ctx = UIContext::new(state, &font_info, Vec2 { x: 20, y: 6 }, ButtonState::Down);
        ctx.slider(rect, &mut slider_state);
        state = ctx.end().new_state;
        assert_eq!(slider_state.value, 6);

        // moving left far enough should decrease value once
        let mut ctx = UIContext::new(state, &font_info, Vec2 { x: 5, y: 6 }, ButtonState::Down);
        ctx.slider(rect, &mut slider_state);
        state = ctx.end().new_state;
        assert_eq!(slider_state.value, 5);

        // release to reset the drag accumulator
        let ctx = UIContext::new(state, &font_info, Vec2 { x: 5, y: 6 }, ButtonState::Up);
        state = ctx.end().new_state;

        // large step decrease should clamp to the minimum without crashing
        slider_state.step = 10;
        let mut ctx = UIContext::new(state, &font_info, Vec2 { x: 90, y: 6 }, ButtonState::Down);
        ctx.slider(rect, &mut slider_state);
        state = ctx.end().new_state;
        let mut ctx = UIContext::new(state, &font_info, Vec2 { x: 0, y: 6 }, ButtonState::Down);
        ctx.slider(rect, &mut slider_state);
        ctx.end();
        assert_eq!(slider_state.value, slider_state.min);
    }
}

/// Implementations of slider values for primitive numerical types
macro_rules! slider_value_impl {
    ($($t:ty),* $(,)?) => {
        $(
            impl SliderValue for $t {
                #[inline]
                fn percentage(value: Self, min: Self, max: Self) -> f32 {
                    if max == min {
                        return 0.0;
                    }
                    (value as f32 - min as f32) / (max as f32 - min as f32)
                }

                #[inline]
                fn increment(value: Self, step: Self, min: Self, max: Self) -> Self {
                    let next = value.saturating_add(step);
                    Self::clamp_value(next, min, max)
                }

                #[inline]
                fn decrement(value: Self, step: Self, min: Self, max: Self) -> Self {
                    let next = value.saturating_sub(step);
                    Self::clamp_value(next, min, max)
                }

                #[inline]
                fn clamp_value(value: Self, min: Self, max: Self) -> Self {
                    value.clamp(min, max)
                }

                #[inline]
                fn step_percentage(step: Self, min: Self, max: Self) -> f32 {
                    if step == 0 || max == min {
                        return 0.0;
                    }
                    let range = (max as f32 - min as f32).abs();
                    if range == 0.0 {
                        0.0
                    } else {
                        (step as f32).abs() / range
                    }
                }
            }
        )*
    };
}

macro_rules! slider_value_impl_floating {
    ($($t:ty),* $(,)?) => {
        $(
            impl SliderValue for $t {
                #[inline]
                fn percentage(value: Self, min: Self, max: Self) -> f32 {
                    if max == min {
                        return 0.0;
                    }

                    ((value - min) / (max - min)) as f32
                }

                #[inline]
                fn increment(value: Self, step: Self, min: Self, max: Self) -> Self {
                    Self::clamp_value(value + step, min, max)
                }

                #[inline]
                fn decrement(value: Self, step: Self, min: Self, max: Self) -> Self {
                    Self::clamp_value(value - step, min, max)
                }

                #[inline]
                fn clamp_value(value: Self, min: Self, max: Self) -> Self {
                    value.clamp(min, max)
                }

                #[inline]
                fn step_percentage(step: Self, min: Self, max: Self) -> f32 {
                    if step == 0.0 || max == min {
                        return 0.0;
                    }
                    let range = max - min;
                    if range == 0.0 {
                        0.0
                    } else {
                        (step / range).abs() as f32
                    }
                }
            }
        )*
    };
}

slider_value_impl!(i8, u8, i16, u16, i32, u32, i64, u64);
slider_value_impl_floating!(f32, f64);
