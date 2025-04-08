use core::panic;

use super::{cpu::CpuStatusRegister, Cpu};

#[derive(Debug, Eq, PartialEq)]
pub enum Operation {
    BRK,
    ORA,
    ASL,
    PHP,
    BPL,
    CLC,
    JSR,
    AND,
    BIT,
    ROL,
    PLP,
    BMI,
    SEC,
    RTI,
    EOR,
    LSR,
    PHA,
    JMP,
    BVC,
    CLI,
    RTS,
    ADC,
    ROR,
    PLA,
    STA,
    STY,
    STX,
    DEY,
    TXA,
    BCC,
    INC,
    SBC,
    SEI,
    BVS,
    TYA,
    TXS,
    LDY,
    LDA,
    LDX,
    TAY,
    TAX,
    BCS,
    CLV,
    TSX,
    CPY,
    CMP,
    DEX,
    DEC,
    INY,
    BNE,
    CLD,
    CPX,
    INX,
    NOP,
    BEQ,
    SED,
    UnknownOperation,
}

pub enum AddressingMode {
    Implied,
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Relative,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
    IndirectX,
    IndirectY,
    Accumulator,
}

impl AddressingMode {
    fn offset(&self) -> u16 {
        match &self {
            &Self::Implied | &Self::Accumulator => 0,
            &Self::ZeroPage
            | &Self::ZeroPageX
            | &Self::ZeroPageY
            | &Self::Indirect
            | &Self::Immediate
            | &Self::Relative
            | &Self::IndirectX
            | &Self::IndirectY => 1,
            _ => 2,
        }
    }
}

pub struct Instruction {
    pub operation: Operation,
    pub address_mode: AddressingMode,
    pub cycles: u8,
}

pub struct InstructionLoadData(Option<u16>, bool);

impl Cpu {
    pub fn load_instruction_address(
        &mut self,
        address_mode: &AddressingMode,
    ) -> InstructionLoadData {
        match address_mode {
            AddressingMode::Immediate => InstructionLoadData(Some(self.pc), false),
            AddressingMode::ZeroPage => {
                InstructionLoadData(Some(self.bus.read(self.pc).into()), false)
            }
            AddressingMode::ZeroPageX => {
                let address = self.bus.read(self.pc).wrapping_add(self.r_x);
                InstructionLoadData(Some(address.into()), false)
            }
            AddressingMode::ZeroPageY => {
                let address = self.bus.read(self.pc).wrapping_add(self.r_y);
                InstructionLoadData(Some(address.into()), false)
            }
            AddressingMode::Relative => {
                InstructionLoadData(Some(self.bus.read(self.pc).into()), false)
            }
            AddressingMode::Absolute => {
                let address = self.bus.read_u16(self.pc);
                InstructionLoadData(Some(address), false)
            }
            AddressingMode::AbsoluteX => {
                let base_address = self.bus.read_u16(self.pc);
                let absolute_address = base_address.wrapping_add(u16::from(self.r_x));
                InstructionLoadData(
                    Some(absolute_address),
                    Cpu::page_cross(base_address, absolute_address),
                )
            }
            AddressingMode::AbsoluteY => {
                let base_address = self.bus.read_u16(self.pc);
                let absolute_address = base_address.wrapping_add(u16::from(self.r_y));
                InstructionLoadData(
                    Some(absolute_address),
                    Cpu::page_cross(base_address, absolute_address),
                )
            }
            AddressingMode::Indirect => {
                let address = self.bus.read_u16(self.pc);

                if address & 0xFF == 0xFF {
                    let lo = self.bus.read(address);
                    let hi = self.bus.read(address & 0xFF00);
                    return InstructionLoadData(Some(u16::from_le_bytes([lo, hi])), false);
                }

                InstructionLoadData(Some(self.bus.read_u16(address)), false)
            }
            AddressingMode::IndirectX => {
                let address = self.bus.read(self.pc).wrapping_add(self.r_x);
                let absolute_address = self.bus.read_u16_zp(address);

                InstructionLoadData(Some(absolute_address), false)
            }
            AddressingMode::IndirectY => {
                let address = self.bus.read(self.pc);
                let relative_address = self.bus.read_u16_zp(address);
                let absolute_address = relative_address.wrapping_add(self.r_y.into());

                InstructionLoadData(
                    Some(absolute_address),
                    Cpu::page_cross(relative_address, absolute_address),
                )
            }
            _ => InstructionLoadData(None, false),
        }
    }

    pub fn load_instruction_data(
        &mut self,
        address_mode: &AddressingMode,
        instruction: &Instruction,
        address: Option<u16>,
    ) -> u8 {
        match address_mode {
            AddressingMode::Implied | AddressingMode::Relative => 0,
            AddressingMode::Immediate => self.bus.read(self.pc),
            AddressingMode::Accumulator => self.r_a,
            _ => self.bus.read(match instruction.operation {
                Operation::STA => return 0,
                Operation::STX => return 0,
                Operation::STY => return 0,
                _ => address
                    .unwrap_or_else(|| panic!("No address provided for addressing instruction")),
            }),
        }
    }

