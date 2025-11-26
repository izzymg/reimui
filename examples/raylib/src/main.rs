use std::{env, process::ExitCode};

use reimui_raylib_example::{checkbox, class_lists, layouts, simple, slider, SampleUI};

/// Simple example runner - actual code may be found inside the relevant file
fn main() -> ExitCode {
    let (mut rl, thread) = raylib::init()
        .size(640, 400)
        .title("reimui + raylib")
        .build();

    // figure out what example to run from the arg or run 'simple'
    let run_flag = env::args().nth(1);
    let run = match run_flag {
        Some(r) => r,
        None => {
            println!("no example arg provided, using 'simple'");
            "simple".into()
        }
    };

    // each example implements the sample ui trait
    let sample_ui: &mut dyn SampleUI = match run.as_str() {
        "simple" => &mut simple::SimpleUI::new(&rl),
        "slider" => &mut slider::SliderUI::new(&rl),
        "layouts" => &mut layouts::LayoutsUI::new(&rl),
        "class_lists" => &mut class_lists::ClassListUI::new(&rl),
        "checkbox" => &mut checkbox::CheckboxUI::new(&rl),
        other => {
            println!("unknown example type: '{}'", other);
            return ExitCode::FAILURE;
        }
    };

    while !rl.window_should_close() {
        sample_ui.draw(&mut rl, &thread);
    }
    ExitCode::SUCCESS
}
