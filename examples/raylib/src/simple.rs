use raylib::prelude::*;

use reimui::prelude::*;
use reimui_raylib_example::{apply_reimui_to_raylib, RaylibFontInfo};

const BUTTON_PADDING: Vec2 = Vec2 { x: 16, y: 12 };

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
            font_info: RaylibFontInfo::new(rl),
        }
    }

    /// Build reimui UI frame
    fn do_reimui(&mut self, mouse_position: Vec2, mouse_state: ButtonState) -> reimui::UIResult {
        let mut ui_ctx = UIContext::new(
            self.ui_state,
            &self.font_info,
            mouse_position,
            mouse_state,
        );

        // build a simple vertical layout
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
            BUTTON_PADDING,
            format!("Click me {}", self.clicked),
        );

        if clicked {
            self.clicked += 1;
        }

        // reassign the state and push the result back for raylib binding
        let ui_result = ui_ctx.end();
        self.ui_state = ui_result.new_state;

        ui_result
    }

    pub fn draw(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread) {
        let mouse = rl.get_mouse_position();
        let mouse_state = if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
            ButtonState::Down
        } else {
            ButtonState::Up
        };

        let ui_result = self.do_reimui(
            Vec2 {
                x: mouse.x.max(0.0) as u32,
                y: mouse.y.max(0.0) as u32,
            },
            mouse_state,
        );

        let mut d = rl.begin_drawing(thread);
        d.clear_background(Color::RAYWHITE);
        apply_reimui_to_raylib(&ui_result, &mut d, &self.font_info);
    }
}