    pub fn execute_instruction(&mut self, instruction: &Instruction) -> u8 {
        self.pc = self.pc.wrapping_add(1);
        let instruction_load_data = self.load_instruction_address(&instruction.address_mode);
        let instruction_data = self.load_instruction_data(
            &instruction.address_mode,
            instruction,
            instruction_load_data.0,
        );

        self.pc = self.pc.wrapping_add(instruction.address_mode.offset());

        let mut cycles = instruction.cycles
            + if instruction.operation == Operation::STA
                || instruction.operation == Operation::STX
                || instruction.operation == Operation::STY
            {
                0
            } else {
                instruction_load_data.1 as u8
            };

        match instruction.operation {
            Operation::ADC => {
                let sum = self.r_a as u16
                    + instruction_data as u16
                    + self.status.intersects(CpuStatusRegister::C) as u16;
                self.status.set(CpuStatusRegister::C, sum > 0xFF);
                self.status.set(CpuStatusRegister::Z, (sum & 0xFF) == 0);
                self.status.set(
                    CpuStatusRegister::V,
                    (!((self.r_a as u16) ^ instruction_data as u16)
                        & ((self.r_a as u16) ^ sum)
                        & 0x0080)
                        != 0,
                );
                self.status.set(CpuStatusRegister::N, (sum & 0x80) != 0);
                self.r_a = (sum & 0xFF) as u8;
            }
            Operation::AND => {
                self.r_a &= instruction_data;
                self.set_zn(self.r_a);
            }
            // Shift left 1 bit
            Operation::ASL => {
                self.status
                    .set(CpuStatusRegister::C, (instruction_data >> 7) & 1 != 0);
                let value = instruction_data.wrapping_shl(1);
                self.set_zn(value);
                self.write_fetched(&instruction.address_mode, instruction_load_data.0, value);
            }
            // Branch on carry clear
            Operation::BCC => {
                if !self.status.intersects(CpuStatusRegister::C) {
                    cycles += self.branch(instruction_load_data.0.unwrap());
                }
            }
            // Branch on carry set
            Operation::BCS => {
                if self.status.intersects(CpuStatusRegister::C) {
                    cycles += self.branch(instruction_load_data.0.unwrap());
                }
            }
            // Branch on equal
            Operation::BEQ => {
                if self.status.intersects(CpuStatusRegister::Z) {
                    cycles += self.branch(instruction_load_data.0.unwrap());
                }
            }
            // Bit Test
            Operation::BIT => {
                let value = self.r_a & instruction_data;
                self.status.set(CpuStatusRegister::Z, value == 0);
                self.status
                    .set(CpuStatusRegister::N, instruction_data & (1 << 7) != 0);
                self.status
                    .set(CpuStatusRegister::V, instruction_data & (1 << 6) != 0);
            }
            // Branch on result minus
            Operation::BMI => {
                if self.status.intersects(CpuStatusRegister::N) {
                    cycles += self.branch(instruction_load_data.0.unwrap());
                }
            }
            // Branch on not equal
            Operation::BNE => {
                if !self.status.intersects(CpuStatusRegister::Z) {
                    cycles += self.branch(instruction_load_data.0.unwrap());
                }
            }
            // Branch on prediction positive
            Operation::BPL => {
                if !self.status.intersects(CpuStatusRegister::N) {
                    cycles += self.branch(instruction_load_data.0.unwrap());
                }
            }
            // Force break interrupt
            Operation::BRK => {
                self.push_u16(self.pc);

                let status = (self.status | CpuStatusRegister::U | CpuStatusRegister::B).bits();

                self.push(status);
                self.status.set(CpuStatusRegister::I, true);

                self.pc = self.bus.read_u16(0xFFFE);
            }
            // Branch on overflow clear
            Operation::BVC => {
                if !self.status.intersects(CpuStatusRegister::V) {
                    cycles += self.branch(instruction_load_data.0.unwrap());
                }
            }
            // Branch on overflow set
            Operation::BVS => {
                if self.status.intersects(CpuStatusRegister::V) {
                    cycles += self.branch(instruction_load_data.0.unwrap());
                }
            }
            // Clear carry flag
            Operation::CLC => {
                self.status.set(CpuStatusRegister::C, false);
            }
            // Clear decimal flag
            Operation::CLD => {
                self.status.set(CpuStatusRegister::D, false);
            }
            // Clear interrupt disable flag
            Operation::CLI => {
                self.status.set(CpuStatusRegister::I, false);
            }
            // Clear overflow flag
            Operation::CLV => {
                self.status.set(CpuStatusRegister::V, false);
            }
            // Compare memory to accumulator
            Operation::CMP => {
                self.compare(self.r_a, instruction_data);
            }
            // Compare memory to x register
            Operation::CPX => {
                self.compare(self.r_x, instruction_data);
            }
            // Compare memory to y register
            Operation::CPY => {
                self.compare(self.r_y, instruction_data);
            }
            // Decrement memory
            Operation::DEC => {
                let value = instruction_data.wrapping_sub(1);
                self.write_fetched(&instruction.address_mode, instruction_load_data.0, value);
                self.set_zn(value);
            }
            // Decrement X register
            Operation::DEX => {
                let value = self.r_x.wrapping_sub(1);
                self.r_x = value;
                self.set_zn(value);
            }
            // Decrement Y register
            Operation::DEY => {
                let value = self.r_y.wrapping_sub(1);
                self.r_y = value;
                self.set_zn(value);
            }
            // Exclusive-OR accumulator with memory
            Operation::EOR => {
                self.r_a ^= instruction_data;
                self.set_zn(self.r_a);
            }
            // Increment memory
            Operation::INC => {
                let value = instruction_data.wrapping_add(1);
                self.write_fetched(&instruction.address_mode, instruction_load_data.0, value);
                self.set_zn(value);
            }
            // Increment X register
            Operation::INX => {
                let value = self.r_x.wrapping_add(1);
                self.r_x = value;
                self.set_zn(value);
            }
            // Increment Y register
            Operation::INY => {
                let value = self.r_y.wrapping_add(1);
                self.r_y = value;
                self.set_zn(value);
            }
            // Jump
            Operation::JMP => {
                self.pc = instruction_load_data.0.unwrap();
            }
            // Jump and save return
            Operation::JSR => {
                self.push_u16(self.pc.wrapping_sub(1));
                self.pc = instruction_load_data.0.unwrap();
            }
            // Load memory into accumulator
            Operation::LDA => {
                self.r_a = instruction_data;
                self.set_zn(self.r_a);
            }
            // Load memory into X register
            Operation::LDX => {
                self.r_x = instruction_data;
                self.set_zn(self.r_x);
            }
            // Load memory into Y register
            Operation::LDY => {
                self.r_y = instruction_data;
                self.set_zn(self.r_y);
            }
            // Shift right
            Operation::LSR => {
                self.status
                    .set(CpuStatusRegister::C, instruction_data & 1 == 1);
                let value = instruction_data.wrapping_shr(1);
                self.write_fetched(&instruction.address_mode, instruction_load_data.0, value);
                self.set_zn(value);
            }
            // No-op
            Operation::NOP => {}
            // OR accumulator
            Operation::ORA => {
                self.r_a |= instruction_data;
                self.set_zn(self.r_a);
            }
            // Push accumulator onto stack
            Operation::PHA => {
                self.push(self.r_a);
            }
            // Push status register to stack
            Operation::PHP => {
                self.push((self.status | CpuStatusRegister::U | CpuStatusRegister::B).bits());
            }
            // Pop stack into accumulator
            Operation::PLA => {
                self.r_a = self.pop();
                self.set_zn(self.r_a);
            }
            // Pop stack into status register
            Operation::PLP => {
                self.status = CpuStatusRegister::from_bits_truncate(self.pop());
                self.status.set(CpuStatusRegister::B, false);
                self.status.set(CpuStatusRegister::U, true);
            }
            // Rotate one bit left
            Operation::ROL => {
                let c = self.status.intersection(CpuStatusRegister::C).bits();
                self.status
                    .set(CpuStatusRegister::C, (instruction_data >> 7) & 1 != 0);
                let value = (instruction_data << 1) | c;
                self.set_zn(value);
                self.write_fetched(&instruction.address_mode, instruction_load_data.0, value);
            }
            // Rotate one bit right
            Operation::ROR => {
                let mut value = instruction_data.rotate_right(1);
                if self.status.intersects(CpuStatusRegister::C) {
                    value |= 1 << 7;
                } else {
                    value &= !(1 << 7);
                }
                self.status
                    .set(CpuStatusRegister::C, instruction_data & 1 != 0);
                self.set_zn(value);
                self.write_fetched(&instruction.address_mode, instruction_load_data.0, value);
            }
            // Return from interrupt
            Operation::RTI => {
                self.status = CpuStatusRegister::from_bits_truncate(self.pop())
                    & !CpuStatusRegister::B
                    | CpuStatusRegister::U;
                self.pc = self.pop_u16();
            }
            // Return from subroutine
            Operation::RTS => {
                self.pc = self.pop_u16().wrapping_add(1);
            }
            // Subtract memory from accumulator
            Operation::SBC => {
                let inverted = (instruction_data as u16) ^ 0x00FF;
                let difference = self.r_a as u16
                    + inverted
                    + self.status.intersects(CpuStatusRegister::C) as u16;

                self.status
                    .set(CpuStatusRegister::C, difference & 0xFF00 != 0);
                self.status
                    .set(CpuStatusRegister::Z, difference & 0xFF == 0);
                self.status.set(
                    CpuStatusRegister::V,
                    ((difference ^ self.r_a as u16) & (difference ^ inverted) & 0x0080) != 0,
                );
                self.status
                    .set(CpuStatusRegister::N, difference & 0x80 != 0);
                self.r_a = (difference & 0xFF) as u8;
            }
            // Set carry flag
            Operation::SEC => {
                self.status.set(CpuStatusRegister::C, true);
            }
            // Set decimal flag
            Operation::SED => {
                self.status.set(CpuStatusRegister::D, true);
            }
            // Set interrupt disable flag
            Operation::SEI => {
                self.status.set(CpuStatusRegister::I, true);
            }
            // Store accumulator in memory
            Operation::STA => {
                self.bus.write(instruction_load_data.0.unwrap(), self.r_a);
            }
            // Store register X in memory
            Operation::STX => {
                self.bus.write(instruction_load_data.0.unwrap(), self.r_x);
            }
            // Store register Y in memory
            Operation::STY => {
                self.bus.write(instruction_load_data.0.unwrap(), self.r_y);
            }
            // Transfer accumulator to X register
            Operation::TAX => {
                self.r_x = self.r_a;
                self.set_zn(self.r_x);
            }
            // Transfer accumulator to Y register
            Operation::TAY => {
                self.r_y = self.r_a;
                self.set_zn(self.r_y);
            }
            // Transfer stack pointer to X register
            Operation::TSX => {
                self.r_x = self.sp;
                self.set_zn(self.r_x);
            }
            // Transfer X register to accumulator
            Operation::TXA => {
                self.r_a = self.r_x;
                self.set_zn(self.r_a);
            }
            // Transfer X register to stack pointer
            Operation::TXS => {
                self.sp = self.r_x;
            }
            // Transfer Y register to accumulator
            Operation::TYA => {
                self.r_a = self.r_y;
                self.set_zn(self.r_a);
            }
            Operation::UnknownOperation => panic!("Not implemented"),
        }

        return cycles;
    }

