use raylib::prelude::*;
use reimui::prelude::*;

use crate::*;

// Slider sizes can be configured independently of their ranges
const BIG_SLIDER_SIZE: Vec2 = Vec2::new(100, 50);
const SMALL_SLIDER_SIZE: Vec2 = Vec2::new(20, 2);

/// A UI demonstrating sliders
pub struct SliderUI {
    ui_state: reimui::UIState,
    font_info: RaylibFontInfo,
    slider_a_state: reimui::SliderState<u32>,
}

impl SliderUI {
    pub fn new(rl: &RaylibHandle) -> Self {
        Self {
            ui_state: reimui::UIState::new(),
            font_info: RaylibFontInfo::new(rl),
            slider_a_state: reimui::SliderState::new_range(0..100, 50),
        }
    }

    /// Build reimui UI frame
    fn do_reimui(&mut self, mouse_position: Vec2, mouse_state: ButtonState) -> reimui::UIResult {
        let mut ui_ctx =
            UIContext::new(self.ui_state, &self.font_info, mouse_position, mouse_state);

        // build a vertical layout
        let mut layout = Layout::new(
            LayoutDirection::Vertical,
            25,
            Vec2::new(28, 28),
            Vec2::new(620, 360),
        );
        ui_ctx.draw_text_layout(&mut layout, "sliders".into());

        // draw our sliders
        ui_ctx.draw_slider(
            Rect {
                size: BIG_SLIDER_SIZE,
                top_left: Vec2::new(50, 50),
            },
            &mut self.slider_a_state,
        );

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

impl SampleUI for SliderUI {
    fn draw(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread) {
        self.draw(rl, thread);
    }
}
