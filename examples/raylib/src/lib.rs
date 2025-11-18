// Some common reimui -> raylib bindings
use raylib::prelude::*;

pub struct RaylibFontInfo {
    pub font_size: i32,
    font: WeakFont,
}

impl RaylibFontInfo {
    pub fn new(rl: &RaylibHandle, font_size: i32) -> Self {
        Self {
            font_size,
            font: rl.get_font_default(),
        }
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
            } => {
                d.draw_text(
                    content,
                    top_left.x as i32,
                    top_left.y as i32,
                    font_info.font_size,
                    if flags & reimui::flags::ACTIVE != 0 {
                        Color::DARKRED
                    } else if flags & reimui::flags::HOVER != 0 {
                        Color::RED
                    } else {
                        Color::BLACK
                    },
                );
            }
        }
    }
}
