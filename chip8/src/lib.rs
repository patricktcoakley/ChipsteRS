pub use chip8::Chip8;
use cpu::Cpu;
use memory::Memory;
pub use state::State;

pub mod state;
pub mod chip8;
mod cpu;
mod memory;

pub const VIDEO_HEIGHT: u8 = 0x20;
pub const VIDEO_WIDTH: u8 = 0x40;
pub const CHAR_SIZE: u8 = 0x5;
pub const PROGRAM_START_ADDRESS: u16 = 0x200;
