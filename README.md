## reimui

#####  (Rust Expressive Immediate Mode User Interface) 

A render agnostic, immediate-mode GUI lib written in Rust.

> ⚠️ This project is in active, frequently breaking development and probably doesn't work yet

## Features

* Buttons with persistent hover & active states
* Sliders for generic arbitrary types - step values, independent size from range
* Vertical & horizontal layout system - build via stack based callbacks
* Optional static string "class lists" for unique styling hooks 

## Usage sample

!! See the [raylib example](examples/raylib) 

```rs
use reimui::prelude::*;

/// Bind reimui draw primitives to your render backend
fn reimui_ui_to_renderer(result: &reimui::UIResult) {
    todo!("bind reimui to your render engine");
}

pub struct Font {
    pub font_size: i32,
}

impl reimui::FontInformation for Font {
    fn compute_text_size(&self, text: &str, scale: f32) -> reimui::Vec2 {
        todo!("tell reimui how to measure your font");
    }
}

fn draw() {
    let mut ui_state = UIState::new();
    loop {
        // transient UI "frame"
        let mut ui = UIContext::new(self.ui_state, &self.font_info, mouse_position, mouse_state);
        // build a vertical layout
        ui.layout(LayoutDirection::Vertical, Some(25), false, |ui| {
            ui.text_layout("hi from reimui!".into());
            ui.text_layout_scaled("big text".into(), 1.5);

            // make a new horizontal layout for the slider and value text
            ui.layout(LayoutDirection::Horizontal, Some(30), false, |ui| {
                // draw our sliders
                ui.slider_layout(BIG_SLIDER_SIZE, &mut self.slider_a_state);
                ui.text_layout_scaled(format!("{}", self.slider_a_state.value), 0.8);
            });
        });
        let result = ui.end();

        reimui_ui_to_renderer(&result); // your draw backend
        ui_state = result.new_state; // you hold the persistent information
    }
}
```

## Design

* Caller holds state
* No colors - attributes and flags the user can branch from
* Transparent API
