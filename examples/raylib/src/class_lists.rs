use raylib::prelude::*;
use reimui::prelude::*;
use reimui::ClassList;

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

    /// Build reimui UI frame
    fn do_reimui(&mut self, mouse_position: Vec2, mouse_state: ButtonState) -> reimui::UIResult {
        let mut ui = UIContext::new(self.ui_state, &self.font_info, mouse_position, mouse_state);

        // The "panel" class colors the layout background in the renderer.
        ui.with_class_list(ClassList::new("panel"), |ui| {
            ui.layout(LayoutDirection::Vertical, Some(18), true, |ui| {
                ui.text_layout("Class list styling".into());

                ui.with_class_list(ClassList::new("muted"), |ui| {
                    ui.text_layout("Tagged with 'muted' class.".into());
                });

                ui.layout(LayoutDirection::Horizontal, Some(12), false, |ui| {
                    ui.with_class_list(ClassList::new("danger"), |ui| {
                        if ui.button_layout(BUTTON_PADDING, "Danger action".into()) {
                            self.danger_clicks += 1;
                        }
                    });

                    ui.with_class_list(ClassList::new("accent"), |ui| {
                        ui.text_layout(format!("Danger clicks: {}", self.danger_clicks));
                    });
                });

                ui.with_class_list(ClassList::new("accent"), |ui| {
                    ui.button_layout(BUTTON_PADDING, "Accent action".into());
                });
            });
        });

        let ui_result = ui.end();
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

impl SampleUI for ClassListUI {
    fn draw(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread) {
        self.draw(rl, thread);
    }
}
