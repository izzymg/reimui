// Some common reimui -> raylib bindings 

use std::ffi::CString;

use raylib::prelude::*;

pub struct RaylibFontInfo {
    pub font_size: i32,
}

impl reimui::FontInformation for RaylibFontInfo {
    fn compute_text_size(&self, text: &str) -> reimui::Vec2 {
        let text = CString::new(text).unwrap_or_default();
        let width = unsafe { raylib::ffi::MeasureText(text.as_ptr(), self.font_size) } as u32;
        reimui::Vec2 {
            x: width,
            y: self.font_size as u32,
        }
    }
}

/// Applies the result of a reimui draw to raylib
pub fn apply_reimui_to_raylib(ui_result: &reimui::UIResult, d: &mut RaylibDrawHandle, font_info: &RaylibFontInfo) {
         for command in &ui_result.commands {
                match command {
                    reimui::DrawCommand::DrawText {
                        content,
                        top_left,
                        flags,
                    } => {
                        d.draw_text(
                            &content,
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