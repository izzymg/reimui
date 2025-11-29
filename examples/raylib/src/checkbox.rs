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
    show_debug: bool,
}

impl CheckboxUI {
    pub fn new(rl: &RaylibHandle) -> Self {
        Self {
            ui_state: reimui::UIState::new(),
            font_info: RaylibFontInfo::new(rl),
            music_on: true,
            show_debug: false,
        }
    }

    /// Build reimui UI frame
    fn do_reimui(&mut self, input_state: reimui::UIInputState) -> reimui::UIResult {
        let mut ui = UIContext::new(self.ui_state, &self.font_info, input_state);

        ui.layout(LayoutDirection::Vertical, Some(SPACING), false, |ui| {
            ui.text_layout("Checkboxes".into());

            ui.layout(LayoutDirection::Horizontal, Some(10), false, |ui| {
                ui.checkbox_layout(CHECKBOX_SIZE, &mut self.music_on);
                ui.text_layout(format!("Music {}", if self.music_on { "on" } else { "off" }));
            });

            ui.layout(LayoutDirection::Horizontal, Some(10), false, |ui| {
                ui.checkbox_layout(CHECKBOX_SIZE, &mut self.show_debug);
                ui.text_layout(format!(
                    "Debug overlay {}",
                    if self.show_debug { "shown" } else { "hidden" }
                ));
            });
        });

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

impl SampleUI for CheckboxUI {
    fn draw(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread) {
        self.draw(rl, thread);
    }
}