    fn compare(&mut self, lhs: u8, rhs: u8) {
        let value = lhs.wrapping_sub(rhs);
        self.set_zn(value);
        self.status.set(CpuStatusRegister::C, lhs >= rhs);
    }
}

impl Instruction {
    pub fn from_u8(value: u8) -> Instruction {
        return match value {
            // 0x0*
            0x00 => Instruction {
                operation: Operation::BRK,
                address_mode: AddressingMode::Implied,
                cycles: 7,
            },
            0x01 => Instruction {
                operation: Operation::ORA,
                address_mode: AddressingMode::IndirectX,
                cycles: 6,
            },
            0x05 => Instruction {
                operation: Operation::ORA,
                address_mode: AddressingMode::ZeroPage,
                cycles: 3,
            },
            0x06 => Instruction {
                operation: Operation::ASL,
                address_mode: AddressingMode::ZeroPage,
                cycles: 5,
            },
            0x08 => Instruction {
                operation: Operation::PHP,
                address_mode: AddressingMode::Implied,
                cycles: 3,
            },
            0x09 => Instruction {
                operation: Operation::ORA,
                address_mode: AddressingMode::Immediate,
                cycles: 2,
            },
            0x0A => Instruction {
                operation: Operation::ASL,
                address_mode: AddressingMode::Accumulator,
                cycles: 2,
            },
            0x0D => Instruction {
                operation: Operation::ORA,
                address_mode: AddressingMode::Absolute,
                cycles: 4,
            },
            0x0E => Instruction {
                operation: Operation::ASL,
                address_mode: AddressingMode::Absolute,
                cycles: 6,
            },
            // 0x1*
            0x10 => Instruction {
                operation: Operation::BPL,
                address_mode: AddressingMode::Relative,
                cycles: 2,
            },
            0x11 => Instruction {
                operation: Operation::ORA,
                address_mode: AddressingMode::IndirectY,
                cycles: 5,
            },
            0x15 => Instruction {
                operation: Operation::ORA,
                address_mode: AddressingMode::ZeroPageX,
                cycles: 4,
            },
            0x16 => Instruction {
                operation: Operation::ASL,
                address_mode: AddressingMode::ZeroPageX,
                cycles: 6,
            },
            0x18 => Instruction {
                operation: Operation::CLC,
                address_mode: AddressingMode::Implied,
                cycles: 2,
            },
            0x19 => Instruction {
                operation: Operation::ORA,
                address_mode: AddressingMode::AbsoluteY,
                cycles: 4,
            },
            0x1D => Instruction {
                operation: Operation::ORA,
                address_mode: AddressingMode::AbsoluteX,
                cycles: 4,
            },
            0x1E => Instruction {
                operation: Operation::ASL,
                address_mode: AddressingMode::AbsoluteX,
                cycles: 7,
            },
            // 0x2*
            0x20 => Instruction {
                operation: Operation::JSR,
                address_mode: AddressingMode::Absolute,
                cycles: 6,
            },
            0x21 => Instruction {
                operation: Operation::AND,
                address_mode: AddressingMode::IndirectX,
                cycles: 6,
            },
            0x24 => Instruction {
                operation: Operation::BIT,
                address_mode: AddressingMode::ZeroPage,
                cycles: 3,
            },
            0x25 => Instruction {
                operation: Operation::AND,
                address_mode: AddressingMode::ZeroPage,
                cycles: 3,
            },
            0x26 => Instruction {
                operation: Operation::ROL,
                address_mode: AddressingMode::ZeroPage,
                cycles: 5,
            },
            0x28 => Instruction {
                operation: Operation::PLP,
                address_mode: AddressingMode::Implied,
                cycles: 4,
            },
            0x29 => Instruction {
                operation: Operation::AND,
                address_mode: AddressingMode::Immediate,
                cycles: 2,
            },
            0x2A => Instruction {
                operation: Operation::ROL,
                address_mode: AddressingMode::Accumulator,
                cycles: 2,
            },
            0x2C => Instruction {
                operation: Operation::BIT,
                address_mode: AddressingMode::Absolute,
                cycles: 4,
            },
            0x2D => Instruction {
                operation: Operation::AND,
                address_mode: AddressingMode::Absolute,
                cycles: 4,
            },
            0x2E => Instruction {
                operation: Operation::ROL,
                address_mode: AddressingMode::Absolute,
                cycles: 6,
            },
            // 0x3*
            0x30 => Instruction {
                operation: Operation::BMI,
                address_mode: AddressingMode::Relative,
                cycles: 2,
            },
            0x31 => Instruction {
                operation: Operation::AND,
                address_mode: AddressingMode::IndirectY,
                cycles: 5,
            },
            0x35 => Instruction {
                operation: Operation::AND,
                address_mode: AddressingMode::ZeroPageX,
                cycles: 4,
            },
            0x36 => Instruction {
                operation: Operation::ROL,
                address_mode: AddressingMode::ZeroPageX,
                cycles: 6,
            },
            0x38 => Instruction {
                operation: Operation::SEC,
                address_mode: AddressingMode::Implied,
                cycles: 2,
            },
            0x39 => Instruction {
                operation: Operation::AND,
                address_mode: AddressingMode::AbsoluteY,
                cycles: 4,
            },
            0x3D => Instruction {
                operation: Operation::AND,
                address_mode: AddressingMode::AbsoluteX,
                cycles: 4,
            },
            0x3E => Instruction {
                operation: Operation::ROL,
                address_mode: AddressingMode::AbsoluteX,
                cycles: 7,
            },
            // 0x4*
            0x40 => Instruction {
                operation: Operation::RTI,
                address_mode: AddressingMode::Implied,
                cycles: 6,
            },
            0x41 => Instruction {
                operation: Operation::EOR,
                address_mode: AddressingMode::IndirectX,
                cycles: 6,
            },
            0x45 => Instruction {
                operation: Operation::EOR,
                address_mode: AddressingMode::ZeroPage,
                cycles: 3,
            },
            0x46 => Instruction {
                operation: Operation::LSR,
                address_mode: AddressingMode::ZeroPage,
                cycles: 5,
            },
            0x48 => Instruction {
                operation: Operation::PHA,
                address_mode: AddressingMode::Implied,
                cycles: 3,
            },
            0x49 => Instruction {
                operation: Operation::EOR,
                address_mode: AddressingMode::Immediate,
                cycles: 2,
            },
            0x4A => Instruction {
                operation: Operation::LSR,
                address_mode: AddressingMode::Accumulator,
                cycles: 2,
            },
            0x4C => Instruction {
                operation: Operation::JMP,
                address_mode: AddressingMode::Absolute,
                cycles: 3,
            },
            0x4D => Instruction {
                operation: Operation::EOR,
                address_mode: AddressingMode::Absolute,
                cycles: 4,
            },
            0x4E => Instruction {
                operation: Operation::LSR,
                address_mode: AddressingMode::Absolute,
                cycles: 6,
            },
            // 0x5*
            0x50 => Instruction {
                operation: Operation::BVC,
                address_mode: AddressingMode::Relative,
                cycles: 2,
            },
            0x51 => Instruction {
                operation: Operation::EOR,
                address_mode: AddressingMode::IndirectY,
                cycles: 5,
            },
            0x55 => Instruction {
                operation: Operation::EOR,
                address_mode: AddressingMode::ZeroPageX,
                cycles: 4,
            },
            0x56 => Instruction {
                operation: Operation::LSR,
                address_mode: AddressingMode::ZeroPageX,
                cycles: 6,
            },
            0x58 => Instruction {
                operation: Operation::CLI,
                address_mode: AddressingMode::Implied,
                cycles: 2,
            },
            0x59 => Instruction {
                operation: Operation::EOR,
                address_mode: AddressingMode::AbsoluteY,
                cycles: 4,
            },
            0x5D => Instruction {
                operation: Operation::EOR,
                address_mode: AddressingMode::AbsoluteX,
                cycles: 4,
            },
            0x5E => Instruction {
                operation: Operation::LSR,
                address_mode: AddressingMode::AbsoluteX,
                cycles: 7,
            },
            // 0x6*
            0x60 => Instruction {
                operation: Operation::RTS,
                address_mode: AddressingMode::Implied,
                cycles: 6,
            },
            0x61 => Instruction {
                operation: Operation::ADC,
                address_mode: AddressingMode::IndirectX,
                cycles: 6,
            },
            0x65 => Instruction {
                operation: Operation::ADC,
                address_mode: AddressingMode::ZeroPage,
                cycles: 3,
            },
            0x66 => Instruction {
                operation: Operation::ROR,
                address_mode: AddressingMode::ZeroPage,
                cycles: 5,
            },
            0x68 => Instruction {
                operation: Operation::PLA,
                address_mode: AddressingMode::Implied,
                cycles: 4,
            },
            0x69 => Instruction {
                operation: Operation::ADC,
                address_mode: AddressingMode::Immediate,
                cycles: 2,
            },
            0x6A => Instruction {
                operation: Operation::ROR,
                address_mode: AddressingMode::Accumulator,
                cycles: 2,
            },
            0x6C => Instruction {
                operation: Operation::JMP,
                address_mode: AddressingMode::Indirect,
                cycles: 5,
            },
            0x6D => Instruction {
                operation: Operation::ADC,
                address_mode: AddressingMode::Absolute,
                cycles: 4,
            },
            0x6E => Instruction {
                operation: Operation::ROR,
                address_mode: AddressingMode::Absolute,
                cycles: 6,
            },
            // 0x7*
            0x70 => Instruction {
                operation: Operation::BVS,
                address_mode: AddressingMode::Relative,
                cycles: 2,
            },
            0x71 => Instruction {
                operation: Operation::ADC,
                address_mode: AddressingMode::IndirectY,
                cycles: 5,
            },
            0x75 => Instruction {
                operation: Operation::ADC,
                address_mode: AddressingMode::ZeroPageX,
                cycles: 4,
            },
            0x76 => Instruction {
                operation: Operation::ROR,
                address_mode: AddressingMode::ZeroPageX,
                cycles: 6,
            },
            0x78 => Instruction {
                operation: Operation::SEI,
                address_mode: AddressingMode::Implied,
                cycles: 2,
            },
            0x79 => Instruction {
                operation: Operation::ADC,
                address_mode: AddressingMode::AbsoluteY,
                cycles: 4,
            },
            0x7D => Instruction {
                operation: Operation::ADC,
                address_mode: AddressingMode::AbsoluteX,
                cycles: 4,
            },
            0x7E => Instruction {
                operation: Operation::ROR,
                address_mode: AddressingMode::AbsoluteX,
                cycles: 7,
            },
            // 0x8*
            0x81 => Instruction {
                operation: Operation::STA,
                address_mode: AddressingMode::IndirectX,
                cycles: 6,
            },
            0x84 => Instruction {
                operation: Operation::STY,
                address_mode: AddressingMode::ZeroPage,
                cycles: 3,
            },
            0x85 => Instruction {
                operation: Operation::STA,
                address_mode: AddressingMode::ZeroPage,
                cycles: 3,
            },
            0x86 => Instruction {
                operation: Operation::STX,
                address_mode: AddressingMode::ZeroPage,
                cycles: 3,
            },
            0x88 => Instruction {
                operation: Operation::DEY,
                address_mode: AddressingMode::Implied,
                cycles: 2,
            },
            0x8A => Instruction {
                operation: Operation::TXA,
                address_mode: AddressingMode::Implied,
                cycles: 2,
            },
            0x8C => Instruction {
                operation: Operation::STY,
                address_mode: AddressingMode::Absolute,
                cycles: 4,
            },
            0x8D => Instruction {
                operation: Operation::STA,
                address_mode: AddressingMode::Absolute,
                cycles: 4,
            },
            0x8E => Instruction {
                operation: Operation::STX,
                address_mode: AddressingMode::Absolute,
                cycles: 4,
            },
            // 0x9*
            0x90 => Instruction {
                operation: Operation::BCC,
                address_mode: AddressingMode::Relative,
                cycles: 2,
            },
            0x91 => Instruction {
                operation: Operation::STA,
                address_mode: AddressingMode::IndirectY,
                cycles: 6,
            },
            0x94 => Instruction {
                operation: Operation::STY,
                address_mode: AddressingMode::ZeroPageX,
                cycles: 4,
            },
            0x95 => Instruction {
                operation: Operation::STA,
                address_mode: AddressingMode::ZeroPageX,
                cycles: 4,
            },
            0x96 => Instruction {
                operation: Operation::STX,
                address_mode: AddressingMode::ZeroPageY,
                cycles: 4,
            },
            0x98 => Instruction {
                operation: Operation::TYA,
                address_mode: AddressingMode::Implied,
                cycles: 2,
            },
            0x99 => Instruction {
                operation: Operation::STA,
                address_mode: AddressingMode::AbsoluteY,
                cycles: 5,
            },
            0x9A => Instruction {
                operation: Operation::TXS,
                address_mode: AddressingMode::Implied,
                cycles: 2,
            },
            0x9D => Instruction {
                operation: Operation::STA,
                address_mode: AddressingMode::AbsoluteX,
                cycles: 5,
            },
            // 0xA*
            0xA0 => Instruction {
                operation: Operation::LDY,
                address_mode: AddressingMode::Immediate,
                cycles: 2,
            },
            0xA1 => Instruction {
                operation: Operation::LDA,
                address_mode: AddressingMode::IndirectX,
                cycles: 6,
            },
            0xA2 => Instruction {
                operation: Operation::LDX,
                address_mode: AddressingMode::Immediate,
                cycles: 2,
            },
            0xA4 => Instruction {
                operation: Operation::LDY,
                address_mode: AddressingMode::ZeroPage,
                cycles: 3,
            },
            0xA5 => Instruction {
                operation: Operation::LDA,
                address_mode: AddressingMode::ZeroPage,
                cycles: 3,
            },
            0xA6 => Instruction {
                operation: Operation::LDX,
                address_mode: AddressingMode::ZeroPage,
                cycles: 3,
            },
            0xA8 => Instruction {
                operation: Operation::TAY,
                address_mode: AddressingMode::Implied,
                cycles: 2,
            },
            0xA9 => Instruction {
                operation: Operation::LDA,
                address_mode: AddressingMode::Immediate,
                cycles: 2,
            },
            0xAA => Instruction {
                operation: Operation::TAX,
                address_mode: AddressingMode::Implied,
                cycles: 2,
            },
            0xAC => Instruction {
                operation: Operation::LDY,
                address_mode: AddressingMode::Absolute,
                cycles: 4,
            },
            0xAD => Instruction {
                operation: Operation::LDA,
                address_mode: AddressingMode::Absolute,
                cycles: 4,
            },
            0xAE => Instruction {
                operation: Operation::LDX,
                address_mode: AddressingMode::Absolute,
                cycles: 4,
            },
            // 0xB*
            0xB0 => Instruction {
                operation: Operation::BCS,
                address_mode: AddressingMode::Relative,
                cycles: 2,
            },
            0xB1 => Instruction {
                operation: Operation::LDA,
                address_mode: AddressingMode::IndirectY,
                cycles: 5,
            },
            0xB4 => Instruction {
                operation: Operation::LDY,
                address_mode: AddressingMode::ZeroPageX,
                cycles: 4,
            },
            0xB5 => Instruction {
                operation: Operation::LDA,
                address_mode: AddressingMode::ZeroPageX,
                cycles: 4,
            },
            0xB6 => Instruction {
                operation: Operation::LDX,
                address_mode: AddressingMode::ZeroPageY,
                cycles: 4,
            },
            0xB8 => Instruction {
                operation: Operation::CLV,
                address_mode: AddressingMode::Implied,
                cycles: 2,
            },
            0xB9 => Instruction {
                operation: Operation::LDA,
                address_mode: AddressingMode::AbsoluteY,
                cycles: 4,
            },
            0xBA => Instruction {
                operation: Operation::TSX,
                address_mode: AddressingMode::Implied,
                cycles: 2,
            },
            0xBC => Instruction {
                operation: Operation::LDY,
                address_mode: AddressingMode::AbsoluteX,
                cycles: 4,
            },
            0xBD => Instruction {
                operation: Operation::LDA,
                address_mode: AddressingMode::AbsoluteX,
                cycles: 4,
            },
            0xBE => Instruction {
                operation: Operation::LDX,
                address_mode: AddressingMode::AbsoluteY,
                cycles: 4,
            },
            // 0xC*
            0xC0 => Instruction {
                operation: Operation::CPY,
                address_mode: AddressingMode::Immediate,
                cycles: 2,
            },
            0xC1 => Instruction {
                operation: Operation::CMP,
                address_mode: AddressingMode::IndirectX,
                cycles: 6,
            },
            0xC4 => Instruction {
                operation: Operation::CPY,
                address_mode: AddressingMode::ZeroPage,
                cycles: 3,
            },
            0xC5 => Instruction {
                operation: Operation::CMP,
                address_mode: AddressingMode::ZeroPage,
                cycles: 3,
            },
            0xC6 => Instruction {
                operation: Operation::DEC,
                address_mode: AddressingMode::ZeroPage,
                cycles: 5,
            },
            0xC8 => Instruction {
                operation: Operation::INY,
                address_mode: AddressingMode::Implied,
                cycles: 2,
            },
            0xC9 => Instruction {
                operation: Operation::CMP,
                address_mode: AddressingMode::Immediate,
                cycles: 2,
            },
            0xCA => Instruction {
                operation: Operation::DEX,
                address_mode: AddressingMode::Implied,
                cycles: 2,
            },
            0xCC => Instruction {
                operation: Operation::CPY,
                address_mode: AddressingMode::Absolute,
                cycles: 4,
            },
            0xCD => Instruction {
                operation: Operation::CMP,
                address_mode: AddressingMode::Absolute,
                cycles: 4,
            },
            0xCE => Instruction {
                operation: Operation::DEC,
                address_mode: AddressingMode::Absolute,
                cycles: 6,
            },
            // 0xD*
            0xD0 => Instruction {
                operation: Operation::BNE,
                address_mode: AddressingMode::Relative,
                cycles: 2,
            },
            0xD1 => Instruction {
                operation: Operation::CMP,
                address_mode: AddressingMode::IndirectY,
                cycles: 5,
            },
            0xD5 => Instruction {
                operation: Operation::CMP,
                address_mode: AddressingMode::ZeroPageX,
                cycles: 4,
            },
            0xD6 => Instruction {
                operation: Operation::DEC,
                address_mode: AddressingMode::ZeroPageX,
                cycles: 6,
            },
            0xD8 => Instruction {
                operation: Operation::CLD,
                address_mode: AddressingMode::Implied,
                cycles: 2,
            },
            0xD9 => Instruction {
                operation: Operation::CMP,
                address_mode: AddressingMode::AbsoluteY,
                cycles: 4,
            },
            0xDD => Instruction {
                operation: Operation::CMP,
                address_mode: AddressingMode::AbsoluteX,
                cycles: 4,
            },
            0xDE => Instruction {
                operation: Operation::DEC,
                address_mode: AddressingMode::AbsoluteX,
                cycles: 7,
            },
            // 0xE*
            0xE0 => Instruction {
                operation: Operation::CPX,
                address_mode: AddressingMode::Immediate,
                cycles: 2,
            },
            0xE1 => Instruction {
                operation: Operation::SBC,
                address_mode: AddressingMode::IndirectX,
                cycles: 6,
            },
            0xE4 => Instruction {
                operation: Operation::CPX,
                address_mode: AddressingMode::ZeroPage,
                cycles: 3,
            },
            0xE5 => Instruction {
                operation: Operation::SBC,
                address_mode: AddressingMode::ZeroPage,
                cycles: 3,
            },
            0xE6 => Instruction {
                operation: Operation::INC,
                address_mode: AddressingMode::ZeroPage,
                cycles: 5,
            },
            0xE8 => Instruction {
                operation: Operation::INX,
                address_mode: AddressingMode::Implied,
                cycles: 2,
            },
            0xE9 => Instruction {
                operation: Operation::SBC,
                address_mode: AddressingMode::Immediate,
                cycles: 2,
            },
            0xEA => Instruction {
                operation: Operation::NOP,
                address_mode: AddressingMode::Implied,
                cycles: 2,
            },
            0xEC => Instruction {
                operation: Operation::CPX,
                address_mode: AddressingMode::Absolute,
                cycles: 4,
            },
            0xED => Instruction {
                operation: Operation::SBC,
                address_mode: AddressingMode::Absolute,
                cycles: 4,
            },
            0xEE => Instruction {
                operation: Operation::INC,
                address_mode: AddressingMode::Absolute,
                cycles: 6,
            },
            // 0xF*
            0xF0 => Instruction {
                operation: Operation::BEQ,
                address_mode: AddressingMode::Relative,
                cycles: 2,
            },
            0xF1 => Instruction {
                operation: Operation::SBC,
                address_mode: AddressingMode::IndirectY,
                cycles: 5,
            },
            0xF5 => Instruction {
                operation: Operation::SBC,
                address_mode: AddressingMode::ZeroPageX,
                cycles: 4,
            },
            0xF6 => Instruction {
                operation: Operation::INC,
                address_mode: AddressingMode::ZeroPageX,
                cycles: 6,
            },
            0xF8 => Instruction {
                operation: Operation::SED,
                address_mode: AddressingMode::Implied,
                cycles: 2,
            },
            0xF9 => Instruction {
                operation: Operation::SBC,
                address_mode: AddressingMode::AbsoluteY,
                cycles: 4,
            },
            0xFD => Instruction {
                operation: Operation::SBC,
                address_mode: AddressingMode::AbsoluteX,
                cycles: 4,
            },
            0xFE => Instruction {
                operation: Operation::INC,
                address_mode: AddressingMode::AbsoluteX,
                cycles: 7,
            },
            _ => Instruction {
                operation: Operation::NOP,
                address_mode: AddressingMode::Implied,
                cycles: 0,
            },
        };
    }
}
