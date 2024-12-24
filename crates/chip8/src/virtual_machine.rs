use crate::error::ExecutionError;
use crate::error::ExecutionError::InvalidOpcode;
use crate::platform::{Platform, Quirks};
use crate::{CHAR_SIZE, PROGRAM_START_ADDRESS};
use log::debug;

const FONTS: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

#[derive(Debug)]
pub struct VirtualMachine {
    pub ram: [u8; 4096],
    pub keypad: [bool; 16],
    pub video: [u8; 2048],
    pub i: u16,
    pub pc: u16,
    pub sp: u8,
    pub dt: u8,
    pub st: u8,
    pub stack: [u16; 16],
    pub registers: [u8; 16],
}

impl VirtualMachine {
    pub fn new() -> Self {
        let mut ram = [0; 4096];
        ram[..FONTS.len()].copy_from_slice(&FONTS);

        Self {
            ram,
            keypad: [false; 16],
            video: [0; 2048],
            i: 0,
            pc: PROGRAM_START_ADDRESS,
            sp: 0,
            dt: 0,
            st: 0,
            stack: [0; 16],
            registers: [0; 16],
        }
    }

    pub fn rand_byte(&self) -> u8 {
        fastrand::u8(..=u8::MAX)
    }

    pub fn execute(&mut self, opcode: u16, platform: &Platform) -> Result<(), ExecutionError> {
        let c = (opcode & 0xF000) >> 12;
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let d = opcode & 0x000F;

        let nn = (opcode & 0x00FF) as u8;
        let nnn = opcode & 0x0FFF;

        match (c, x, y, d) {
            (0x0, 0x0, 0xE, 0x0) => self.op_00e0(),
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
            (0x8, _, _, 0x6) => self.op_8xy6(platform, x, y),
            (0x8, _, _, 0x7) => self.op_8xy7(x, y),
            (0x8, _, _, 0xE) => self.op_8xye(platform, x, y),
            (0x9, _, _, 0x0) => self.op_9xy0(x, y),
            (0xA, _, _, _) => self.op_annn(nnn),
            (0xB, _, _, _) => self.op_bnnn(nnn),
            (0xC, _, _, _) => self.op_cxkk(x, nn),
            (0xD, _, _, _) => self.op_dxyn(platform, x, y, d),
            (0xE, _, 0x9, 0xE) => self.op_ex9e(x),
            (0xE, _, 0xA, 0x1) => self.op_exa1(x),
            (0xF, _, 0x0, 0x7) => self.op_fx07(x),
            (0xF, _, 0x0, 0xA) => self.op_fx0a(x),
            (0xF, _, 0x1, 0x5) => self.op_fx15(x),
            (0xF, _, 0x1, 0x8) => self.op_fx18(x),
            (0xF, _, 0x1, 0xE) => self.op_fx1e(x),
            (0xF, _, 0x2, 0x9) => self.op_fx29(x),
            (0xF, _, 0x3, 0x3) => self.op_fx33(x),
            (0xF, _, 0x5, 0x5) => self.op_fx55(x),
            (0xF, _, 0x6, 0x5) => self.op_fx65(x),
            _ => {
                return Err(InvalidOpcode(opcode));
            }
        }

        Ok(())
    }

    pub fn sne(&mut self) {
        self.pc += 2;
    }

