## reimui

#####  (Rust Expressive Immediate Mode User Interface) 

A render agnostic, immediate-mode GUI lib written in Rust.

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
    fn compute_text_size(&self, text: &str) -> reimui::Vec2 {
        todo!("tell reimui how to measure your font");
    }
}

fn draw() {
    let mut ui_state = UIState::new();
    loop {
        // transient UI "frame"
        let mut ui = UIContext::new(UIState::new(), &Font, Vec2 { x: 0, y: 0 }, ButtonState::Up);
        let mut layout = Layout::new(LayoutDirection::Vertical, 12, Vec2 { x: 16, y: 16 }, Vec2::zero());

        ui.draw_text_layout(&mut layout, "Hello from reimui".into());
        let clicked = ui.draw_button_layout(&mut layout, Vec2 { x: 12, y: 8 }, "Press me".into());
        if clicked {
            println!("clicky");
        }

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
