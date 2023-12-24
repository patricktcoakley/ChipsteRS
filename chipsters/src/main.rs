use std::path::Path;
use std::{env, process::exit};

use chipsters::ChipsteRS;

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

    let mut chipsters = ChipsteRS::new();
    chipsters.load(rom_path);

    loop {
        chipsters.handle_input();
        chipsters.update().expect("failed to update");
        chipsters.draw().await
    }
}
