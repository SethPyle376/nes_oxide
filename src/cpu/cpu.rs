use bitflags::bitflags;
use num_traits::AsPrimitive;

use super::instructions::{AddressingMode, Operation};
use super::Instruction;
use super::{Bus, Controller};

bitflags! {
    #[derive(Default, Debug, Copy, Clone)]
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
    pub pc: u16,                   // Program Counter
    pub sp: u8,                    // Stack Pointer
    pub r_a: u8,                   // Accumulator
    pub r_x: u8,                   // X Register
    pub r_y: u8,                   // Y Register
    pub status: CpuStatusRegister, // Status Register
    pub bus: Bus,
    pub controller: Controller,
}

impl Cpu {
    pub fn new(mut bus: Bus) -> Self {
        Self {
            cycle: 7,
            pc: bus.read_u16(0xFFFC),
            sp: 0xFD,
            r_a: 0,
            r_x: 0,
            r_y: 0,
            status: (CpuStatusRegister::empty() | CpuStatusRegister::U | CpuStatusRegister::I),
            bus,
            controller: Controller::default(),
        }
    }

    pub fn step<F>(&mut self, mut inject: F)
    where
        F: FnMut(&mut Cpu),
    {
        inject(self);

        if !self.controller.pause {
            let opcode = self.bus.read(self.pc);
            let instruction = Instruction::from_u8(opcode);
            let cycles = self.execute_instruction(&instruction);
            self.cycle = self.cycle + cycles as u64;
        }

        if self.controller.step_mode {
            self.controller.pause = true;
        }
    }

    pub fn page_cross(base: u16, absolute: u16) -> bool {
        return (base & 0xFF00) != (absolute & 0xFF00);
    }

    pub fn push(&mut self, data: u8) {
        self.bus.write(0x100 | u16::from(self.sp), data);
        self.sp = self.sp.wrapping_sub(1);
    }

    pub fn push_u16(&mut self, data: u16) {
        let [lo, hi] = u16::to_le_bytes(data);
        self.push(hi);
        self.push(lo);
    }

    pub fn pop(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        self.bus.read(0x100 | u16::from(self.sp))
    }

    pub fn pop_u16(&mut self) -> u16 {
        let lo = self.pop();
        let hi = self.pop();
        u16::from_le_bytes([lo, hi])
    }

    pub fn set_zn(&mut self, value: u8) {
        self.status.set(CpuStatusRegister::Z, value == 0);
        self.status.set(CpuStatusRegister::N, value & 0x80 == 0x80);
    }

    pub fn write_fetched(
        &mut self,
        address_mode: &AddressingMode,
        address: Option<u16>,
        value: u8,
    ) {
        match address_mode {
            AddressingMode::Implied | AddressingMode::Accumulator => self.r_a = value,
            AddressingMode::Immediate => (),
            _ => self.bus.write(address.unwrap(), value),
        }
    }

    pub fn branch(&mut self, relative_address: u16) -> u8 {
        let absolute_address = if relative_address & 0x80 == 0x80 {
            self.pc.wrapping_add(relative_address | 0xFF00)
        } else {
            self.pc.wrapping_add(relative_address)
        };

        self.pc = absolute_address;

        if Self::page_cross(self.pc, absolute_address) {
            return 2;
        }

        return 1;
    }

