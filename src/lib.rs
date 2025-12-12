// Stupidly simple render-agnostic immediate mode UI lib

use crate::flags::Flags;
use std::{collections::VecDeque, ops::Range};

pub mod prelude {
    pub use super::{
        ButtonState, FontInformation, Layout, LayoutDirection, Rect, UIContext, UIDrawRole,
        UIInputState, UIState, Vec2,
    };
}

#[rustfmt::skip]
pub mod flags {
    pub type Flags = u32;
    pub const NONE: Flags           = 0;
    pub const HOVER: Flags          = 1 << 0;
    pub const DISABLED: Flags       = 1 << 1;
    pub const ACTIVE: Flags         = 1 << 2;
    pub const FOCUSED: Flags        = 1 << 2;
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

#[allow(clippy::should_implement_trait, reason = "No operator overloading")]
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

    /// Component-wise division
    pub fn div_cmp(a: Vec2, b: u32) -> Self {
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
    pub class_list: Option<ClassList>,
}

/// The output of a reimui ui run
#[derive(Debug, Clone)]
pub enum DrawCommand {
    DrawText {
        content: String,
        text_scale: f32,
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
    fn compute_text_size(&self, text: &str, scale: f32) -> Vec2;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Hint about what a draw command is for
pub enum UIDrawRole {
    Text,
    ButtonText,
    ButtonBackground,
    SliderRect,
    SliderKnob,
    CheckboxBox,
    CheckboxCheck,
    LayoutBackground,
}

/// Tiny wrapper for an assumed-space-separated list of classes/tags.
/// Allows for unique styling of UI elements (e.g. "cool-btn text-red").
/// Fast enough if not used excessively.
#[derive(Copy, Clone, Debug)]
pub struct ClassList {
    pub classes: &'static str,
}

impl ClassList {
    pub fn new(tags: &'static str) -> Self {
        Self { classes: tags }
    }

    /// Returns true if these tags contain the given tag, by separating on whitespace.
    pub fn has(&self, tag: &'static str) -> bool {
        self.classes.split_whitespace().any(|t| t.eq(tag))
    }
}

impl PartialEq for ClassList {
    fn eq(&self, other: &Self) -> bool {
        self.classes.eq(other.classes)
    }
}

#[derive(Copy, Clone)]
/// Persistent UI state object
pub struct UIState {
    active_rect: Option<Rect>,
    last_mouse_position: Vec2,
    active_drag_amt: f32,
    focused: Option<Rect>,
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
            focused: None,
        }
    }

    pub fn focused_rect(&self) -> Option<Rect> {
        self.focused
    }
}

/// Result of a computation of the UI
pub struct UIResult {
    pub new_state: UIState,
    pub commands: Vec<DrawCommand>,
}

/// Tell reimui what's going on with your user's physical inputs
pub struct UIInputState {
    /// 0,0 top left
    pub mouse_position: Vec2,

    pub activate_button: ButtonState,
    pub focus_next_button: ButtonState,
}

impl Default for UIInputState {
    /// All buttons up, mouse at 0,0
    fn default() -> Self {
        Self {
            mouse_position: Vec2::zero(),
            activate_button: ButtonState::Up,
            focus_next_button: ButtonState::Up,
        }
    }
}

#[derive(Copy, Clone, Debug)]
/// Data about what happened to draw a checkbox
pub struct CheckboxResult { 
    pub rect: Rect,
    pub interacted: bool,
}

/// Transient draw context
pub struct UIContext<'f> {
    state: UIState,
    font_info: &'f dyn FontInformation,
    input_state: UIInputState,

    hover_rect: Option<Rect>,

    command_buffer: VecDeque<DrawCommand>,

    layout_stack: Vec<Layout>,

    next_class: Option<ClassList>,

    focusables: Vec<Rect>,
}

impl<'f> UIContext<'f> {
    pub fn new(
        state: UIState,
        font_info: &'f dyn FontInformation,
        input_state: UIInputState,
    ) -> Self {
        let initial_layout_stack = vec![Layout::new(
            LayoutDirection::Horizontal,
            0,
            Vec2::zero(),
            Vec2::zero(),
        )];

        Self {
            command_buffer: VecDeque::new(),
            input_state,
            hover_rect: None,
            state,
            font_info,
            layout_stack: initial_layout_stack,
            next_class: None,
            focusables: vec![],
        }
    }

