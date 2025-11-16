use std::ffi::CString;

use raylib::prelude::*;

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
        .title("r e i m u i")
        .build();

    let font_info = RaylibFontInfo { font_size: 22 };

    while !rl.window_should_close() {
        let mouse = rl.get_mouse_position();
        let mouse_state = if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
            ButtonState::Down
        } else {
            ButtonState::Up
        };

        let mut ctx = UIContext::new(
            &font_info,
            Vec2 {
                x: mouse.x.max(0.0) as u32,
                y: mouse.y.max(0.0) as u32,
            },
            mouse_state,
        );

        let mut layout = Layout::new(
            LayoutDirection::Vertical,
            10,
            Vec2 { x: 28, y: 28 },
            Vec2 { x: 620, y: 360 },
        );
        ctx.draw_text_layout(&mut layout, "reimui + raylib".to_string());
        ctx.draw_text_layout(
            &mut layout,
            "Immediate mode UI rendering to raylib".to_string(),
        );
        let clicked = ctx.draw_button(&mut layout, Vec2 { x: 18, y: 12 }, "Click me".to_string());

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::RAYWHITE);

        for command in ctx.take_commands() {
            match command {
                DrawCommand::DrawText { content, x, y, .. } => {
                    d.draw_text(
                        &content,
                        x as i32,
                        y as i32,
                        font_info.font_size,
                        Color::BLACK,
                    );
                }
            }
        }

        if clicked {
            d.draw_text(
                "Button pressed",
                28,
                (layout.top_left.y + 4) as i32,
                font_info.font_size,
                Color::DARKGREEN,
            );
        }
    }
}
