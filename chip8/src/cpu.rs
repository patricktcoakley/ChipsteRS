use log::{debug, warn};

use crate::{Memory, CHAR_SIZE, PROGRAM_START_ADDRESS, VIDEO_HEIGHT, VIDEO_WIDTH};

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

    pub fn execute(&mut self, opcode: u16, mem: &mut Memory) {
        self.sne();
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
            (0x8, _, _, 0x6) => self.op_8xy6(x),
            (0x8, _, _, 0x7) => self.op_8xy7(x, y),
            (0x8, _, _, 0xE) => self.op_8xye(x),
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
                warn!("Unknown opcode {opcode}");
            }
        }
    }

    fn sne(&mut self) {
        self.pc += 2;
    }

    fn op_00e0(&self, mem: &mut Memory) {
        debug!("00E0: CLS - Clear screen");

        mem.video.fill(0);
    }

    fn op_00ee(&mut self) {
        debug!("00EE: RET - Return");

        self.sp -= 1;
        self.pc = self.stack[self.sp as usize];
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
    }

    fn op_4xkk(&mut self, x: usize, nn: u8) {
        debug!("4XKK: SNE Vx, byte - Skip next instruction if Vx != byte");
        if self.registers[x] != nn {
            self.sne();
        }
    }

    fn op_5xy0(&mut self, x: usize, y: usize) {
        debug!("5XY0: SE Vx, Vy - Skip next instruction if Vx = Vy");
        if self.registers[x] == self.registers[y] {
            self.sne();
        }
    }

    fn op_6xkk(&mut self, x: usize, nn: u8) {
        debug!("6XKK: LD Vx, byte - Set Vx = byte");

        self.registers[x] = nn
    }

    fn op_7xkk(&mut self, x: usize, nn: u8) {
        debug!("7XKK: ADD Vx, byte - Add byte to Vx");

        let result = self.registers[x] as u16 + nn as u16;
        self.registers[x] = result as u8;
    }

    fn op_8xy0(&mut self, x: usize, y: usize) {
        debug!("8XY0: LD Vx, Vy - Set Vx = Vy");

        self.registers[x] = self.registers[y];
    }

    fn op_8xy1(&mut self, x: usize, y: usize) {
        debug!("8XY1: OR Vx, Vy - Set Vx = Vx OR Vy");

        self.registers[x] |= self.registers[y];
    }

    fn op_8xy2(&mut self, x: usize, y: usize) {
        debug!("8XY2 AND Vx, Vy - Set Vx = Vx AND Vy");

        self.registers[x] &= self.registers[y];
    }

    fn op_8xy3(&mut self, x: usize, y: usize) {
        debug!("8XY3: XOR Vx, Vy - Set Vx = Vx XOR Vy");

        self.registers[x] ^= self.registers[y];
    }

    fn op_8xy4(&mut self, x: usize, y: usize) {
        debug!("8XY4: ADD Vx, Vy - Set Vx = Vx + Vy, Set VF = carry");

        let (result, updated_vf) = self.registers[x].overflowing_add(self.registers[y]);
        self.registers[x] = result;
        self.registers[0xF] = updated_vf.into()
    }

    fn op_8xy5(&mut self, x: usize, y: usize) {
        debug!("8XY5: SUB Vx, Vy - Set Vx = Vx - Vy, Set VF = not borrow");

        let (result, updated_vf) = self.registers[x].overflowing_sub(self.registers[y]);
        self.registers[x] = result;
        self.registers[0xF] = (!updated_vf).into();
    }

    fn op_8xy6(&mut self, x: usize) {
        debug!("8XY6: SHR Vx - Set Vx = Vx SHR 1");

        let updated_vf = self.registers[x] & 0x1;
        self.registers[x] >>= 1;
        self.registers[0xF] = updated_vf;
    }

    fn op_8xy7(&mut self, x: usize, y: usize) {
        debug!("8XY7: SUB Vx, Vy - Set Vx = Vy - Vx, Set VF = not borrow");

        let (result, updated_vf) = self.registers[y].overflowing_sub(self.registers[x]);
        self.registers[x] = result;
        self.registers[0xF] = (!updated_vf).into();
    }

    fn op_8xye(&mut self, x: usize) {
        debug!("8XYE - SHL Vx - Set Vx = Vx SHL 1");

        let updated_vf = (self.registers[x] & 0x80) >> 7;
        self.registers[x] <<= 1;
        self.registers[0xF] = updated_vf;
    }

    fn op_9xy0(&mut self, x: usize, y: usize) {
        debug!("9XY0: SNE Vx, Vy - Skip next instruction if Vx != Vy");

        if self.registers[x] != self.registers[y] {
            self.sne();
        }
    }

    fn op_annn(&mut self, nnn: u16) {
        debug!("ANNN: LD I, addr - Set I = nnn");

        self.i = nnn;
    }

    fn op_bnnn(&mut self, nnn: u16) {
        debug!("BNNN: JP V0, addr - Jump to address V0 + addr");

        self.pc = self.registers[nnn as usize] as u16;
    }

    fn op_cxkk(&mut self, mem: &mut Memory, x: usize, nn: u8) {
        debug!("CXKK: RND Vx, byte - Set Vx = random byte AND byte");

        self.registers[x] = mem.rand_byte() & nn;
    }

    fn op_dxyn(&mut self, mem: &mut Memory, x: usize, y: usize, d: u16) {
        debug!("DXYN: DRW Vx, Vy, nibble - Display n-byte sprite starting at I to coordinates (Vx, Vy), Set VF = collision");

        let mut collision = 0;
        for display_y in 0..d {
            let pixel = mem.ram[(self.i + display_y) as usize];

            for display_x in 0..8 {
                if pixel & (0x80 >> display_x) != 0 {
                    let x_pos = self.registers[x].overflowing_add(display_x).0 % VIDEO_WIDTH;
                    let y_pos = self.registers[y].overflowing_add(display_y as u8).0 % VIDEO_HEIGHT;
                    let pixel_pos = (y_pos as u16 * VIDEO_WIDTH as u16 + x_pos as u16) as usize;
                    collision = (mem.video[pixel_pos] == 0x1).into();
                    mem.video[pixel_pos] ^= 0x1
                }
            }
        }

        self.registers[0xF] = collision
    }

    fn op_ex9e(&mut self, mem: &mut Memory, x: usize) {
        debug!("EX9E: SKP Vx - Skip next instruction if key with the value of Vx is pressed");

        if mem.keypad[self.registers[x] as usize] {
            self.sne();
        }
    }

    fn op_exa1(&mut self, mem: &mut Memory, x: usize) {
        debug!("EXA1: SKNP Vx - Skip next instruction if key with the value of Vx is not pressed");

        if !mem.keypad[self.registers[x] as usize] {
            self.sne();
        }
    }

    fn op_fx07(&mut self, x: usize) {
        debug!("FX07: LD Vx, DT - Set Vx = delay timer");

        self.registers[x] = self.dt;
    }

    fn op_fx0a(&mut self, mem: &mut Memory, x: usize) {
        debug!("FX0A: LD Vx, K - Wait for key press and store the value into Vx");

        if mem.keypad[0] {
            self.registers[x] = 0;
        } else if mem.keypad[1] {
            self.registers[x] = 1;
        } else if mem.keypad[2] {
            self.registers[x] = 2;
        } else if mem.keypad[3] {
            self.registers[x] = 3;
        } else if mem.keypad[4] {
            self.registers[x] = 4;
        } else if mem.keypad[5] {
            self.registers[x] = 5;
        } else if mem.keypad[6] {
            self.registers[x] = 6;
        } else if mem.keypad[7] {
            self.registers[x] = 7;
        } else if mem.keypad[8] {
            self.registers[x] = 8;
        } else if mem.keypad[9] {
            self.registers[x] = 9;
        } else if mem.keypad[10] {
            self.registers[x] = 10;
        } else if mem.keypad[11] {
            self.registers[x] = 11;
        } else if mem.keypad[12] {
            self.registers[x] = 12;
        } else if mem.keypad[13] {
            self.registers[x] = 13;
        } else if mem.keypad[14] {
            self.registers[x] = 14;
        } else if mem.keypad[15] {
            self.registers[x] = 15;
        } else {
            self.pc -= 2;
        }
    }

    fn op_fx15(&mut self, x: usize) {
        debug!("FX15: LD DT, Vx - Set delay timer = Vx");

        self.dt = self.registers[x];
    }

    fn op_fx18(&mut self, x: usize) {
        debug!("FX18: LD ST, Vx - Set sound timer = Vx");

        self.st = self.registers[x];
    }

    fn op_fx1e(&mut self, x: usize) {
        debug!("FX1E: Add I, Vx - Set I = I + Vx");

        self.i += self.registers[x] as u16;
    }

    fn op_fx29(&mut self, x: usize) {
        debug!("FX29: LD F, Vx - Set I = location of sprite for digit Vx");

        self.i = CHAR_SIZE as u16 * self.registers[x] as u16;
    }

    fn op_fx33(&mut self, mem: &mut Memory, x: usize) {
        debug!("FX33: LD B, Vx - Store BCD (Binary-Coded Decimal) representation of Vx in memory locations I, I + 1, and I + 2");

        let mut result = self.registers[x];
        for offset in (0..3).rev() {
            mem.ram[(self.i + offset) as usize] = result % 10;
            result /= 10;
        }
    }

    fn op_fx55(&mut self, mem: &mut Memory, x: usize) {
        debug!("FX55: LD [I], Vx - Store V0~Vx in memory starting at location I");

        for offset in 0..=x {
            mem.ram[self.i as usize + offset] = self.registers[offset]
        }
    }

    fn op_fx65(&mut self, mem: &mut Memory, x: usize) {
        debug!("FX65: LD Vx, [I] - Read registers V0~Vx from memory starting at location I");

        for offset in 0..=x {
            self.registers[offset] = mem.ram[self.i as usize + offset];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Cpu;

    #[test]
    fn test_sne() {
        let mut cpu = Cpu::new();

        cpu.sne();

        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    fn test_op_00e0() {
        let cpu = Cpu::new();
        let mut mem = crate::memory::Memory::new();

        cpu.op_00e0(&mut mem);

        assert_eq!(mem.video, [0; 2048]);
    }
}