    pub fn new_layout_init(
        state: UIState,
        font_info: &'f dyn FontInformation,
        input_state: UIInputState,
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
            input_state,
            hover_rect: None,
            state,
            font_info,
            layout_stack: initial_layout_stack,
            next_class: None,
            focusables: vec![],
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

    pub fn register_focusable(&mut self, rect: Rect) -> bool {
        self.focusables.push(rect);
        self.state.focused.is_some_and(|r| r == rect)
    }

    /// Sets the classlist used for all draws to `class_list`.
    /// Clear it using `clear_class_list` or see `with_class_list`.
    pub fn set_class_list(&mut self, class_list: ClassList) {
        self.next_class = Some(class_list);
    }

    /// Clears the currently used class list for drawing.
    /// Set it using `set_class_list` or see `with_class_list`.
    pub fn clear_class_list(&mut self) {
        self.next_class = None;
    }

    /// Executes `func` providing this UI context and returning its result, with the `class_list` set for the duration of the call.
    pub fn with_class_list<F, T>(&mut self, class_list: ClassList, func: F) -> T
    where
        F: FnOnce(&mut Self) -> T,
    {
        self.set_class_list(class_list);
        let ret = func(self);
        self.clear_class_list();
        ret
    }

    /// Returns the index into the command buffer of this draw
    pub fn rect_raw(&mut self, rect: Rect, flags: Flags, role: UIDrawRole) -> usize {
        let idx = self.command_buffer.len();
        self.command_buffer.push_back(DrawCommand::DrawRect {
            draw_data: DrawData {
                rect,
                flags,
                role,
                class_list: self.next_class,
            },
        });
        idx
    }

    pub fn text_raw(
        &mut self,
        label: String,
        rect: Rect,
        flags: Flags,
        role: UIDrawRole,
        scale: f32,
    ) {
        self.command_buffer.push_back(DrawCommand::DrawText {
            content: label,
            text_scale: scale,
            draw_data: DrawData {
                rect,
                flags,
                role,
                class_list: self.next_class,
            },
        });
    }

    pub fn button_raw(
        &mut self,
        top_left: Vec2,
        text_size: Vec2,
        padding: Vec2,
        label: String,
        text_scale: f32,
    ) -> bool {
        let button_size = Vec2::add(text_size, padding);
        let rect = Rect {
            top_left,
            size: button_size,
        };

        let hovered = self.check_set_hover(rect);
        let active = self.is_active(rect);
        let focused = self.register_focusable(rect);

        let mut flags = flags::NONE;
        if hovered {
            flags |= flags::HOVER;
        }
        if active {
            flags |= flags::ACTIVE;
        }
        if focused {
            flags |= flags::FOCUSED;
        }

        let half_padding = Vec2::div_cmp(Vec2::sub(rect.size, text_size), 2);
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
            text_scale,
        );

        (hovered || focused) && self.clicked_rect(rect)
    }

    fn is_active(&self, rect: Rect) -> bool {
        self.state
            .active_rect
            .is_some_and(|active_rect| active_rect == rect)
    }

    fn clicked_rect(&self, rect: Rect) -> bool {
        self.input_state.activate_button == ButtonState::Up && self.is_active(rect)
    }

    fn check_set_hover(&mut self, rect: Rect) -> bool {
        let is_hover = rect.contains(self.input_state.mouse_position);
        if is_hover {
            self.hover_rect = Some(rect);
        }

        is_hover
    }

    pub fn text(&mut self, label: String, rect: Rect) {
        self.text_scaled(label, rect, 1.0);
    }

    pub fn text_scaled(&mut self, label: String, rect: Rect, scale: f32) {
        self.text_raw(label, rect, flags::NONE, UIDrawRole::Text, scale);
    }

    pub fn text_layout(&mut self, label: String) -> Vec2 {
        self.text_layout_scaled(label, 1.0)
    }

    /// Returns the size of the text
    pub fn text_at(&mut self, label: String, position: Vec2) -> Vec2 {
        let text_size = self.font_info.compute_text_size(&label, 1.0);
        self.text(label, Rect {
            size: text_size,
            top_left: position
        });
        text_size
    }

    pub fn text_at_scaled(&mut self, label: String, position: Vec2, scale: f32) -> Vec2 {
        let text_size = self.font_info.compute_text_size(&label, scale);
        self.text(label, Rect {
            size: text_size,
            top_left: position
        });
        text_size
    }

