use crate::platform::Platform;
pub use chip8::Chip8;
use cpu::Cpu;
use memory::Memory;
pub use state::State;
use std::sync::OnceLock;

pub mod chip8;
mod cpu;
mod error;
mod memory;
pub mod state;
mod platform;

pub const CHAR_SIZE: u8 = 0x5;
pub const PROGRAM_START_ADDRESS: u16 = 0x200;

static PLATFORM: OnceLock<Platform> = OnceLock::new();

pub fn init_platform(platform: Platform) {
    PLATFORM.get_or_init(|| platform);
}

pub fn init_default_platform() {
    init_platform(Platform::default());
}

pub fn get_platform() -> &'static Platform {
    PLATFORM.get().expect("Platform not initialized")
}