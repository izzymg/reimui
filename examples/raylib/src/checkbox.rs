use raylib::prelude::*;
use reimui::prelude::*;

use crate::*;

const CHECKBOX_SIZE: Vec2 = Vec2::new(18, 18);
const SPACING: u32 = 18;

/// A UI demonstrating checkbox toggles.
pub struct CheckboxUI {
    ui_state: reimui::UIState,
    font_info: RaylibFontInfo,
    music_on: bool,
    sfx_on: bool,
    show_debug: bool,
}

impl CheckboxUI {
    pub fn new(rl: &RaylibHandle) -> Self {
        Self {
            ui_state: reimui::UIState::new(),
            font_info: RaylibFontInfo::new(rl),
            music_on: true,
            sfx_on: false,
            show_debug: false,
        }
    }

    pub fn draw(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread) {
        let input_state = raylib_input_state(rl, &self.ui_state);
        let music_label = if self.music_on { "Music on" } else { "Music off" };
        let sfx_label = if self.sfx_on { "SFX on" } else { "SFX off" };

        let mut ui = UIContext::new(self.ui_state, &self.font_info, input_state);

        ui.layout(LayoutDirection::Vertical, Some(SPACING), false, |ui| {
            ui.text_layout("Checkboxes");

            ui.checkbox_layout_label_left(
                Vec2::new(20, 20),
                &mut self.music_on,
                music_label,
                1.0,
                100,
            );

            ui.checkbox_layout_label_right(
                CHECKBOX_SIZE,
                &mut self.sfx_on,
                sfx_label,
                2.0,
                100,
            );
        });

        let ui_result = ui.end();

        let mut d = rl.begin_drawing(thread);
        d.clear_background(Color::RAYWHITE);
        apply_reimui_to_raylib(&ui_result, &mut d, &self.font_info);
        self.ui_state = ui_result.new_state;
    }
}

impl SampleUI for CheckboxUI {
    fn draw(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread) {
        self.draw(rl, thread);
    }
}
