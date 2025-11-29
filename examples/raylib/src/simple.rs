use raylib::prelude::*;
use reimui::prelude::*;

use crate::*;

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
    fn do_reimui(&mut self, input_state: reimui::UIInputState) -> reimui::UIResult {
        let mut ui = UIContext::new(self.ui_state, &self.font_info, input_state);

        // put some text somewhere specific
        let pos = Vec2::new(364, 298);
        ui.text_at_scaled(format!("I'm at {}, {}", pos.x, pos.y), pos, 1.5);

        // build a simple vertical layout
        ui.layout(LayoutDirection::Vertical, Some(25), false, |ui| {
            ui.text_layout_scaled("reimui + raylib".into(), 5.0);
            ui.text_layout("Immediate mode UI rendering to raylib".into());
            let clicked = ui.button_layout(BUTTON_PADDING, format!("Click me {}", self.clicked));

            if clicked {
                self.clicked += 1;
            }
        });
        // reassign the state and push the result back for raylib binding
        let ui_result = ui.end();
        self.ui_state = ui_result.new_state;

        ui_result
    }

    pub fn draw(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread) {
        let input_state = raylib_input_state(rl, &self.ui_state);
        let ui_result = self.do_reimui(input_state);

        let mut d = rl.begin_drawing(thread);
        d.clear_background(Color::RAYWHITE);
        apply_reimui_to_raylib(&ui_result, &mut d, &self.font_info);
    }
}

impl SampleUI for SimpleUI {
    fn draw(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread) {
        self.draw(rl, thread);
    }
}
