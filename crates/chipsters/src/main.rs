use anyhow::{anyhow, Result};
use chipsters::ChipsteRS;
use std::env;
use std::path::Path;

#[macroquad::main("ChipsteRS")]
async fn main() -> Result<()> {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        return Err(anyhow!("Usage: chipsters <rom_path>"));
    }

    let rom_path = Path::new(&args[1]);
    let mut chipsters = ChipsteRS::default();
    chipsters.load(rom_path)
        .map_err(|err| anyhow!(err))?;

    loop {
        chipsters.handle_input()?;
        chipsters.update()?;
        chipsters.draw().await?;
    }
}