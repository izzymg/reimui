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

    /// Build reimui UI frame
    fn do_reimui(&mut self, input_state: reimui::UIInputState) -> reimui::UIResult {
        let mut ui = UIContext::new(self.ui_state, &self.font_info, input_state);

        // main layout - horizontal
        ui.layout(
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
                        ui.text_layout("Layouts - simple list".into());

                        for i in 0..5 {
                            ui.layout(
                                LayoutDirection::Horizontal,
                                Some(SPACING),
                                self.show_layouts,
                                |ui| {
                                    let text = format!("* Item {}", i);
                                    let btn_text = format!("Item {} button", i);
                                    ui.text_layout(text);
                                    ui.button_layout(BUTTON_PADDING, btn_text);
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
                        if ui.button_layout(
                            BUTTON_PADDING,
                            if self.show_layouts {
                                "Hide layouts".into()
                            } else {
                                "Show layouts".into()
                            },
                        ) {
                            self.show_layouts = !self.show_layouts;
                        }
                    },
                );
            },
        );

        // reassign the state and push the result back for raylib binding
        let ui_result = ui.end();
        self.ui_state = ui_result.new_state;

        ui_result
    }

    pub fn draw(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread) {
        let input_state = raylib_input_state(rl);
        let ui_result = self.do_reimui(input_state);

        let mut d = rl.begin_drawing(thread);
        d.clear_background(Color::RAYWHITE);
        apply_reimui_to_raylib(&ui_result, &mut d, &self.font_info);
    }
}

impl SampleUI for LayoutsUI {
    fn draw(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread) {
        self.draw(rl, thread);
    }
}