    // Output instruction trace string and next instruction address
    pub fn trace_instruction(&mut self, addr: u16) -> (String, u16) {
        let opcode = self.bus.read(addr);
        let instruction = Instruction::from_u8(opcode);

        let mut instruction_bytes = Vec::with_capacity(3);
        instruction_bytes.push(opcode);

        let mode = match instruction.address_mode {
            AddressingMode::Immediate => {
                instruction_bytes.push(self.bus.read(addr.wrapping_add(1)));
                format!(" #${:02X}", instruction_bytes[1])
            }
            AddressingMode::ZeroPage => {
                instruction_bytes.push(self.bus.read(addr.wrapping_add(1)));
                let value = self.bus.read(instruction_bytes[1].into());
                format!(" ${:02X} = {value:02X}", instruction_bytes[1])
            }
            AddressingMode::ZeroPageX => {
                instruction_bytes.push(self.bus.read(addr.wrapping_add(1)));
                let offset = instruction_bytes[1].wrapping_add(self.r_x);
                let value = self.bus.read(offset.into());
                format!(
                    " ${:02X},X @ {offset:02X} = {value:02X}",
                    instruction_bytes[1]
                )
            }
            AddressingMode::ZeroPageY => {
                instruction_bytes.push(self.bus.read(addr.wrapping_add(1)));
                let offset = instruction_bytes[1].wrapping_add(self.r_y);
                let value = self.bus.read(offset.into());
                format!(
                    " ${:02X},Y @ {offset:02X} = {value:02X}",
                    instruction_bytes[1]
                )
            }
            AddressingMode::Absolute => {
                instruction_bytes.push(self.bus.read(addr.wrapping_add(1)));
                instruction_bytes.push(self.bus.read(addr.wrapping_add(2)));
                let address = self.bus.read_u16(addr.wrapping_add(1));

                if instruction.operation == Operation::JMP
                    || instruction.operation == Operation::JSR
                {
                    format!(" ${address:04X}")
                } else {
                    let value = self.bus.read(address);
                    format!(" ${address:04X} = {value:02X}")
                }
            }
            AddressingMode::AbsoluteX => {
                instruction_bytes.push(self.bus.read(addr.wrapping_add(1)));
                instruction_bytes.push(self.bus.read(addr.wrapping_add(2)));
                let address = self.bus.read_u16(addr.wrapping_add(1));
                let offset = address.wrapping_add(self.r_x.into());
                let value = self.bus.read(offset);
                format!(" ${address:04X},X @ {offset:04X} = {value:02X}")
            }
            AddressingMode::AbsoluteY => {
                instruction_bytes.push(self.bus.read(addr.wrapping_add(1)));
                instruction_bytes.push(self.bus.read(addr.wrapping_add(2)));
                let address = self.bus.read_u16(addr.wrapping_add(1));
                let offset = address.wrapping_add(self.r_y.into());
                let value = self.bus.read(offset);
                format!(" ${address:04X},Y @ {offset:04X} = {value:02X}")
            }
            AddressingMode::Indirect => {
                instruction_bytes.push(self.bus.read(addr.wrapping_add(1)));
                instruction_bytes.push(self.bus.read(addr.wrapping_add(2)));
                let address = self.bus.read_u16(addr.wrapping_add(1));

                let lo = self.bus.read(address);
                let hi = if address & 0xFF == 0xFF {
                    self.bus.read(address & 0xFF00)
                } else {
                    self.bus.read(address + 1)
                };

                let value = u16::from_le_bytes([lo, hi]);
                format!(" (${address:04X}) = {value:04X}")
            }
            AddressingMode::IndirectX => {
                instruction_bytes.push(self.bus.read(addr.wrapping_add(1)));
                let offset = instruction_bytes[1].wrapping_add(self.r_x);
                let address = self.bus.read_u16_zp(offset);
                let value = self.bus.read(address);
                format!(
                    " (${:02X},X) @ {offset:02X} = {address:04X} = {value:02X}",
                    instruction_bytes[1]
                )
            }
            AddressingMode::IndirectY => {
                instruction_bytes.push(self.bus.read(addr.wrapping_add(1)));
                let address = self.bus.read_u16_zp(instruction_bytes[1]);
                let offset = address.wrapping_add(self.r_y.into());
                let value = self.bus.read(offset);
                format!(
                    " (${:02X}),Y = {address:04X} @ {offset:04X} = {value:02X}",
                    instruction_bytes[1]
                )
            }
            AddressingMode::Relative => {
                instruction_bytes.push(self.bus.read(addr.wrapping_add(1)));
                let mut address = self.bus.read(addr.wrapping_add(1)).into();
                if address & 0x80 == 0x80 {
                    address |= 0xFF00;
                }
                format!(" ${:04X}", addr.wrapping_add(2).wrapping_add(address))
            }
            AddressingMode::Accumulator => " A".to_string(),
            AddressingMode::Implied => "".to_string(),
        };

        let byte_str = instruction_bytes
            .iter()
            .map(|byte| format!("{:02x}", byte))
            .collect::<Vec<String>>()
            .join(" ");
        let instruction_string = format!(
            "{:04x}  {:8}  {:?}{}",
            addr, byte_str, instruction.operation, mode
        )
        .to_ascii_uppercase();

        return (
            instruction_string,
            addr.wrapping_add(instruction_bytes.len().as_()),
        );
    }

    pub fn trace(&mut self) -> (String, u16) {
        let instruction = self.trace_instruction(self.pc);
        let trace_string = format!(
            "{:47} A:{:02x} X:{:02x} Y:{:02x} P:{:02x} SP:{:02x} CYC:{}\n",
            instruction.0,
            self.r_a,
            self.r_x,
            self.r_y,
            self.status.bits(),
            self.sp,
            self.cycle
        )
        .to_ascii_uppercase();

        return (trace_string, instruction.1);
    }
}
