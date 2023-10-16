use bitflags::bitflags;

use super::Bus;
use super::Instruction;

bitflags! {
    pub struct CpuStatusRegister: u8 {
        const C = 1;
        const Z = 1 << 1;
        const I = 1 << 2;
        const D = 1 << 3;
        const B = 1 << 4;
        const U = 1 << 5;
        const V = 1 << 6;
        const N = 1 << 7;
    }
}

pub struct Cpu {
    pub cycle: u64,
    pub pc: u16,                    // Program Counter
    pub sp: u8,                     // Stack Pointer
    pub r_a: u8,                    // Accumulator
    pub r_x: u8,                    // X Register
    pub r_y: u8,                    // Y Register
    pub status: CpuStatusRegister,  // Status Register
    pub bus: Bus
}

impl Cpu {
    pub fn step(&mut self) {
        let opcode = self.bus.read(self.pc);
        self.pc += 1;

        let instruction = Instruction::from_u8(opcode);
    }

    fn trace(&self, instruction: &Instruction, data: Option<u8>) -> &str {
        todo!()
    }
}
