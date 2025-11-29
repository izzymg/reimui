// Some common reimui -> raylib bindings
use raylib::prelude::*;
pub mod class_lists;
pub mod checkbox;
pub mod layouts;
pub mod simple;
pub mod slider;

pub trait SampleUI {
    fn draw(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread);
}

pub struct RaylibFontInfo {
    pub font_size: i32,
    font: WeakFont,
}

impl RaylibFontInfo {
    pub fn new(rl: &RaylibHandle) -> Self {
        let font = rl.get_font_default();
        let font_size = font.baseSize;
        Self { font_size, font }
    }
}

impl reimui::FontInformation for RaylibFontInfo {
    fn compute_text_size(&self, text: &str, scale: f32) -> reimui::Vec2 {
        let font_size = self.font_size as f32 * scale;
        let text_size = self.font.measure_text(text, font_size, 1.0);
        reimui::Vec2 {
            x: text_size.x.ceil() as u32,
            y: text_size.y.ceil() as u32,
        }
    }
}

/// A simple way to implement a color palette by examining the role hint of the draw command and its set flags.
pub fn color_palette(
    role: reimui::UIDrawRole,
    flags: reimui::flags::Flags,
    class_list: Option<reimui::ClassList>,
) -> Color {
    let is_active = flags & reimui::flags::ACTIVE != 0;
    let is_hover = flags & reimui::flags::HOVER != 0;
    let is_focus = flags & reimui::flags::FOCUSED != 0;
    let has_class = |tag: &'static str| class_list.is_some_and(|cls| cls.has(tag));
    let mut color = match role {
        reimui::UIDrawRole::Text => {
            if is_active {
                Color::WHITE
            } else if is_hover {
                Color::RED
            } else {
                Color::BLACK
            }
        }
        reimui::UIDrawRole::ButtonBackground => {
            if is_active {
                Color::DARKBLUE
            } else if is_hover {
                Color::LIGHTBLUE
            } else if is_focus {
                Color::BLUE
            } else {
                Color::BLUEVIOLET
            }
        }
        reimui::UIDrawRole::ButtonText => {
            if is_active || is_hover {
                Color::WHITE
            } else {
                Color::BLACK
            }
        }
        reimui::UIDrawRole::SliderKnob => if is_focus {
            Color::DARKBLUE
        } else {
            Color::BLUE
        },
        reimui::UIDrawRole::SliderRect => Color::GRAY,
        reimui::UIDrawRole::CheckboxBox => {
            if is_active {
                Color::DARKGRAY
            } else if is_hover {
                Color::LIGHTGRAY
            } else {
                Color::GRAY
            }
        }
        reimui::UIDrawRole::CheckboxCheck => Color::DARKBLUE,
        reimui::UIDrawRole::LayoutBackground => Color::GREEN,
    };

    if matches!(role, reimui::UIDrawRole::LayoutBackground) && has_class("panel") {
        color = Color::LIGHTGRAY;
    }

    if matches!(role, reimui::UIDrawRole::Text | reimui::UIDrawRole::ButtonText) {
        if has_class("muted") {
            color = Color::DARKGRAY;
        }
        if has_class("accent") {
            color = Color::DARKBLUE;
        }
    }

    if has_class("danger") {
        match role {
            reimui::UIDrawRole::ButtonBackground => {
                color = if is_active {
                    Color::MAROON
                } else if is_hover {
                    Color::RED
                } else {
                    Color::ORANGE
                };
            }
            reimui::UIDrawRole::ButtonText => color = Color::WHITE,
            _ => {}
        }
    }

    color
}

/// Applies the result of a reimui draw to raylib
pub fn apply_reimui_to_raylib(
    ui_result: &reimui::UIResult,
    d: &mut RaylibDrawHandle,
    font_info: &RaylibFontInfo,
) {
    for command in &ui_result.commands {
        match command {
            reimui::DrawCommand::DrawText {
                content,
                draw_data,
                text_scale,
            } => {
                let font_size = ((font_info.font_size as f32) * text_scale).max(1.0);
                d.draw_text(
                    content,
                    draw_data.rect.top_left.x as i32,
                    draw_data.rect.top_left.y as i32,
                    font_size.ceil() as i32,
                    color_palette(draw_data.role, draw_data.flags, draw_data.class_list),
                );
            }
            reimui::DrawCommand::DrawRect { draw_data } => {
                d.draw_rectangle(
                    draw_data.rect.top_left.x as i32,
                    draw_data.rect.top_left.y as i32,
                    draw_data.rect.size.x as i32,
                    draw_data.rect.size.y as i32,
                    color_palette(draw_data.role, draw_data.flags, draw_data.class_list),
                );
            }
        }
    }
}

/// Collect the raylib input state and map it into reimui's expected input format.
pub fn raylib_input_state(
    rl: &mut RaylibHandle,
    ui_state: &reimui::UIState,
) -> reimui::UIInputState {
    let mouse = rl.get_mouse_position();
    let activate_button = if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
        reimui::ButtonState::Down
    } else {
        reimui::ButtonState::Up
    };

    let mut input = reimui::UIInputState {
        mouse_position: reimui::Vec2 {
            x: mouse.x.max(0.0) as u32,
            y: mouse.y.max(0.0) as u32,
        },
        activate_button,
        focus_next_button: if rl.is_key_pressed(KeyboardKey::KEY_TAB) {
            reimui::ButtonState::Down
        } else {
            reimui::ButtonState::Up
        },
    };

    // Allow pressing enter to "click" the currently focused control.
    if rl.is_key_pressed(KeyboardKey::KEY_ENTER) || rl.is_key_pressed(KeyboardKey::KEY_KP_ENTER) {
        if let Some(focused_rect) = ui_state.focused_rect() {
            input.mouse_position = focused_rect.top_left;
            input.activate_button = reimui::ButtonState::Down;
        }
    }

    input
}
