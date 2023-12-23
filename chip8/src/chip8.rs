use crate::Memory;
use crate::State;
use crate::PROGRAM_START_ADDRESS;
use crate::{Cpu, VIDEO_WIDTH};

use log::info;
use std::path::Path;

#[derive(Debug)]
pub struct Chip8 {
    pub state: State,
    memory: Memory,
    cpu: Cpu,
    program_size: u16,
}

impl Chip8 {
    pub fn new() -> Self {
        Self {
            memory: Memory::new(),
            cpu: Cpu::new(),
            state: State::Off,
            program_size: 0,
        }
    }

    pub fn load_rom(&mut self, rom_path: &Path) -> Result<(), std::io::Error> {
        let rom = std::fs::read(rom_path)?;
        self.program_size = rom.len() as u16;

        if rom.len() > 0x1000 - PROGRAM_START_ADDRESS as usize {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("ROM is too big to fit in memory: {} bytes", rom.len()),
            ));
        }

        info!("Loading {}", rom_path.display());
        self.memory.ram
            [PROGRAM_START_ADDRESS as usize..(PROGRAM_START_ADDRESS + rom.len() as u16) as usize]
            .copy_from_slice(&rom);
        self.state = State::Running;
        Ok(())
    }

    pub fn reset(&mut self) {
        self.cpu = Cpu::new();
        self.cpu.execute(0x00E0, &mut self.memory);
        self.cpu.pc = PROGRAM_START_ADDRESS;
    }

    pub fn reset_keys(&mut self) {
        self.memory.keypad = [false; 16];
    }

    pub fn step(&mut self) {
        if self.state != State::Running {
            return;
        }

        if self.cpu.pc >= PROGRAM_START_ADDRESS + self.program_size {
            self.state = State::Finished;
            return;
        }

        let opcode = self.opcode();
        self.cpu.execute(opcode, &mut self.memory);

        if self.cpu.dt > 0 {
            self.cpu.dt -= 1;
        }

        if self.cpu.st > 0 {
            self.cpu.st -= 1
        }
    }
    pub fn key_down(&mut self, i: usize) {
        self.memory.keypad[i] = true;
    }

    pub fn has_color(&self, x: u8, y: u8) -> bool {
        self.memory.video[(y as usize * VIDEO_WIDTH as usize) + x as usize] == 0x1
    }

    fn opcode(&self) -> u16 {
        (self.memory.ram[self.cpu.pc as usize] as u16) << 8
            | (self.memory.ram[(self.cpu.pc + 1) as usize] as u16)
    }
}

impl Default for Chip8 {
    fn default() -> Self {
        Self::new()
    }
}
