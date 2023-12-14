use macroquad::window::next_frame;
use std::process::exit;

#[macroquad::main("ChipsteRS")]
async fn main() {
    env_logger::init();

    let mut chipsters = chipsters::ChipsteRS::new();

    while chipsters.should_run() {
        chipsters.handle_input();
        chipsters.update();
        chipsters.draw();

        next_frame().await
    }

    exit(0);
}
