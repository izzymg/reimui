// Some common reimui -> raylib bindings
use raylib::prelude::*;
pub mod class_lists;
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
    fn compute_text_size(&self, text: &str) -> reimui::Vec2 {
        let text_size = self.font.measure_text(text, self.font_size as f32, 1.0);
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
        reimui::UIDrawRole::SliderKnob => Color::BLUE,
        reimui::UIDrawRole::SliderRect => Color::GRAY,
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
            reimui::DrawCommand::DrawText { content, draw_data } => {
                d.draw_text(
                    content,
                    draw_data.rect.top_left.x as i32,
                    draw_data.rect.top_left.y as i32,
                    font_info.font_size,
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
