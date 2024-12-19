use log::debug;

use ExecutionError::InvalidOpcode;

use crate::error::ExecutionError;
use crate::platform::Quirks;
use crate::{get_platform, Memory, CHAR_SIZE, PROGRAM_START_ADDRESS};

#[derive(Debug)]
pub struct Cpu {
    pub i: u16,
    pub pc: u16,
    pub sp: u8,
    pub dt: u8,
    pub st: u8,
    pub stack: [u16; 16],
    pub registers: [u8; 16],
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            i: 0,
            pc: PROGRAM_START_ADDRESS,
            sp: 0,
            dt: 0,
            st: 0,
            stack: [0; 16],
            registers: [0; 16],
        }
    }

    pub fn execute(&mut self, opcode: u16, mem: &mut Memory) -> Result<(), ExecutionError> {
        let c = (opcode & 0xF000) >> 12;
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let d = opcode & 0x000F;

        let nn = (opcode & 0x00FF) as u8;
        let nnn = opcode & 0x0FFF;

        match (c, x, y, d) {
            (0x0, 0x0, 0xE, 0x0) => self.op_00e0(mem),
            (0x0, 0x0, 0xE, 0xE) => self.op_00ee(),
            (0x1, _, _, _) => self.op_1nnn(nnn),
            (0x2, _, _, _) => self.op_2nnn(nnn),
            (0x3, _, _, _) => self.op_3xkk(x, nn),
            (0x4, _, _, _) => self.op_4xkk(x, nn),
            (0x5, _, _, 0x0) => self.op_5xy0(x, y),
            (0x6, _, _, _) => self.op_6xkk(x, nn),
            (0x7, _, _, _) => self.op_7xkk(x, nn),
            (0x8, _, _, 0x0) => self.op_8xy0(x, y),
            (0x8, _, _, 0x1) => self.op_8xy1(x, y),
            (0x8, _, _, 0x2) => self.op_8xy2(x, y),
            (0x8, _, _, 0x3) => self.op_8xy3(x, y),
            (0x8, _, _, 0x4) => self.op_8xy4(x, y),
            (0x8, _, _, 0x5) => self.op_8xy5(x, y),
            (0x8, _, _, 0x6) => self.op_8xy6(x, y),
            (0x8, _, _, 0x7) => self.op_8xy7(x, y),
            (0x8, _, _, 0xE) => self.op_8xye(x, y),
            (0x9, _, _, 0x0) => self.op_9xy0(x, y),
            (0xA, _, _, _) => self.op_annn(nnn),
            (0xB, _, _, _) => self.op_bnnn(nnn),
            (0xC, _, _, _) => self.op_cxkk(mem, x, nn),
            (0xD, _, _, _) => self.op_dxyn(mem, x, y, d),
            (0xE, _, 0x9, 0xE) => self.op_ex9e(mem, x),
            (0xE, _, 0xA, 0x1) => self.op_exa1(mem, x),
            (0xF, _, 0x0, 0x7) => self.op_fx07(x),
            (0xF, _, 0x0, 0xA) => self.op_fx0a(mem, x),
            (0xF, _, 0x1, 0x5) => self.op_fx15(x),
            (0xF, _, 0x1, 0x8) => self.op_fx18(x),
            (0xF, _, 0x1, 0xE) => self.op_fx1e(x),
            (0xF, _, 0x2, 0x9) => self.op_fx29(x),
            (0xF, _, 0x3, 0x3) => self.op_fx33(mem, x),
            (0xF, _, 0x5, 0x5) => self.op_fx55(mem, x),
            (0xF, _, 0x6, 0x5) => self.op_fx65(mem, x),
            _ => {
                return Err(InvalidOpcode(opcode));
            }
        }

        Ok(())
    }

    pub fn sne(&mut self) {
        self.pc += 2;
    }

    fn op_00e0(&mut self, mem: &mut Memory) {
        debug!("00E0: CLS - Clear screen");

        mem.video.fill(0);
        self.sne();
    }

    fn op_00ee(&mut self) {
        debug!("00EE: RET - Return");

        self.sp -= 1;
        self.pc = self.stack[self.sp as usize];
        self.sne();
    }

    fn op_1nnn(&mut self, nnn: u16) {
        debug!("1NNN: JP addr - Jump to address");
        self.pc = nnn;
    }

    fn op_2nnn(&mut self, nnn: u16) {
        debug!("2NNN: CALL addr - Call subroutine at address");

        self.stack[self.sp as usize] = self.pc;
        self.sp += 1;
        self.pc = nnn;
    }

    fn op_3xkk(&mut self, x: usize, nn: u8) {
        debug!("3XKK: SE Vx, byte - Skip next instruction if Vx = byte");

        if self.registers[x] == nn {
            self.sne();
        }

        self.sne();
    }

    fn op_4xkk(&mut self, x: usize, nn: u8) {
        debug!("4XKK: SNE Vx, byte - Skip next instruction if Vx != byte");
        if self.registers[x] != nn {
            self.sne();
        }

        self.sne();
    }

    fn op_5xy0(&mut self, x: usize, y: usize) {
        debug!("5XY0: SE Vx, Vy - Skip next instruction if Vx = Vy");
        if self.registers[x] == self.registers[y] {
            self.sne();
        }

        self.sne();
    }

    fn op_6xkk(&mut self, x: usize, nn: u8) {
        debug!("6XKK: LD Vx, byte - Set Vx = byte");

        self.registers[x] = nn;

        self.sne();
    }

    fn op_7xkk(&mut self, x: usize, nn: u8) {
        debug!("7XKK: ADD Vx, byte - Add byte to Vx");

        let result = u16::from(self.registers[x]) + u16::from(nn);
        self.registers[x] = result as u8;

        self.sne();
    }

    fn op_8xy0(&mut self, x: usize, y: usize) {
        debug!("8XY0: LD Vx, Vy - Set Vx = Vy");

        self.registers[x] = self.registers[y];

        self.sne();
    }

    fn op_8xy1(&mut self, x: usize, y: usize) {
        debug!("8XY1: OR Vx, Vy - Set Vx = Vx OR Vy");

        self.registers[x] |= self.registers[y];
        self.registers[0xf] = 0;

        self.sne();
    }

    fn op_8xy2(&mut self, x: usize, y: usize) {
        debug!("8XY2 AND Vx, Vy - Set Vx = Vx AND Vy");

        self.registers[x] &= self.registers[y];
        self.registers[0xf] = 0;

        self.sne();
    }

    fn op_8xy3(&mut self, x: usize, y: usize) {
        debug!("8XY3: XOR Vx, Vy - Set Vx = Vx XOR Vy");

        self.registers[x] ^= self.registers[y];
        self.registers[0xf] = 0;

        self.sne();
    }

    fn op_8xy4(&mut self, x: usize, y: usize) {
        debug!("8XY4: ADD Vx, Vy - Set Vx = Vx + Vy, Set VF = carry");

        let (result, did_overflow) = self.registers[x].overflowing_add(self.registers[y]);

        self.registers[x] = result;
        self.registers[0xF] = did_overflow.into();

        self.sne();
    }

    fn op_8xy5(&mut self, x: usize, y: usize) {
        debug!("8XY5: SUB Vx, Vy - Set Vx = Vx - Vy, Set VF = not borrow");

        let (result, did_overflow) = self.registers[x].overflowing_sub(self.registers[y]);

        self.registers[x] = result;
        self.registers[0xF] = (!did_overflow).into();

        self.sne();
    }

    fn op_8xy6(&mut self, x: usize, y: usize) {
        debug!("8XY6: SHR Vx - Set Vx = Vx SHR 1");

        if !get_platform().has_quirk(Quirks::SHIFT) {
            self.registers[x] = self.registers[y];
        }

        let updated_vf = self.registers[x] & 0x1;
        self.registers[x] >>= 1;
        self.registers[0xF] = updated_vf;

        self.sne();
    }

    fn op_8xy7(&mut self, x: usize, y: usize) {
        debug!("8XY7: SUB Vx, Vy - Set Vx = Vy - Vx, Set VF = not borrow");

        let (result, did_overflow) = self.registers[y].overflowing_sub(self.registers[x]);

        self.registers[x] = result;
        self.registers[0xF] = (!did_overflow).into();

        self.sne();
    }

    fn op_8xye(&mut self, x: usize, y: usize) {
        debug!("8XYE - SHL Vx - Set Vx = Vx SHL 1");

        if !get_platform().has_quirk(Quirks::SHIFT) {
            self.registers[x] = self.registers[y];
        }

        let updated_vf = (self.registers[x] & 0x80) >> 7;
        self.registers[x] <<= 1;
        self.registers[0xF] = updated_vf;

        self.sne();
    }

    fn op_9xy0(&mut self, x: usize, y: usize) {
        debug!("9XY0: SNE Vx, Vy - Skip next instruction if Vx != Vy");

        if self.registers[x] != self.registers[y] {
            self.sne();
        }

        self.sne();
    }

    fn op_annn(&mut self, nnn: u16) {
        debug!("ANNN: LD I, addr - Set I = nnn");

        self.i = nnn;

        self.sne();
    }

    fn op_bnnn(&mut self, nnn: u16) {
        debug!("BNNN: JP V0, addr - Jump to address V0 + addr");

        self.pc = u16::from(self.registers[0]) + nnn;

        self.sne();
    }

    fn op_cxkk(&mut self, mem: &mut Memory, x: usize, nn: u8) {
        debug!("CXKK: RND Vx, byte - Set Vx = random byte AND byte");

        self.registers[x] = mem.rand_byte() & nn;

        self.sne();
    }

    fn op_dxyn(&mut self, mem: &mut Memory, x: usize, y: usize, n: u16) {
        debug!("DXYN: DRW Vx, Vy, nibble - Display n-byte sprite starting at I to coordinates (Vx, Vy), Set VF = collision");

        let mut collision = 0;
        let (video_width, video_height) = (get_platform().video_width, get_platform().video_height);
        for display_y in 0..n {
            let pixel = mem.ram[(self.i + display_y) as usize];

            for display_x in 0..8 {
                if pixel & (0x80 >> display_x) != 0 {
                    let mut x_pos = self.registers[x] as u16 + display_x;
                    let mut y_pos = self.registers[y] as u16 + display_y;
                    x_pos %= video_width;
                    y_pos %= video_height;

                    let pixel_pos: usize = (y_pos * video_width + x_pos) as usize;

                    collision |= mem.video[pixel_pos] & 1;
                    mem.video[pixel_pos] ^= 1;
                }
            }
        }

        self.registers[0xF] = collision;

        self.sne();
    }

    fn op_ex9e(&mut self, mem: &mut Memory, x: usize) {
        debug!("EX9E: SKP Vx - Skip next instruction if key with the value of Vx is pressed");

        if mem.keypad[self.registers[x] as usize] {
            self.sne();
        }

        self.sne();
    }

    fn op_exa1(&mut self, mem: &mut Memory, x: usize) {
        debug!("EXA1: SKNP Vx - Skip next instruction if key with the value of Vx is not pressed");

        if !mem.keypad[self.registers[x] as usize] {
            self.sne();
        }

        self.sne();
    }

    fn op_fx07(&mut self, x: usize) {
        debug!("FX07: LD Vx, DT - Set Vx = delay timer");

        self.registers[x] = self.dt;

        self.sne();
    }

    fn op_fx0a(&mut self, mem: &mut Memory, x: usize) {
        debug!("FX0A: LD Vx, K - Wait for key press and store the value into Vx");

        if mem.keypad[0] {
            self.registers[x] = 0;
            self.sne();
        } else if mem.keypad[1] {
            self.registers[x] = 1;
            self.sne();
        } else if mem.keypad[2] {
            self.registers[x] = 2;
            self.sne();
        } else if mem.keypad[3] {
            self.registers[x] = 3;
            self.sne();
        } else if mem.keypad[4] {
            self.registers[x] = 4;
            self.sne();
        } else if mem.keypad[5] {
            self.registers[x] = 5;
            self.sne();
        } else if mem.keypad[6] {
            self.registers[x] = 6;
            self.sne();
        } else if mem.keypad[7] {
            self.registers[x] = 7;
            self.sne();
        } else if mem.keypad[8] {
            self.registers[x] = 8;
            self.sne();
        } else if mem.keypad[9] {
            self.registers[x] = 9;
            self.sne();
        } else if mem.keypad[10] {
            self.registers[x] = 10;
            self.sne();
        } else if mem.keypad[11] {
            self.registers[x] = 11;
            self.sne();
        } else if mem.keypad[12] {
            self.registers[x] = 12;
            self.sne();
        } else if mem.keypad[13] {
            self.registers[x] = 13;
            self.sne();
        } else if mem.keypad[14] {
            self.registers[x] = 14;
            self.sne();
        } else if mem.keypad[15] {
            self.registers[x] = 15;
            self.sne();
        }
    }

    fn op_fx15(&mut self, x: usize) {
        debug!("FX15: LD DT, Vx - Set delay timer = Vx");

        self.dt = self.registers[x];

        self.sne();
    }

    fn op_fx18(&mut self, x: usize) {
        debug!("FX18: LD ST, Vx - Set sound timer = Vx");

        self.st = self.registers[x];

        self.sne();
    }

    fn op_fx1e(&mut self, x: usize) {
        debug!("FX1E: Add I, Vx - Set I = I + Vx");

        self.i += u16::from(self.registers[x]);

        self.sne();
    }

    fn op_fx29(&mut self, x: usize) {
        debug!("FX29: LD F, Vx - Set I = location of sprite for digit Vx");

        self.i = u16::from(CHAR_SIZE) * u16::from(self.registers[x]);

        self.sne();
    }

    fn op_fx33(&mut self, mem: &mut Memory, x: usize) {
        debug!("FX33: LD B, Vx - Store BCD (Binary-Coded Decimal) representation of Vx in memory locations I, I + 1, and I + 2");

        let mut result = self.registers[x];
        for offset in (0..3).rev() {
            mem.ram[(self.i + offset) as usize] = result % 10;
            result /= 10;
        }

        self.sne();
    }

    fn op_fx55(&mut self, mem: &mut Memory, x: usize) {
        debug!("FX55: LD [I], Vx - Store V0~Vx in memory starting at location I");

        for offset in 0..=x {
            mem.ram[self.i as usize + offset] = self.registers[offset];
        }

        self.sne();
    }

    fn op_fx65(&mut self, mem: &mut Memory, x: usize) {
        debug!("FX65: LD Vx, [I] - Read registers V0~Vx from memory starting at location I");

        for offset in 0..=x {
            self.registers[offset] = mem.ram[self.i as usize + offset];
        }

        self.sne();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::Memory;

    #[test]
    fn test_00e0() {
        let mut mem = Memory::new();
        mem.video.fill(1);
        let mut cpu = Cpu::new();

        cpu.op_00e0(&mut mem);

        assert!(mem.video.iter().all(|&x| x == 0));
    }

    #[test]
    fn test_00ee() {
        let mut cpu = Cpu::new();
        cpu.sp = 1;
        let return_addr = 0x500;
        cpu.stack[0] = return_addr;

        cpu.op_00ee();

        assert_eq!(0, cpu.sp);
        assert_eq!(return_addr + 2, cpu.pc);
    }

    #[test]
    fn test_1nnn() {
        let mut cpu = Cpu::new();

        cpu.op_1nnn(0x1222);

        assert_eq!(0x1222, cpu.pc);
    }

    #[test]
    fn test_2nnn() {
        let mut cpu = Cpu::new();
        cpu.sp = 0xE;

        cpu.op_2nnn(0x2111);

        assert_eq!(0xF, cpu.sp);
        assert_eq!(PROGRAM_START_ADDRESS, cpu.stack[cpu.sp as usize - 1]);
        assert_eq!(0x2111, cpu.pc);
    }

    #[test]
    fn test_3xkk() {
        // Test skip case
        let mut cpu = Cpu::new();
        let x = 0xE;
        let nn = 0xFF;
        let mut starting_pc = cpu.pc;
        cpu.registers[x] = nn;

        cpu.op_3xkk(x, nn);
        assert_eq!(starting_pc + 4, cpu.pc); // Should skip (PC+4)

        // Test no skip case
        cpu = Cpu::new();
        starting_pc = cpu.pc;
        cpu.registers[x] = nn - 1;

        cpu.op_3xkk(x, nn);
        assert_eq!(starting_pc + 2, cpu.pc); // Should not skip (PC+2)
    }

    #[test]
    fn test_4xkk() {
        // Test skip case
        let mut cpu = Cpu::new();
        let x = 0xE;
        let nn = 0xFF;
        let mut starting_pc = cpu.pc;
        cpu.registers[x] = nn - 1;

        cpu.op_4xkk(x, nn);
        assert_eq!(starting_pc + 4, cpu.pc); // Should skip (PC+4)

        // Test no skip case
        cpu = Cpu::new();
        starting_pc = cpu.pc;
        cpu.registers[x] = nn;

        cpu.op_4xkk(x, nn);
        assert_eq!(starting_pc + 2, cpu.pc); // Should not skip (PC+2)
    }

    #[test]
    fn test_5xy0() {
        // Test skip case
        let mut cpu = Cpu::new();
        let x = 0xD;
        let y = 0xE;
        let nn = 0xFF;
        let mut starting_pc = cpu.pc;
        cpu.registers[x] = nn;
        cpu.registers[y] = nn;

        cpu.op_5xy0(x, y);
        assert_eq!(starting_pc + 4, cpu.pc); // Should skip (PC+4)

        // Test no skip case
        cpu = Cpu::new();
        starting_pc = cpu.pc;
        cpu.registers[x] = nn;
        cpu.registers[y] = nn - 1;

        cpu.op_5xy0(x, y);
        assert_eq!(starting_pc + 2, cpu.pc); // Should not skip (PC+2)
    }

    #[test]
    fn test_6xkk() {
        let mut cpu = Cpu::new();
        let x = 0xD;
        let nn = 0xFF;
        cpu.registers[x] = nn;

        cpu.op_6xkk(x, nn);

        assert_eq!(nn, cpu.registers[x]);
    }

    #[test]
    fn test_7xkk() {
        let mut cpu = Cpu::new();
        let x = 0xD;
        let nn = 0xA;
        cpu.registers[x] = 0xE2;
        let expected = cpu.registers[x].wrapping_add(nn);

        cpu.op_7xkk(x, nn);

        assert_eq!(expected, cpu.registers[x]);
    }

    #[test]
    fn test_8xy0() {
        let mut cpu = Cpu::new();
        let x = 0xD;
        let y = 0xE;
        cpu.registers[y] = 0x1;

        cpu.op_8xy0(x, y);

        assert_eq!(cpu.registers[x], cpu.registers[y]);
    }

    #[test]
    fn test_8xy1() {
        let mut cpu = Cpu::new();
        let x = 0xD;
        let y = 0xE;
        cpu.registers[x] = 0x8;
        cpu.registers[y] = 0xA;
        let expected = cpu.registers[x] | cpu.registers[y];

        cpu.op_8xy1(x, y);

        assert_eq!(expected, cpu.registers[x]);
    }

    #[test]
    fn test_8xy2() {
        let mut cpu = Cpu::new();
        let x = 0xD;
        let y = 0xE;
        cpu.registers[x] = 0x8;
        cpu.registers[y] = 0xA;
        let expected = cpu.registers[x] & cpu.registers[y];

        cpu.op_8xy2(x, y);

        assert_eq!(expected, cpu.registers[x]);
    }

    #[test]
    fn test_8xy3() {
        let mut cpu = Cpu::new();
        let x = 0xD;
        let y = 0xE;
        cpu.registers[x] = 0x8;
        cpu.registers[y] = 0xA;
        let expected = cpu.registers[x] ^ cpu.registers[y];

        cpu.op_8xy3(x, y);

        assert_eq!(expected, cpu.registers[x]);
    }

    #[test]
    fn test_8xy4() {
        let mut cpu = Cpu::new();
        let x = 0xD;
        let y = 0xE;
        cpu.registers[x] = 0xF;
        cpu.registers[y] = 0xF;
        let (expected_result, expected_vf) = cpu.registers[x].overflowing_add(cpu.registers[y]);

        cpu.op_8xy4(x, y);

        assert_eq!(expected_result, cpu.registers[x]);
        assert_eq!(expected_vf as u8, cpu.registers[0xF]);
    }

    #[test]
    fn test_8xy5() {
        let mut cpu = Cpu::new();
        let x = 0xD;
        let y = 0xE;
        cpu.registers[x] = 0xF;
        cpu.registers[y] = 0xA;
        let (expected_result, did_borrow) = cpu.registers[x].overflowing_sub(cpu.registers[y]);
        let expected_vf = !did_borrow as u8;

        cpu.op_8xy5(x, y);

        assert_eq!(expected_result, cpu.registers[x]);
        assert_eq!(expected_vf, cpu.registers[0xF]);
    }

    #[test]
    fn test_8xy6() {
        let mut cpu = Cpu::new();
        let x = 0xD;
        cpu.registers[x] = 0xF;
        let expected_result = cpu.registers[x] >> 1;
        let expected_vf = cpu.registers[x] & 1;

        cpu.op_8xy6(x, x);

        assert_eq!(expected_result, cpu.registers[x]);
        assert_eq!(expected_vf, cpu.registers[0xF]);
    }

    #[test]
    fn test_8xy7() {
        let mut cpu = Cpu::new();
        let x = 0xD;
        let y = 0xE;
        cpu.registers[x] = 0xA;
        cpu.registers[y] = 0xF;
        let (expected_result, did_borrow) = cpu.registers[y].overflowing_sub(cpu.registers[x]);
        let expected_vf = !did_borrow as u8;

        cpu.op_8xy7(x, y);

        assert_eq!(expected_result, cpu.registers[x]);
        assert_eq!(expected_vf, cpu.registers[0xF]);
    }

    #[test]
    fn test_8xye() {
        let mut cpu = Cpu::new();
        let x = 0xD;
        cpu.registers[x] = 0xF;
        let expected_result = cpu.registers[x] << 1;
        let expected_vf = (cpu.registers[x] & 0x80) >> 7;

        cpu.op_8xye(x, x);

        assert_eq!(expected_result, cpu.registers[x]);
        assert_eq!(expected_vf, cpu.registers[0xF]);
    }

    #[test]
    fn test_9xy0() {
        // Skip case
        let mut cpu = Cpu::new();
        let x = 0xD;
        let y = 0xE;
        let mut starting_pc = cpu.pc;
        cpu.registers[x] = 0x5;
        cpu.registers[y] = 0x6;

        cpu.op_9xy0(x, y);
        assert_eq!(starting_pc + 4, cpu.pc); // Should skip (PC+4)

        // No skip case
        cpu = Cpu::new();
        starting_pc = cpu.pc;
        cpu.registers[x] = 0x5;
        cpu.registers[y] = 0x5;

        cpu.op_9xy0(x, y);
        assert_eq!(starting_pc + 2, cpu.pc); // Should not skip (PC+2)
    }

    #[test]
    fn test_annn() {
        let mut cpu = Cpu::new();
        let nnn = 0x100;

        cpu.op_annn(nnn);

        assert_eq!(nnn, cpu.i);
    }

    #[test]
    fn test_bnnn() {
        let mut cpu = Cpu::new();
        let nnn = 0x200;
        cpu.registers[0] = 0x5;
        let expected = nnn + cpu.registers[0] as u16;

        cpu.op_bnnn(nnn);

        assert_eq!(expected + 2, cpu.pc); // Jump target + 2
    }

    #[test]
    fn test_cxkk() {
        let mut cpu = Cpu::new();
        let mut mem = Memory::new();
        let x = 0xD;
        let nn = 0xE;
        cpu.registers[x] = nn;

        cpu.op_cxkk(&mut mem, x, nn);

        assert_ne!(nn, cpu.registers[x]);
    }

    #[test]
    fn test_op_dxyn() {
        let mut cpu = Cpu::new();
        let mut mem = Memory::new();
        cpu.i = 0x200;
        mem.ram[cpu.i as usize] = 0x1;
        mem.video[0x7] = 0x1;

        cpu.op_dxyn(&mut mem, 0, 0, 1);

        assert_eq!(0x0, mem.video[0x7]);
        assert_eq!(0x1, cpu.registers[0xF]);
    }

    #[test]
    fn test_ex9e() {
        // Skip case
        let mut cpu = Cpu::new();
        let mut mem = Memory::new();
        let x = 0xD;
        let mut starting_pc = cpu.pc;

        cpu.registers[x] = 0x5;
        mem.keypad[0x5] = true;
        cpu.op_ex9e(&mut mem, x);

        assert_eq!(starting_pc + 4, cpu.pc); // Should skip (PC+4)

        // No skip case
        cpu = Cpu::new();
        mem = Memory::new();
        starting_pc = cpu.pc;

        cpu.registers[x] = 0x5;
        cpu.op_ex9e(&mut mem, x);

        assert_eq!(starting_pc + 2, cpu.pc); // Should not skip (PC+2)
    }

    #[test]
    fn test_exa1() {
        // Skip case
        let mut cpu = Cpu::new();
        let mut mem = Memory::new();
        let x = 0xD;
        let mut starting_pc = cpu.pc;
        cpu.registers[x] = 0x5;

        cpu.op_exa1(&mut mem, x);
        assert_eq!(starting_pc + 4, cpu.pc); // Should skip (PC+4)

        // No skip case
        cpu = Cpu::new();
        mem = Memory::new();
        starting_pc = cpu.pc;
        cpu.registers[x] = 0x5;
        mem.keypad[0x5] = true;

        cpu.op_exa1(&mut mem, x);
        assert_eq!(starting_pc + 2, cpu.pc); // Should not skip (PC+2)
    }

    #[test]
    fn test_fx07() {
        let mut cpu = Cpu::new();
        let x = 0xD;
        cpu.registers[x] = 0x1;
        cpu.dt = 0xF;

        cpu.op_fx07(x);

        assert_eq!(cpu.registers[x], cpu.dt);
    }

    #[test]
    fn test_fx0a() {
        let mut cpu = Cpu::new();
        let mut mem = Memory::new();
        let x = 0xD;
        let starting_pc = cpu.pc;
        mem.keypad[5] = true;

        cpu.op_fx0a(&mut mem, x);

        assert_eq!(5, cpu.registers[x]); // Register set to pressed key
        assert_eq!(starting_pc + 2, cpu.pc); // PC incremented
    }

    #[test]
    fn test_fx15() {
        let mut cpu = Cpu::new();
        let x = 0xD;
        cpu.registers[x] = 0x11;
        cpu.dt = 0x12;

        cpu.op_fx15(x);

        assert_eq!(cpu.dt, cpu.registers[x]);
    }

    #[test]
    fn test_fx18() {
        let mut cpu = Cpu::new();
        let x = 0xD;
        cpu.registers[x] = 0x11;
        cpu.st = 0x12;

        cpu.op_fx18(x);

        assert_eq!(cpu.st, cpu.registers[x]);
    }

    #[test]
    fn test_fx1e() {
        let mut cpu = Cpu::new();
        let x = 0xD;
        cpu.registers[x] = 0x11;
        cpu.i = 0x12;
        let expected = cpu.registers[x] as u16 + cpu.i;

        cpu.op_fx1e(x);

        assert_eq!(expected, cpu.i);
    }

    #[test]
    fn test_fx29() {
        let mut cpu = Cpu::new();
        let x = 0xD;
        cpu.registers[x] = 0x11;
        cpu.i = 0x12;
        let expected = cpu.registers[x] as u16 * CHAR_SIZE as u16;

        cpu.op_fx29(x);

        assert_eq!(expected, cpu.i);
    }

    #[test]
    fn test_fx33() {
        let mut cpu = Cpu::new();
        let mut mem = Memory::new();
        let x = 0xD;
        cpu.registers[x] = 0xFF;
        cpu.i = 0xFF;
        let mut result = cpu.registers[x];
        let expected1 = result % 10;
        result /= 10;
        let expected2 = result % 10;
        result /= 10;
        let expected3 = result % 10;

        cpu.op_fx33(&mut mem, x);

        assert_eq!(expected1, mem.ram[cpu.i as usize + 2]);
        assert_eq!(expected2, mem.ram[cpu.i as usize + 1]);
        assert_eq!(expected3, mem.ram[cpu.i as usize]);
    }

    #[test]
    fn test_fx55() {
        let mut cpu = Cpu::new();
        let mut mem = Memory::new();
        let x = 0x2;
        let expected = 0xEF;
        for offset in 0..=x {
            cpu.registers[offset] = expected;
        }

        cpu.op_fx55(&mut mem, x);

        assert_eq!(expected, mem.ram[cpu.i as usize]);
        assert_eq!(expected, mem.ram[cpu.i as usize + 1]);
        assert_eq!(expected, mem.ram[cpu.i as usize + 2]);
    }

    #[test]
    fn test_fx65() {
        let mut cpu = Cpu::new();
        let mut mem = Memory::new();
        let x = 0x2;
        let expected = 0xEF;
        for offset in 0..=x {
            mem.ram[cpu.i as usize + offset] = expected;
        }

        cpu.op_fx65(&mut mem, x);

        assert_eq!(expected, cpu.registers[0]);
        assert_eq!(expected, cpu.registers[1]);
        assert_eq!(expected, cpu.registers[2]);
    }
}
