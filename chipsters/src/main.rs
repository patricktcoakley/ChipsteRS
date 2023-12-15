use std::path::Path;
use std::{env, process::exit};

#[macroquad::main("ChipsteRS")]
async fn main() {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    let rom_path = match args.len() {
        2 => Path::new(&args[1]),
        _ => {
            println!("Usage: chipsters <rom_path>");
            exit(1);
        }
    };

    let mut chipsters = chipsters::ChipsteRS::new(rom_path);

    while chipsters.should_run() {
        chipsters.handle_input();
        chipsters.update();
        chipsters.draw().await
    }

    exit(0);
}
