use crate::{Cpu, VIDEO_WIDTH};
use crate::Memory;
use crate::PROGRAM_START_ADDRESS;
use crate::State;

pub struct Chip8 {
    memory: Memory,
    cpu: Cpu,
    pub state: State,
}

impl Chip8 {
    pub fn set_key(&mut self, i: usize, flag: bool) {
        self.memory.keypad[i] = flag;
    }
}

impl Chip8 {
    pub fn new() -> Self {
        Self {
            memory: Memory::new(),
            cpu: Cpu::new(),
            state: State::Stopped,
        }
    }

    pub fn load_rom(&mut self, rom_path: &str) {
        let rom = std::fs::read(rom_path).unwrap();
        self.memory.ram[PROGRAM_START_ADDRESS as usize..(PROGRAM_START_ADDRESS + rom.len() as u16) as usize].copy_from_slice(&rom);
        self.state = State::Running;
    }

    pub fn step(&mut self) {
        let opcode = self.opcode();
        self.cpu.execute(opcode, &mut self.memory);

        if self.cpu.dt > 0 {
            self.cpu.dt -= 1;
        }

        if self.cpu.st > 0 {
            self.cpu.st -= 1
        }
    }

    fn opcode(&self) -> u16 {
        (self.memory.ram[self.cpu.pc as usize] as u16) << 8 | (self.memory.ram[(self.cpu.pc + 1) as usize] as u16)
    }

    pub fn has_color(&self, x: u8, y: u8) -> bool {
        self.memory.video[(y as usize * VIDEO_WIDTH as usize) + x as usize] == 0x1
    }
}

impl Default for Chip8 {
    fn default() -> Self {
        Self::new()
    }
}