    pub fn text_layout_scaled(&mut self, label: String, scale: f32) -> Vec2 {
        let layout = self.get_current_layout();
        let text_size = self.font_info.compute_text_size(&label, scale);
        self.text_scaled(
            label,
            Rect {
                size: text_size,
                top_left: layout.top_left,
            },
            scale,
        );
        self.recompute_current_layout(text_size);
        text_size
    }

    pub fn button(&mut self, top_left: Vec2, padding: Vec2, label: String) -> bool {
        self.button_scaled(top_left, padding, label, 1.0)
    }

    pub fn button_scaled(
        &mut self,
        top_left: Vec2,
        padding: Vec2,
        label: String,
        scale: f32,
    ) -> bool {
        let text_size = self.font_info.compute_text_size(&label, scale);
        self.button_raw(top_left, text_size, padding, label, scale)
    }

    pub fn button_layout(&mut self, padding: Vec2, label: String) -> bool {
        self.button_layout_scaled(padding, label, 1.0)
    }

    pub fn button_layout_scaled(&mut self, padding: Vec2, label: String, scale: f32) -> bool {
        let layout = self.get_current_layout();
        let text_size = self.font_info.compute_text_size(&label, scale);
        let clicked = self.button_raw(layout.top_left, text_size, padding, label, scale);
        self.recompute_current_layout(Vec2::add(text_size, padding));
        clicked
    }

    /// Draws a checkbox at `top_left` with a given box `size`.
    /// Returns true when the checkbox toggles, and mutates the caller-held `checked` value.
    pub fn checkbox(&mut self, top_left: Vec2, size: Vec2, checked: &mut bool) -> bool {
        let rect = Rect { top_left, size };

        let hovered = self.check_set_hover(rect);
        let active = self.is_active(rect);
        let focused = self.register_focusable(rect);

        let mut flags = flags::NONE;
        if hovered {
            flags |= flags::HOVER;
        }
        if active {
            flags |= flags::ACTIVE;
        }
        if focused {
            flags |= flags::FOCUSED;
        }

        let toggled = (hovered || focused) && self.clicked_rect(rect);
        if toggled {
            *checked = !*checked;
        }

        self.rect_raw(rect, flags, UIDrawRole::CheckboxBox);

        if *checked {
            let inset = Vec2::new(size.x / 4, size.y / 4);
            let check_top_left = Vec2::add(rect.top_left, inset);
            let check_size = Vec2 {
                x: size.x.saturating_sub(inset.x.saturating_mul(2)),
                y: size.y.saturating_sub(inset.y.saturating_mul(2)),
            };
            self.rect_raw(
                Rect {
                    top_left: check_top_left,
                    size: check_size,
                },
                flags,
                UIDrawRole::CheckboxCheck,
            );
        }

        toggled
    }

    /// Draws a checkbox using the current layout position.
    pub fn checkbox_layout(&mut self, size: Vec2, checked: &mut bool) -> CheckboxResult {
        let top_left = self.get_current_layout().top_left;
        let toggled = self.checkbox(top_left, size, checked);
        self.recompute_current_layout(size);
        CheckboxResult {
            interacted: toggled,
            rect: Rect { top_left, size }
        }
    }

    /// Draws a checkbox using the current layout, and `label` centered on the left.
    pub fn checkbox_layout_label_left(&mut self, size: Vec2, checked: &mut bool, label: String, label_scale: f32) -> bool {
        self.layout(LayoutDirection::Horizontal, None, false, |ui| {
            let layout = *ui.get_current_layout();
            // add half the size y to center the text
            let label_top_left = Vec2::add(layout.top_left, Vec2::new(0, size.y / 4));
            let text_size = ui.text_at_scaled(label, label_top_left, label_scale);
            ui.recompute_current_layout(text_size);

            // now draw checkbox next to it

            let interacted = ui.checkbox_layout(size, checked);
            interacted.interacted
        })
    }

