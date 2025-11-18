
use raylib::{ prelude::*};

use reimui::{
    ButtonState, DrawCommand, FontInformation, Layout, LayoutDirection, UIContext, Vec2,
};
use reimui_raylib_example::{RaylibFontInfo, apply_reimui_to_raylib};


/// A simple reimui layout with a few pieces of text and a button with a click counter.
pub struct SimpleUI {
    clicked: u32,
    ui_state: reimui::UIState,
    font_info: RaylibFontInfo,
}

impl SimpleUI {

    pub fn new(rl: &RaylibHandle) -> Self {
        Self {
            clicked: 0,
            ui_state: reimui::UIState::new(),
            font_info: RaylibFontInfo::new(rl, 22),
        }
    }

    pub fn draw(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread) {
     
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
                &self.font_info,
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
            ui_ctx.draw_text_layout(&mut layout, "Immediate mode UI rendering to raylib".into());
            let clicked = ui_ctx.draw_button_layout(
                &mut layout,
                Vec2 { x: 18, y: 12 },
                format!("Click me {}", click_count).into(),
            );

            if clicked {
                click_count += 1;
            }

            let mut d = rl.begin_drawing(&thread);
            d.clear_background(Color::RAYWHITE);

            let ui_result = ui_ctx.end();
            apply_reimui_to_raylib(&ui_result, &mut d, &self.font_info);

            // make sure to reassign the update UI state for the next pass
            ui_state = ui_result.new_state;
        }
    }

}
