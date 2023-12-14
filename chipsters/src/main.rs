use std::{env, process::exit};

#[macroquad::main("ChipsteRS")]
async fn main() {
    env_logger::init();
    let args: Vec<String> = env::args().collect();

    let mut chipsters = chipsters::ChipsteRS::new(args);

    while chipsters.should_run() {
        chipsters.handle_input();
        chipsters.update();
        chipsters.draw().await
    }

    exit(0);
}
