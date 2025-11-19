// Some common reimui -> raylib bindings
use raylib::prelude::*;
pub mod slider;
pub mod simple;


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
    fn compute_text_size(&self, text: &str) -> reimui::Vec2 {
        let text_size = self.font.measure_text(text, self.font_size as f32, 1.0);
        reimui::Vec2 {
            x: text_size.x as u32,
            y: text_size.y as u32,
        }
    }
}

/// A simple way to implement a color palette by examining the role hint of the draw command and its set flags.
pub fn color_palette(role: reimui::UIDrawRole, flags: reimui::flags::Flags) -> Color {
    let is_active = flags & reimui::flags::ACTIVE != 0;
    let is_hover = flags & reimui::flags::HOVER != 0;
    match role {
        reimui::UIDrawRole::Text => {
            if is_active {
                Color::WHITE
            } else if is_hover {
                Color::BLACK
            } else {
                Color::DARKGRAY
            }
        }
        reimui::UIDrawRole::ButtonBackground => {
            if is_active {
                Color::DARKBLUE
            } else if is_hover {
                Color::LIGHTBLUE
            } else {
                Color::BLUE
            }
        }
        reimui::UIDrawRole::ButtonText => {
            if is_active || is_hover {
                Color::WHITE
            } else {
                Color::BLACK
            }
        }
        reimui::UIDrawRole::SliderKnob => {
            Color::BLUE
        }
        reimui::UIDrawRole::SliderRect => {
            Color::GRAY
        }
    }
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
                top_left,
                flags,
                role,
            } => {
                d.draw_text(
                    content,
                    top_left.x as i32,
                    top_left.y as i32,
                    font_info.font_size,
                    color_palette(*role, *flags),
                );
            }
            reimui::DrawCommand::DrawRect {
                top_left,
                size,
                flags,
                role,
            } => {
                d.draw_rectangle(
                    top_left.x as i32,
                    top_left.y as i32,
                    size.x as i32,
                    size.y as i32,
                    color_palette(*role, *flags),
                );
            }
        }
    }
}