    pub fn slider<T: SliderValue>(&mut self, rect: Rect, state: &mut SliderState<T>) {
        let hovered = self.check_set_hover(rect);
        let is_active = self.is_active(rect);
        let focused = self.register_focusable(rect);
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
            let delta_x =
                self.input_state.mouse_position.x as f32 - self.state.last_mouse_position.x as f32;
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
                    self.state.active_drag_amt -= steps * pixels_per_step;
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
        if focused {
            flags |= flags::FOCUSED;
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

    /// Runs `F` inside a layout, using the current layout.
    /// If `spacing` is `None` it will use the current layout spacing.
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
        let current_layout = self.get_current_layout();
        self.layout_at(
            current_layout.top_left,
            direction,
            spacing.unwrap_or(current_layout.spacing),
            with_bg,
            draw,
        )
    }

    /// Runs `F` inside a layout, using the provided position.
    pub fn layout_at<F, T>(
        &mut self,
        top_left: Vec2,
        direction: LayoutDirection,
        spacing: u32,
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
                    top_left,
                    size: Vec2::zero(), // temp
                },
                flags::NONE,
                UIDrawRole::LayoutBackground,
            );
            bg_idx = Some(idx);
        }

        // push a new layout based on the current layout position
        self.layout_stack.push(Layout {
            direction,
            size: Vec2::zero(),
            spacing,
            top_left,
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
        // mouse/key down over hover/focus => active
        if self.input_state.activate_button == ButtonState::Down {
            let target_rect = self.hover_rect.or(self.state.focused);
            if self.state.active_rect != target_rect {
                self.state.active_drag_amt = 0.0;
            }
            self.state.active_rect = target_rect;
        } else {
            self.state.active_rect = None;
            self.state.active_drag_amt = 0.0;
        }

        // figure out what the next thing to focus is
        if self.input_state.focus_next_button == ButtonState::Down {
            // if we had something focused, we find the next one
            if let Some(prev_focus_rect) = self.state.focused {
                let next_idx = self
                    .focusables
                    .iter()
                    .copied()
                    .position(|r| r == prev_focus_rect)
                    .map(|p| p + 1)
                    .unwrap_or_default();
                let next_rect = self
                    .focusables
                    .get(next_idx)
                    .or_else(|| self.focusables.first())
                    .copied();
                self.state.focused = next_rect;
            } else {
                self.state.focused = self.focusables.first().copied();
            }
        }

        self.state.last_mouse_position = self.input_state.mouse_position;
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
            fn compute_text_size(&self, text: &str, scale: f32) -> Vec2 {
                let scale = scale.max(0.0);
                Vec2 {
                    x: (text.len() as f32 * MOCK_TEXT_WIDTH as f32 * scale).ceil() as u32,
                    y: (MOCK_TEXT_HEIGHT as f32 * scale).ceil() as u32,
                }
            }
        }
        MockFontInfo
    }

    #[test]
    fn layout() {
        const SECTION_TEXT_LEN: u32 = 9;

        let input_state = UIInputState {
            activate_button: ButtonState::Up,
            focus_next_button: ButtonState::Up,
            mouse_position: Vec2::zero(),
        };

        let font_info = mock_font_info();
        let ui_state = UIState::new();
        let mut ctx = super::UIContext::new(ui_state, &font_info, input_state);
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
        let mut ctx = super::UIContext::new(ui_state, &font_info, UIInputState::default());

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
    fn layout_at_uses_given_position_for_background() {
        let font_info = mock_font_info();
        let mut ctx = super::UIContext::new(UIState::new(), &font_info, UIInputState::default());

        let layout_pos = Vec2 { x: 20, y: 30 };
        ctx.layout_at(layout_pos, LayoutDirection::Vertical, 2, true, |ctx| {
            ctx.text_layout("abc".into())
        });

        assert_eq!(ctx.command_buffer.len(), 2);
        match &ctx.command_buffer[0] {
            DrawCommand::DrawRect { draw_data } => {
                assert_eq!(draw_data.rect.top_left, layout_pos);
                assert_eq!(
                    draw_data.rect.size,
                    Vec2 {
                        x: MOCK_TEXT_WIDTH * 3,
                        y: MOCK_TEXT_HEIGHT
                    }
                );
            }
            _ => panic!("expected layout background to be a rect draw"),
        }
    }

    #[test]
    fn button_click() {
        let font_info = mock_font_info();
        let ui_state = UIState::new();

        // first frame: mouse down over button
        let mut ctx = super::UIContext::new(
            ui_state,
            &font_info,
            UIInputState {
                activate_button: ButtonState::Down,
                ..Default::default()
            },
        );
        let clicked = ctx.button(Vec2 { x: 0, y: 0 }, Vec2 { x: 8, y: 4 }, "Click me".into());
        assert!(!clicked, "button should not register click on mouse down");
        let result = ctx.end();

        // second frame: mouse up over button
        let mut ctx = super::UIContext::new(
            result.new_state,
            &font_info,
            UIInputState {
                activate_button: ButtonState::Up,
                ..Default::default()
            },
        );
        let clicked = ctx.button(Vec2 { x: 0, y: 0 }, Vec2 { x: 8, y: 4 }, "Click me".into());
        assert!(clicked, "button should register click on mouse up");
    }

    #[test]
    fn button_click_outside() {
        let font_info = mock_font_info();
        let ui_state = UIState::new();

        let input_state = UIInputState {
            mouse_position: Vec2 { x: 100, y: 100 },
            activate_button: ButtonState::Down,
            ..Default::default()
        };

        // first frame: mouse down outside button
        let mut ctx = super::UIContext::new(ui_state, &font_info, input_state);
        let clicked = ctx.button(Vec2 { x: 0, y: 0 }, Vec2 { x: 8, y: 4 }, "Click me".into());
        assert!(
            !clicked,
            "button should not register click on mouse down outside"
        );
        let result = ctx.end();

        let input_state = UIInputState {
            mouse_position: Vec2 { x: 100, y: 100 },
            ..Default::default()
        };

        // second frame: mouse up outside button
        let mut ctx = super::UIContext::new(result.new_state, &font_info, input_state);
        let clicked = ctx.button(Vec2 { x: 0, y: 0 }, Vec2 { x: 8, y: 4 }, "Click me".into());
        assert!(
            !clicked,
            "button should not register click on mouse up outside"
        );
    }

    #[test]
    fn enter_key_activates_focused_button() {
        let font_info = mock_font_info();
        let button_padding = Vec2::zero();
        let button_pos = Vec2::zero();
        let mouse_far = Vec2 { x: 999, y: 999 };

        // focus the button
        let mut ctx = super::UIContext::new(
            UIState::new(),
            &font_info,
            UIInputState {
                focus_next_button: ButtonState::Down,
                mouse_position: mouse_far,
                ..Default::default()
            },
        );
        ctx.button(button_pos, button_padding, "A".into());
        let result = ctx.end();

        // key down should mark it active but not click yet
        let mut ctx = super::UIContext::new(
            result.new_state,
            &font_info,
            UIInputState {
                activate_button: ButtonState::Down,
                mouse_position: mouse_far,
                ..Default::default()
            },
        );
        let clicked = ctx.button(button_pos, button_padding, "A".into());
        assert!(
            !clicked,
            "activate key down alone should not register a click"
        );
        let state = ctx.end().new_state;

        // releasing the key should click the focused button even without hover
        let mut ctx = super::UIContext::new(
            state,
            &font_info,
            UIInputState {
                activate_button: ButtonState::Up,
                mouse_position: mouse_far,
                ..Default::default()
            },
        );
        let clicked = ctx.button(button_pos, button_padding, "A".into());
        assert!(clicked, "activate key up should click the focused button");
    }

    #[test]
    fn tab_focus_advances_through_focusables() {
        let font_info = mock_font_info();
        let button_padding = Vec2::zero();
        let first_button_pos = Vec2::zero();
        let second_button_pos = Vec2 { x: 50, y: 0 };

        // first tab press should focus the first registered control
        let mut ctx = super::UIContext::new(
            UIState::new(),
            &font_info,
            UIInputState {
                focus_next_button: ButtonState::Down,
                ..Default::default()
            },
        );
        ctx.button(first_button_pos, button_padding, "A".into());
        ctx.button(second_button_pos, button_padding, "B".into());
        let result = ctx.end();
        assert_eq!(result.new_state.focused.unwrap().top_left, first_button_pos);

        // next press should advance to the next focusable
        let mut ctx = super::UIContext::new(
            result.new_state,
            &font_info,
            UIInputState {
                focus_next_button: ButtonState::Down,
                ..Default::default()
            },
        );
        ctx.button(first_button_pos, button_padding, "A".into());
        ctx.button(second_button_pos, button_padding, "B".into());
        let result = ctx.end();
        assert_eq!(
            result.new_state.focused.unwrap().top_left,
            second_button_pos
        );

        // pressing again should wrap back to the first
        let mut ctx = super::UIContext::new(
            result.new_state,
            &font_info,
            UIInputState {
                focus_next_button: ButtonState::Down,
                ..Default::default()
            },
        );
        ctx.button(first_button_pos, button_padding, "A".into());
        ctx.button(second_button_pos, button_padding, "B".into());
        let result = ctx.end();
        assert_eq!(result.new_state.focused.unwrap().top_left, first_button_pos);
    }

    #[test]
    fn slider_updates_direction_and_clamps() {
        let font_info = mock_font_info();
        let rect = Rect {
            top_left: Vec2 { x: 0, y: 0 },
            size: Vec2 { x: 100, y: 12 },
        };
        let mut slider_state = SliderState::new(0_u32, 10_u32, 5_u32, 1_u32);

        let input_state = UIInputState {
            mouse_position: Vec2 { x: 10, y: 6 },
            activate_button: ButtonState::Down,
            ..Default::default()
        };

        // prime the slider to become active
        let mut ctx = UIContext::new(UIState::new(), &font_info, input_state);
        ctx.slider(rect, &mut slider_state);
        let mut state = ctx.end().new_state;

        // small motions should not cause a step yet
        let mut ctx = UIContext::new(
            state,
            &font_info,
            UIInputState {
                mouse_position: Vec2::new(14, 6),
                activate_button: ButtonState::Down,
                ..Default::default()
            },
        );
        ctx.slider(rect, &mut slider_state);
        state = ctx.end().new_state;
        assert_eq!(slider_state.value, 5);

        // accumulate enough motion to register a single step

        let mut ctx = UIContext::new(
            state,
            &font_info,
            UIInputState {
                mouse_position: Vec2::new(20, 6),
                activate_button: ButtonState::Down,
                ..Default::default()
            },
        );
        ctx.slider(rect, &mut slider_state);
        state = ctx.end().new_state;
        assert_eq!(slider_state.value, 6);

        // moving left far enough should decrease value once
        let mut ctx = UIContext::new(
            state,
            &font_info,
            UIInputState {
                mouse_position: Vec2 { x: 5, y: 6 },
                activate_button: ButtonState::Down,
                ..Default::default()
            },
        );
        ctx.slider(rect, &mut slider_state);
        state = ctx.end().new_state;
        assert_eq!(slider_state.value, 5);

        // release to reset the drag accumulator
        let ctx = UIContext::new(
            state,
            &font_info,
            UIInputState {
                mouse_position: Vec2 { x: 5, y: 6 },
                activate_button: ButtonState::Up,
                ..Default::default()
            },
        );
        state = ctx.end().new_state;

        // large step decrease should clamp to the minimum without crashing
        slider_state.step = 10;
        let mut ctx = UIContext::new(
            state,
            &font_info,
            UIInputState {
                mouse_position: Vec2 { x: 90, y: 6 },
                activate_button: ButtonState::Down,
                ..Default::default()
            },
        );
        ctx.slider(rect, &mut slider_state);
        state = ctx.end().new_state;
        let mut ctx = UIContext::new(
            state,
            &font_info,
            UIInputState {
                mouse_position: Vec2 { x: 0, y: 6 },
                activate_button: ButtonState::Down,
                ..Default::default()
            },
        );
        ctx.slider(rect, &mut slider_state);
        ctx.end();
        assert_eq!(slider_state.value, slider_state.min);
    }

    #[test]
    fn checkbox_toggles_and_draws_check() {
        let font_info = mock_font_info();
        let rect = Rect {
            top_left: Vec2 { x: 0, y: 0 },
            size: Vec2 { x: 20, y: 20 },
        };
        let mut checked = false;

        // press down over the box
        let mut ctx = UIContext::new(
            UIState::new(),
            &font_info,
            UIInputState {
                mouse_position: Vec2 { x: 10, y: 10 },
                activate_button: ButtonState::Down,
                ..Default::default()
            },
        );
        let toggled = ctx.checkbox(rect.top_left, rect.size, &mut checked);
        assert!(!toggled);
        assert!(!checked);
        let state = ctx.end().new_state;

        // release over the box should toggle
        let mut ctx = UIContext::new(
            state,
            &font_info,
            UIInputState {
                mouse_position: Vec2 { x: 10, y: 10 },
                activate_button: ButtonState::Up,
                ..Default::default()
            },
        );
        let toggled = ctx.checkbox(rect.top_left, rect.size, &mut checked);
        assert!(toggled);
        assert!(checked);

        // when checked, a check draw command is emitted after the box
        assert_eq!(ctx.command_buffer.len(), 2);
        match (&ctx.command_buffer[0], &ctx.command_buffer[1]) {
            (
                DrawCommand::DrawRect {
                    draw_data: box_draw,
                },
                DrawCommand::DrawRect {
                    draw_data: check_draw,
                },
            ) => {
                assert_eq!(box_draw.role, UIDrawRole::CheckboxBox);
                assert_eq!(check_draw.role, UIDrawRole::CheckboxCheck);
            }
            _ => panic!("expected two rectangle draws for checkbox"),
        }
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
