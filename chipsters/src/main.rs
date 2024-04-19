use std::path::Path;
use std::{env, process::exit};

use chipsters::ChipsteRS;

#[macroquad::main("ChipsteRS")]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: chipsters <rom_path> | <roms_folder_path>");
        exit(1);
    }

    let rom_path = Path::new(&args[1]);
    if !rom_path.exists() {
        eprintln!("Invalid path: {}", &args[1]);
        exit(1);
    }

    let mut chipsters = ChipsteRS::default();
    if let Err(e) = chipsters.load(rom_path) {
        eprintln!("{e}");
        exit(1);
    }

    loop {
        chipsters.handle_input()?;
        chipsters.update().expect("failed to update ");
        chipsters.draw().await?
    }
}
