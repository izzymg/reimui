use raylib::prelude::*;
use reimui::prelude::*;

use crate::*;

const SPACING: u32 = 30;
const BUTTON_PADDING: Vec2 = Vec2::new(10, 8);

/// A UI demonstrating sliders
pub struct LayoutsUI {
    ui_state: reimui::UIState,
    font_info: RaylibFontInfo,
    show_layouts: bool,
}

impl LayoutsUI {
    pub fn new(rl: &RaylibHandle) -> Self {
        Self {
            ui_state: reimui::UIState::new(),
            font_info: RaylibFontInfo::new(rl),
            show_layouts: false,
        }
    }

    pub fn draw(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread) {
        let input_state = raylib_input_state(rl, &self.ui_state);
        let item_texts: Vec<String> = (0..5).map(|i| format!("* Item {}", i)).collect();
        let button_texts: Vec<String> = (0..5).map(|i| format!("Item {} button", i)).collect();

        let mut ui = UIContext::new(self.ui_state, &self.font_info, input_state);

        // main layout - horizontal
        let toggled = ui.layout(
            LayoutDirection::Horizontal,
            Some(SPACING),
            self.show_layouts,
            |ui| {
                // build a vertical layout with a list of buttons & text
                ui.layout(
                    LayoutDirection::Vertical,
                    Some(SPACING),
                    self.show_layouts,
                    |ui| {
                        ui.text_layout("Layouts - simple list");

                        for i in 0..5 {
                            ui.layout(
                                LayoutDirection::Horizontal,
                                Some(SPACING),
                                self.show_layouts,
                                |ui| {
                                    ui.text_layout(item_texts[i].as_str());
                                    ui.button_layout(BUTTON_PADDING, button_texts[i].as_str());
                                },
                            );
                        }
                    },
                );

                // another vertical layout with a toggle for the layouts
                ui.layout(
                    LayoutDirection::Vertical,
                    Some(SPACING),
                    self.show_layouts,
                    |ui| {
                        ui.button_layout(BUTTON_PADDING, "Toggle layouts")
                    },
                )
            },
        );

        if toggled {
            self.show_layouts = !self.show_layouts;
        }

        let ui_result = ui.end();

        let mut d = rl.begin_drawing(thread);
        d.clear_background(Color::RAYWHITE);
        apply_reimui_to_raylib(&ui_result, &mut d, &self.font_info);
        self.ui_state = ui_result.new_state;
    }
}

impl SampleUI for LayoutsUI {
    fn draw(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread) {
        self.draw(rl, thread);
    }
}
