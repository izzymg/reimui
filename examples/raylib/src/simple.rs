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

    pub fn draw(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread) {
        let input_state = raylib_input_state(rl, &self.ui_state);
        let pos = Vec2::new(364, 298);
        let pos_label = format!("I'm at {}, {}", pos.x, pos.y);
        let button_label = format!("Click me {}", self.clicked);

        let mut ui = UIContext::new(self.ui_state, &self.font_info, input_state);

        // put some text somewhere specific
        ui.text_at_scaled(&pos_label, pos, 1.5);

        // build a simple vertical layout
        let clicked = ui.layout(LayoutDirection::Vertical, Some(25), false, |ui| {
            ui.text_layout_scaled("reimui + raylib", 5.0);
            ui.text_layout("Immediate mode UI rendering to raylib");
            ui.button_layout(BUTTON_PADDING, &button_label)
        });

        if clicked {
            self.clicked += 1;
        }

        let ui_result = ui.end();

        let mut d = rl.begin_drawing(thread);
        d.clear_background(Color::RAYWHITE);
        apply_reimui_to_raylib(&ui_result, &mut d, &self.font_info);
        self.ui_state = ui_result.new_state;
    }
}

impl SampleUI for SimpleUI {
    fn draw(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread) {
        self.draw(rl, thread);
    }
}
