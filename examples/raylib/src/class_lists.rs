use raylib::prelude::*;
use reimui::ClassList;
use reimui::prelude::*;

use crate::*;

const BUTTON_PADDING: Vec2 = Vec2::new(12, 10);

/// Using class lists to style unique elements when receiving reimui data back for your renderer.
pub struct ClassListUI {
    ui_state: reimui::UIState,
    font_info: RaylibFontInfo,
    danger_clicks: u32,
}

impl ClassListUI {
    pub fn new(rl: &RaylibHandle) -> Self {
        Self {
            ui_state: reimui::UIState::new(),
            font_info: RaylibFontInfo::new(rl),
            danger_clicks: 0,
        }
    }

    pub fn draw(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread) {
        let input_state = raylib_input_state(rl, &self.ui_state);
        let danger_clicks_label = format!("Danger clicks: {}", self.danger_clicks);

        let mut ui = UIContext::new(self.ui_state, &self.font_info, input_state);

        // The "panel" class colors the layout background in the renderer.
        ui.with_class_list(ClassList::new("panel"), |ui| {
            ui.layout(LayoutDirection::Vertical, Some(18), true, |ui| {
                ui.text_layout("Class list styling");

                ui.with_class_list(ClassList::new("muted"), |ui| {
                    ui.text_layout("Tagged with 'muted' class.");
                });

                ui.layout(LayoutDirection::Horizontal, Some(12), false, |ui| {
                    ui.with_class_list(ClassList::new("danger"), |ui| {
                        if ui.button_layout(BUTTON_PADDING, "Danger action") {
                            self.danger_clicks += 1;
                        }
                    });

                    ui.with_class_list(ClassList::new("accent"), |ui| {
                        ui.text_layout(&danger_clicks_label);
                    });
                });

                ui.with_class_list(ClassList::new("accent"), |ui| {
                    ui.button_layout(BUTTON_PADDING, "Accent action");
                });
            });
        });

        let ui_result = ui.end();

        let mut d = rl.begin_drawing(thread);
        d.clear_background(Color::RAYWHITE);
        apply_reimui_to_raylib(&ui_result, &mut d, &self.font_info);
        self.ui_state = ui_result.new_state;
    }
}

impl SampleUI for ClassListUI {
    fn draw(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread) {
        self.draw(rl, thread);
    }
}
