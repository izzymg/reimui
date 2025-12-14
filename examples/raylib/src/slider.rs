use raylib::prelude::*;
use reimui::prelude::*;

use crate::*;

// Slider sizes can be configured independently of their ranges
const BIG_SLIDER_SIZE: Vec2 = Vec2::new(100, 50);
const SMALL_SLIDER_SIZE: Vec2 = Vec2::new(40, 10);

/// A UI demonstrating sliders
pub struct SliderUI {
    ui_state: reimui::UIState,
    font_info: RaylibFontInfo,
    slider_a_state: reimui::SliderState<u32>,
    slider_b_state: reimui::SliderState<f32>,
}

impl SliderUI {
    pub fn new(rl: &RaylibHandle) -> Self {
        Self {
            ui_state: reimui::UIState::new(),
            font_info: RaylibFontInfo::new(rl),
            slider_a_state: reimui::SliderState::new_range(0..100, 50, 5),
            slider_b_state: reimui::SliderState::new_range(0f32..10f32, 5.5, 0.5),
        }
    }

    /// Build reimui UI frame
    fn do_reimui(&mut self, input_state: reimui::UIInputState) -> reimui::UIResult {
        let mut ui = UIContext::new(self.ui_state, &self.font_info, input_state);

        // build a vertical layout
        ui.layout(LayoutDirection::Vertical, Some(25), false, |ui| {
            ui.text_layout("sliders".into());

            let a_val = format!("{}", self.slider_a_state.value);
            if ui.slider_layout_label_right(
                BIG_SLIDER_SIZE,
                &mut self.slider_a_state,
                a_val,
                1.0,
                50,
            ) {
                println!("slider moved!");
            }

            // Note that we pad the float value here to ensure the text size is consistent.
            // If we didn't, the slider would shift as the number of digits changed.
            // The format specifier here ensures 1 decimal place and a width of 4 characters.
            // We pad the value for visual consistency, but we also use a fixed label width.
            // Variable width fonts mean that even padded strings can change width, shifting the slider.
            // Since focus relies on a stable Rect, shifting causes focus loss.
            let b_val = format!("{:4.1}", self.slider_b_state.value);
            ui.slider_layout_label_left(
                SMALL_SLIDER_SIZE,
                &mut self.slider_b_state,
                b_val,
                1.0,
                50,
            );
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

impl SampleUI for SliderUI {
    fn draw(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread) {
        self.draw(rl, thread);
    }
}