    fn op_00e0(&mut self) {
        debug!("00E0: CLS - Clear screen");

        self.video.fill(0);
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

    fn op_8xy6(&mut self, platform: &Platform, x: usize, y: usize) {
        debug!("8XY6: SHR Vx - Set Vx = Vx SHR 1");

        if !platform.has_quirk(Quirks::SHIFT) {
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

    fn op_8xye(&mut self, platform: &Platform, x: usize, y: usize) {
        debug!("8XYE - SHL Vx - Set Vx = Vx SHL 1");

        if !platform.has_quirk(Quirks::SHIFT) {
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

    fn op_cxkk(&mut self, x: usize, nn: u8) {
        debug!("CXKK: RND Vx, byte - Set Vx = random byte AND byte");

        self.registers[x] = self.rand_byte() & nn;

        self.sne();
    }

    fn op_dxyn(&mut self, platform: &Platform, x: usize, y: usize, n: u16) {
        debug!("DXYN: DRW Vx, Vy, nibble - Display n-byte sprite starting at I to coordinates (Vx, Vy), Set VF = collision");

        let mut collision = 0;
        let (video_width, video_height) = (platform.video_width, platform.video_height);
        let vx = self.registers[x] as u16 % video_width;
        let vy = self.registers[y] as u16 % video_height;

        for display_y in 0..n {
            let pixel = self.ram[(self.i + display_y) as usize];

            for display_x in 0..8 {
                if pixel & (0x80 >> display_x) != 0 {
                    let x_pos = vx + display_x;
                    let y_pos = vy + display_y;

                    if !platform.has_quirk(Quirks::WRAP) && x_pos >= video_width
                        || y_pos >= video_height
                    {
                        continue;
                    }

                    let pixel_pos = (y_pos * video_width + x_pos) as usize;
                    collision |= self.video[pixel_pos] & 1;
                    self.video[pixel_pos] ^= 1;
                }
            }
        }

        self.registers[0xF] = collision;
        self.sne();
    }

    fn op_ex9e(&mut self, x: usize) {
        debug!("EX9E: SKP Vx - Skip next instruction if key with the value of Vx is pressed");

        if self.keypad[self.registers[x] as usize] {
            self.sne();
        }

        self.sne();
    }

    fn op_exa1(&mut self, x: usize) {
        debug!("EXA1: SKNP Vx - Skip next instruction if key with the value of Vx is not pressed");

        if !self.keypad[self.registers[x] as usize] {
            self.sne();
        }

        self.sne();
    }

    fn op_fx07(&mut self, x: usize) {
        debug!("FX07: LD Vx, DT - Set Vx = delay timer");

        self.registers[x] = self.dt;

        self.sne();
    }

    fn op_fx0a(&mut self, x: usize) {
        debug!("FX0A: LD Vx, K - Wait for key press and store the value into Vx");

        if self.keypad[0] {
            self.registers[x] = 0;
            self.sne();
        } else if self.keypad[1] {
            self.registers[x] = 1;
            self.sne();
        } else if self.keypad[2] {
            self.registers[x] = 2;
            self.sne();
        } else if self.keypad[3] {
            self.registers[x] = 3;
            self.sne();
        } else if self.keypad[4] {
            self.registers[x] = 4;
            self.sne();
        } else if self.keypad[5] {
            self.registers[x] = 5;
            self.sne();
        } else if self.keypad[6] {
            self.registers[x] = 6;
            self.sne();
        } else if self.keypad[7] {
            self.registers[x] = 7;
            self.sne();
        } else if self.keypad[8] {
            self.registers[x] = 8;
            self.sne();
        } else if self.keypad[9] {
            self.registers[x] = 9;
            self.sne();
        } else if self.keypad[10] {
            self.registers[x] = 10;
            self.sne();
        } else if self.keypad[11] {
            self.registers[x] = 11;
            self.sne();
        } else if self.keypad[12] {
            self.registers[x] = 12;
            self.sne();
        } else if self.keypad[13] {
            self.registers[x] = 13;
            self.sne();
        } else if self.keypad[14] {
            self.registers[x] = 14;
            self.sne();
        } else if self.keypad[15] {
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

    fn op_fx33(&mut self, x: usize) {
        debug!("FX33: LD B, Vx - Store BCD (Binary-Coded Decimal) representation of Vx in memory locations I, I + 1, and I + 2");

        let mut result = self.registers[x];
        for offset in (0..3).rev() {
            self.ram[(self.i + offset) as usize] = result % 10;
            result /= 10;
        }

        self.sne();
    }

    fn op_fx55(&mut self, x: usize) {
        debug!("FX55: LD [I], Vx - Store V0~Vx in memory starting at location I");

        for offset in 0..=x {
            self.ram[self.i as usize + offset] = self.registers[offset];
        }

        self.sne();
    }

    fn op_fx65(&mut self, x: usize) {
        debug!("FX65: LD Vx, [I] - Read registers V0~Vx from memory starting at location I");

        for offset in 0..=x {
            self.registers[offset] = self.ram[self.i as usize + offset];
        }

        self.sne();
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_00e0() {
        let mut vm = VirtualMachine::new();
        vm.video.fill(1);

        vm.op_00e0();

        assert!(vm.video.iter().all(|&x| x == 0));
    }

    #[test]
    fn test_00ee() {
        let mut vm = VirtualMachine::new();
        vm.sp = 1;
        let return_addr = 0x500;
        vm.stack[0] = return_addr;

        vm.op_00ee();

        assert_eq!(0, vm.sp);
        assert_eq!(return_addr + 2, vm.pc);
    }

    #[test]
    fn test_1nnn() {
        let mut vm = VirtualMachine::new();

        vm.op_1nnn(0x1222);

        assert_eq!(0x1222, vm.pc);
    }

    #[test]
    fn test_2nnn() {
        let mut vm = VirtualMachine::new();
        vm.sp = 0xE;

        vm.op_2nnn(0x2111);

        assert_eq!(0xF, vm.sp);
        assert_eq!(PROGRAM_START_ADDRESS, vm.stack[vm.sp as usize - 1]);
        assert_eq!(0x2111, vm.pc);
    }

    #[test]
    fn test_3xkk() {
        // Test skip case
        let mut vm = VirtualMachine::new();
        let x = 0xE;
        let nn = 0xFF;
        let mut starting_pc = vm.pc;
        vm.registers[x] = nn;

        vm.op_3xkk(x, nn);
        assert_eq!(starting_pc + 4, vm.pc); // Should skip (PC+4)

        // Test no skip case
        vm = VirtualMachine::new();
        starting_pc = vm.pc;
        vm.registers[x] = nn - 1;

        vm.op_3xkk(x, nn);
        assert_eq!(starting_pc + 2, vm.pc); // Should not skip (PC+2)
    }

    #[test]
    fn test_4xkk() {
        // Test skip case
        let mut vm = VirtualMachine::new();
        let x = 0xE;
        let nn = 0xFF;
        let mut starting_pc = vm.pc;
        vm.registers[x] = nn - 1;

        vm.op_4xkk(x, nn);
        assert_eq!(starting_pc + 4, vm.pc); // Should skip (PC+4)

        // Test no skip case
        vm = VirtualMachine::new();
        starting_pc = vm.pc;
        vm.registers[x] = nn;

        vm.op_4xkk(x, nn);
        assert_eq!(starting_pc + 2, vm.pc); // Should not skip (PC+2)
    }

    #[test]
    fn test_5xy0() {
        // Test skip case
        let mut vm = VirtualMachine::new();
        let x = 0xD;
        let y = 0xE;
        let nn = 0xFF;
        let mut starting_pc = vm.pc;
        vm.registers[x] = nn;
        vm.registers[y] = nn;

        vm.op_5xy0(x, y);
        assert_eq!(starting_pc + 4, vm.pc); // Should skip (PC+4)

        // Test no skip case
        vm = VirtualMachine::new();
        starting_pc = vm.pc;
        vm.registers[x] = nn;
        vm.registers[y] = nn - 1;

        vm.op_5xy0(x, y);
        assert_eq!(starting_pc + 2, vm.pc); // Should not skip (PC+2)
    }

    #[test]
    fn test_6xkk() {
        let mut vm = VirtualMachine::new();
        let x = 0xD;
        let nn = 0xFF;
        vm.registers[x] = nn;

        vm.op_6xkk(x, nn);

        assert_eq!(nn, vm.registers[x]);
    }

    #[test]
    fn test_7xkk() {
        let mut vm = VirtualMachine::new();
        let x = 0xD;
        let nn = 0xA;
        vm.registers[x] = 0xE2;
        let expected = vm.registers[x].wrapping_add(nn);

        vm.op_7xkk(x, nn);

        assert_eq!(expected, vm.registers[x]);
    }

    #[test]
    fn test_8xy0() {
        let mut vm = VirtualMachine::new();
        let x = 0xD;
        let y = 0xE;
        vm.registers[y] = 0x1;

        vm.op_8xy0(x, y);

        assert_eq!(vm.registers[x], vm.registers[y]);
    }

    #[test]
    fn test_8xy1() {
        let mut vm = VirtualMachine::new();
        let x = 0xD;
        let y = 0xE;
        vm.registers[x] = 0x8;
        vm.registers[y] = 0xA;
        let expected = vm.registers[x] | vm.registers[y];

        vm.op_8xy1(x, y);

        assert_eq!(expected, vm.registers[x]);
    }

    #[test]
    fn test_8xy2() {
        let mut vm = VirtualMachine::new();
        let x = 0xD;
        let y = 0xE;
        vm.registers[x] = 0x8;
        vm.registers[y] = 0xA;
        let expected = vm.registers[x] & vm.registers[y];

        vm.op_8xy2(x, y);

        assert_eq!(expected, vm.registers[x]);
    }

    #[test]
    fn test_8xy3() {
        let mut vm = VirtualMachine::new();
        let x = 0xD;
        let y = 0xE;
        vm.registers[x] = 0x8;
        vm.registers[y] = 0xA;
        let expected = vm.registers[x] ^ vm.registers[y];

        vm.op_8xy3(x, y);

        assert_eq!(expected, vm.registers[x]);
    }

    #[test]
    fn test_8xy4() {
        let mut vm = VirtualMachine::new();
        let x = 0xD;
        let y = 0xE;
        vm.registers[x] = 0xF;
        vm.registers[y] = 0xF;
        let (expected_result, expected_vf) = vm.registers[x].overflowing_add(vm.registers[y]);

        vm.op_8xy4(x, y);

        assert_eq!(expected_result, vm.registers[x]);
        assert_eq!(expected_vf as u8, vm.registers[0xF]);
    }

    #[test]
    fn test_8xy5() {
        let mut vm = VirtualMachine::new();
        let x = 0xD;
        let y = 0xE;
        vm.registers[x] = 0xF;
        vm.registers[y] = 0xA;
        let (expected_result, did_borrow) = vm.registers[x].overflowing_sub(vm.registers[y]);
        let expected_vf = !did_borrow as u8;

        vm.op_8xy5(x, y);

        assert_eq!(expected_result, vm.registers[x]);
        assert_eq!(expected_vf, vm.registers[0xF]);
    }

    #[test]
    fn test_8xy6() {
        let mut vm = VirtualMachine::new();
        let x = 0xD;
        vm.registers[x] = 0xF;
        let expected_result = vm.registers[x] >> 1;
        let expected_vf = vm.registers[x] & 1;
        let platform = Platform::default();

        vm.op_8xy6(&platform, x, x);

        assert_eq!(expected_result, vm.registers[x]);
        assert_eq!(expected_vf, vm.registers[0xF]);
    }

    #[test]
    fn test_8xy7() {
        let mut vm = VirtualMachine::new();
        let x = 0xD;
        let y = 0xE;
        vm.registers[x] = 0xA;
        vm.registers[y] = 0xF;
        let (expected_result, did_borrow) = vm.registers[y].overflowing_sub(vm.registers[x]);
        let expected_vf = !did_borrow as u8;

        vm.op_8xy7(x, y);

        assert_eq!(expected_result, vm.registers[x]);
        assert_eq!(expected_vf, vm.registers[0xF]);
    }

    #[test]
    fn test_8xye() {
        let mut vm = VirtualMachine::new();
        let x = 0xD;
        vm.registers[x] = 0xF;
        let expected_result = vm.registers[x] << 1;
        let expected_vf = (vm.registers[x] & 0x80) >> 7;
        let platform = Platform::default();

        vm.op_8xye(&platform, x, x);

        assert_eq!(expected_result, vm.registers[x]);
        assert_eq!(expected_vf, vm.registers[0xF]);
    }

    #[test]
    fn test_9xy0() {
        // Skip case
        let mut vm = VirtualMachine::new();
        let x = 0xD;
        let y = 0xE;
        let mut starting_pc = vm.pc;
        vm.registers[x] = 0x5;
        vm.registers[y] = 0x6;

        vm.op_9xy0(x, y);
        assert_eq!(starting_pc + 4, vm.pc); // Should skip (PC+4)

        // No skip case
        vm = VirtualMachine::new();
        starting_pc = vm.pc;
        vm.registers[x] = 0x5;
        vm.registers[y] = 0x5;

        vm.op_9xy0(x, y);
        assert_eq!(starting_pc + 2, vm.pc); // Should not skip (PC+2)
    }

    #[test]
    fn test_annn() {
        let mut vm = VirtualMachine::new();
        let nnn = 0x100;

        vm.op_annn(nnn);

        assert_eq!(nnn, vm.i);
    }

    #[test]
    fn test_bnnn() {
        let mut vm = VirtualMachine::new();
        let nnn = 0x200;
        vm.registers[0] = 0x5;
        let expected = nnn + vm.registers[0] as u16;

        vm.op_bnnn(nnn);

        assert_eq!(expected + 2, vm.pc); // Jump target + 2
    }

    #[test]
    fn test_cxkk() {
        let mut vm = VirtualMachine::new();
        let x = 0xD;
        let nn = 0xE;
        vm.registers[x] = nn;

        vm.op_cxkk(x, nn);

        assert_ne!(nn, vm.registers[x]);
    }

    #[test]
    fn test_op_dxyn() {
        let mut vm = VirtualMachine::new();
        vm.i = 0x200;
        vm.ram[vm.i as usize] = 0x1;
        vm.video[0x7] = 0x1;
        let platform = Platform::default();

        vm.op_dxyn(&platform, 0, 0, 1);

        assert_eq!(0x0, vm.video[0x7]);
        assert_eq!(0x1, vm.registers[0xF]);
    }

    #[test]
    fn test_ex9e() {
        // Skip case
        let mut vm = VirtualMachine::new();
        let x = 0xD;
        let mut starting_pc = vm.pc;

        vm.registers[x] = 0x5;
        vm.keypad[0x5] = true;
        vm.op_ex9e(x);

        assert_eq!(starting_pc + 4, vm.pc); // Should skip (PC+4)

        // No skip case
        vm = VirtualMachine::new();
        starting_pc = vm.pc;

        vm.registers[x] = 0x5;
        vm.op_ex9e(x);

        assert_eq!(starting_pc + 2, vm.pc); // Should not skip (PC+2)
    }

    #[test]
    fn test_exa1() {
        // Skip case
        let mut vm = VirtualMachine::new();
        let x = 0xD;
        let mut starting_pc = vm.pc;
        vm.registers[x] = 0x5;

        vm.op_exa1(x);
        assert_eq!(starting_pc + 4, vm.pc); // Should skip (PC+4)

        // No skip case
        vm = VirtualMachine::new();
        starting_pc = vm.pc;
        vm.registers[x] = 0x5;
        vm.keypad[0x5] = true;

        vm.op_exa1(x);
        assert_eq!(starting_pc + 2, vm.pc); // Should not skip (PC+2)
    }

    #[test]
    fn test_fx07() {
        let mut vm = VirtualMachine::new();
        let x = 0xD;
        vm.registers[x] = 0x1;
        vm.dt = 0xF;

        vm.op_fx07(x);

        assert_eq!(vm.registers[x], vm.dt);
    }

    #[test]
    fn test_fx0a() {
        let mut vm = VirtualMachine::new();
        let x = 0xD;
        let starting_pc = vm.pc;
        vm.keypad[5] = true;

        vm.op_fx0a(x);

        assert_eq!(5, vm.registers[x]); // Register set to pressed key
        assert_eq!(starting_pc + 2, vm.pc); // PC incremented
    }

    #[test]
    fn test_fx15() {
        let mut vm = VirtualMachine::new();
        let x = 0xD;
        vm.registers[x] = 0x11;
        vm.dt = 0x12;

        vm.op_fx15(x);

        assert_eq!(vm.dt, vm.registers[x]);
    }

    #[test]
    fn test_fx18() {
        let mut vm = VirtualMachine::new();
        let x = 0xD;
        vm.registers[x] = 0x11;
        vm.st = 0x12;

        vm.op_fx18(x);

        assert_eq!(vm.st, vm.registers[x]);
    }

    #[test]
    fn test_fx1e() {
        let mut vm = VirtualMachine::new();
        let x = 0xD;
        vm.registers[x] = 0x11;
        vm.i = 0x12;
        let expected = vm.registers[x] as u16 + vm.i;

        vm.op_fx1e(x);

        assert_eq!(expected, vm.i);
    }

    #[test]
    fn test_fx29() {
        let mut vm = VirtualMachine::new();
        let x = 0xD;
        vm.registers[x] = 0x11;
        vm.i = 0x12;
        let expected = vm.registers[x] as u16 * CHAR_SIZE as u16;

        vm.op_fx29(x);

        assert_eq!(expected, vm.i);
    }

    #[test]
    fn test_fx33() {
        let mut vm = VirtualMachine::new();
        let x = 0xD;
        vm.registers[x] = 0xFF;
        vm.i = 0xFF;
        let mut result = vm.registers[x];
        let expected1 = result % 10;
        result /= 10;
        let expected2 = result % 10;
        result /= 10;
        let expected3 = result % 10;

        vm.op_fx33(x);

        assert_eq!(expected1, vm.ram[vm.i as usize + 2]);
        assert_eq!(expected2, vm.ram[vm.i as usize + 1]);
        assert_eq!(expected3, vm.ram[vm.i as usize]);
    }

    #[test]
    fn test_fx55() {
        let mut vm = VirtualMachine::new();
        let x = 0x2;
        let expected = 0xEF;
        for offset in 0..=x {
            vm.registers[offset] = expected;
        }

        vm.op_fx55(x);

        assert_eq!(expected, vm.ram[vm.i as usize]);
        assert_eq!(expected, vm.ram[vm.i as usize + 1]);
        assert_eq!(expected, vm.ram[vm.i as usize + 2]);
    }

    #[test]
    fn test_fx65() {
        let mut vm = VirtualMachine::new();
        let x = 0x2;
        let expected = 0xEF;
        for offset in 0..=x {
            vm.ram[vm.i as usize + offset] = expected;
        }

        vm.op_fx65(x);

        assert_eq!(expected, vm.registers[0]);
        assert_eq!(expected, vm.registers[1]);
        assert_eq!(expected, vm.registers[2]);
    }
}
