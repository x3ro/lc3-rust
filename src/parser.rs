use opcodes::Opcode;
use state::Registers;
use util;

use std::result::Result;

trait BitTools {
    fn has_bit(&self, index: u8) -> bool;
    fn to_register(&self, lowest_bit_index: u8) -> Registers;
    fn to_immediate(&self, num_bits: u8) -> u16;
}

impl BitTools for u16 {
    fn has_bit(&self, index: u8) -> bool {
        ((self >> index) & 1) > 0
    }

    fn to_register(&self, lowest_bit_index: u8) -> Registers {
        Registers::from_u16_or_panic((self >> lowest_bit_index) & 0b111)
    }

    fn to_immediate(&self, num_bits: u8) -> u16 {
        let imm = self & (1 << num_bits) - 1;
        util::sign_extend(imm, num_bits as u16)
    }
}

use Instruction::*;
#[derive(Debug)]
pub enum Instruction {
    Br { n: bool, z: bool, p: bool, pc_offset9: u16 },
    Jmp { base_r: Registers },
    AddImmediate { dr: Registers, sr1: Registers, imm5: u16 },
    AddRegister { dr: Registers, sr1: Registers, sr2: Registers },
    Ld { dr: Registers, offset9: u16 },
    Lea { dr: Registers, offset9: u16 },
    Trap { trapvect8: u16 },
}

impl Instruction {
    pub fn from_raw(raw: u16) -> Result<Self,String> {
        let opcode = Opcode::from_instruction(raw);

        match opcode {
            Opcode::BR => Ok(Self::from_br(raw)),
            Opcode::JMP => Ok(Self::from_jmp(raw)),
            Opcode::ADD => Ok(Self::from_add(raw)),
            Opcode::LEA => Ok(Self::from_lea(raw)),
            Opcode::LD => Ok(Self::from_ld(raw)),
            Opcode::TRAP => Ok(Self::from_trap(raw)),
            _ => Err(format!("Unrecognized opcode <0x{:x}>", opcode as u16))
        }
    }

    fn from_br(raw: u16) -> Self {
        let n = raw.has_bit(11);
        let z = raw.has_bit(10);
        let p = raw.has_bit(9);
        let pc_offset9 = raw.to_immediate(9);
        Br { n, z, p, pc_offset9 }
    }

    fn from_jmp(raw: u16) -> Self {
        let base_r = raw.to_register(6);
        Jmp { base_r }
    }

    fn from_add(raw: u16) -> Self {
        let dr = raw.to_register(9);
        let sr1 = raw.to_register(6);

        if raw.has_bit(5) {
            let imm5 = raw.to_immediate(5);
            AddImmediate { dr, sr1, imm5 }
        } else {
            let sr2 = raw.to_register(0);
            AddRegister { dr, sr1, sr2 }
        }
    }

    fn from_lea(raw: u16) -> Self {
        let dr = raw.to_register(9);
        let offset9 = raw.to_immediate(9);
        Lea { dr, offset9 }
    }

    fn from_ld(raw: u16) -> Self {
        let dr = raw.to_register(9);
        let offset9 = raw.to_immediate(9);
        Ld { dr, offset9 }
    }

    fn from_trap(raw: u16) -> Self {
        let trapvect8 = raw.to_immediate(8);
        Trap { trapvect8 }
    }
}