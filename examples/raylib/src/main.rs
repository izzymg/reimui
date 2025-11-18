mod simple;

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(640, 400)
        .title("reimui + raylib")
        .build();

    let mut simple_ui = simple::SimpleUI::new();

    while !rl.window_should_close() {
        simple_ui.draw(&mut rl, &thread);
    }
}
