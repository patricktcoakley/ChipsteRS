use crate::platform::Platform;
pub use chip8::Chip8;
pub use state::State;
use virtual_machine::VirtualMachine;

pub mod chip8;
mod error;
mod platform;
pub mod state;
mod virtual_machine;

pub const CHAR_SIZE: u8 = 0x5;
pub const PROGRAM_START_ADDRESS: u16 = 0x200;
