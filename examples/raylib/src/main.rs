use std::ffi::CString;

use raylib::{ prelude::*};

use reimui::{
    ButtonState, DrawCommand, FontInformation, Layout, LayoutDirection, UIContext, Vec2,
};
 
struct RaylibFontInfo {
    font_size: i32,
}

impl FontInformation for RaylibFontInfo {
    fn compute_text_size(&self, text: &str) -> Vec2 {
        let text = CString::new(text).unwrap_or_default();
        let width = unsafe { raylib::ffi::MeasureText(text.as_ptr(), self.font_size) } as u32;
        Vec2 {
            x: width,
            y: self.font_size as u32,
        }
    }
}

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(640, 400)
        .title("reimui + raylib")
        .build();

    let font_info = RaylibFontInfo { font_size: 22 };
    let mut ui_state = reimui::UIState::new();
    let mut click_count = 0;

    while !rl.window_should_close() {
        let mouse = rl.get_mouse_position();
        let mouse_state = if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
            ButtonState::Down
        } else {
            ButtonState::Up
        };

        let mut ui_ctx = UIContext::new(
            ui_state,
            &font_info,
            Vec2 {
                x: mouse.x.max(0.0) as u32,
                y: mouse.y.max(0.0) as u32,
            },
            mouse_state,
        );

        let mut layout = Layout::new(
            LayoutDirection::Vertical,
            25,
            Vec2 { x: 28, y: 28 },
            Vec2 { x: 620, y: 360 },
        );
        ui_ctx.draw_text_layout(&mut layout, "reimui + raylib".into());
        ui_ctx.draw_text_layout(
            &mut layout,
            "Immediate mode UI rendering to raylib".into(),
        );
        let clicked = ui_ctx.draw_button_layout(&mut layout, Vec2 { x: 18, y: 12 }, format!("Click me {}", click_count).into());

        if clicked {
            click_count += 1;
        }

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::RAYWHITE);

        let ui_result = ui_ctx.end();
        for command in ui_result.commands {
            match command {
                DrawCommand::DrawText { content, top_left, flags } => {
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
                        }
                    );
                }
            }
        }


        // make sure to reassign the update UI state for the next pass
        ui_state = ui_result.new_state;
    }
}
