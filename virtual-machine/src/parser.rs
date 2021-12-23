// TODO: rename to decoder (in the fashion of FETCH -> DECODE -> ...)

use num_traits::FromPrimitive;

use std::fmt;

use crate::state::Registers;
use crate::util;

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

#[derive(FromPrimitive)]
pub enum Opcode {
    BR = 0x0,   /* branch */
    ADD = 0x1,  /* add  */
    LD = 0x2,   /* load */
    ST = 0x3,   /* store */
    JSR = 0x4,  /* jump register */
    AND = 0x5,  /* bitwise and */
    LDR = 0x6,  /* load register */
    STR = 0x7,  /* store register */
    RTI = 0x8,  /* unused */
    NOT = 0x9,  /* bitwise not */
    LDI = 0xA,  /* load indirect */
    STI = 0xB,  /* store indirect */
    JMP = 0xC,  /* jump */
    RES = 0xD,  /* reserved (unused) */
    LEA = 0xE,  /* load effective address */
    TRAP = 0xF, /* execute trap */
}

impl Opcode {
    pub fn from_instruction(instruction: u16) -> Self {
        // The upper three bits of an instruction are the opcode
        let opcode = instruction >> 12;
        match Opcode::from_u16(opcode) {
            Some(x) => x,
            None => panic!("Could not instantiate opcode from <0x{:X}>", opcode),
        }
    }
}

fn hex_fmt<T: fmt::Debug>(n: &T, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "0x{:02X?}", n)
}

use Instruction::*;
#[derive(custom_debug::Debug)]
pub enum Instruction {
    Br {
        n: bool,
        z: bool,
        p: bool,
        #[debug(with = "hex_fmt")]
        pc_offset9: u16,
    },
    Jmp {
        base_r: Registers,
    },
    JsrImmediate {
        #[debug(with = "hex_fmt")]
        pc_offset11: u16,
    },
    JsrRegister {
        base_r: Registers,
    },
    AddImmediate {
        dr: Registers,
        sr1: Registers,
        #[debug(with = "hex_fmt")]
        imm5: u16,
    },
    AddRegister {
        dr: Registers,
        sr1: Registers,
        sr2: Registers,
    },
    AndImmediate {
        dr: Registers,
        sr1: Registers,
        #[debug(with = "hex_fmt")]
        imm5: u16,
    },
    AndRegister {
        dr: Registers,
        sr1: Registers,
        sr2: Registers,
    },
    Ld {
        dr: Registers,
        #[debug(with = "hex_fmt")]
        offset9: u16,
    },
    Ldi {
        dr: Registers,
        #[debug(with = "hex_fmt")]
        offset9: u16,
    },
    Ldr {
        dr: Registers,
        base_r: Registers,
        #[debug(with = "hex_fmt")]
        offset6: u16,
    },
    Lea {
        dr: Registers,
        #[debug(with = "hex_fmt")]
        offset9: u16,
    },
    Not {
        dr: Registers,
        sr: Registers,
    },
    Rti {},
    St {
        sr: Registers,
        #[debug(with = "hex_fmt")]
        pc_offset9: u16,
    },
    Sti {
        sr: Registers,
        #[debug(with = "hex_fmt")]
        pc_offset9: u16,
    },
    Str {
        sr: Registers,
        base_r: Registers,
        #[debug(with = "hex_fmt")]
        offset6: u16,
    },
    Trap {
        #[debug(with = "hex_fmt")]
        trapvect8: u16,
    },
}

impl Instruction {
    pub fn from_raw(raw: u16) -> Self {
        let opcode = Opcode::from_instruction(raw);

        match opcode {
            Opcode::ADD => Self::from_add(raw),
            Opcode::AND => Self::from_and(raw),
            Opcode::BR => Self::from_br(raw),
            Opcode::JMP => Self::from_jmp(raw),
            Opcode::JSR => Self::from_jsr(raw),
            Opcode::LD => Self::from_ld(raw),
            Opcode::LDI => Self::from_ldi(raw),
            Opcode::LDR => Self::from_ldr(raw),
            Opcode::LEA => Self::from_lea(raw),
            Opcode::NOT => Self::from_not(raw),
            Opcode::RTI => Rti {},
            Opcode::ST => Self::from_st(raw),
            Opcode::STI => Self::from_sti(raw),
            Opcode::STR => Self::from_str(raw),
            Opcode::TRAP => Self::from_trap(raw),
            Opcode::RES => todo!(),
        }
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

    fn from_and(raw: u16) -> Self {
        let dr = raw.to_register(9);
        let sr1 = raw.to_register(6);

        if raw.has_bit(5) {
            let imm5 = raw.to_immediate(5);
            AndImmediate { dr, sr1, imm5 }
        } else {
            let sr2 = raw.to_register(0);
            AndRegister { dr, sr1, sr2 }
        }
    }

    fn from_br(raw: u16) -> Self {
        let n = raw.has_bit(11);
        let z = raw.has_bit(10);
        let p = raw.has_bit(9);
        let pc_offset9 = raw.to_immediate(9);
        Br {
            n,
            z,
            p,
            pc_offset9,
        }
    }

    fn from_jmp(raw: u16) -> Self {
        let base_r = raw.to_register(6);
        Jmp { base_r }
    }

    fn from_jsr(raw: u16) -> Self {
        if raw.has_bit(11) {
            let pc_offset11 = raw.to_immediate(11);
            JsrImmediate { pc_offset11 }
        } else {
            let base_r = raw.to_register(6);
            JsrRegister { base_r }
        }
    }

    fn from_ld(raw: u16) -> Self {
        let dr = raw.to_register(9);
        let offset9 = raw.to_immediate(9);
        Ld { dr, offset9 }
    }

    fn from_ldi(raw: u16) -> Self {
        let dr = raw.to_register(9);
        let offset9 = raw.to_immediate(9);
        Ldi { dr, offset9 }
    }

    fn from_ldr(raw: u16) -> Self {
        let dr = raw.to_register(9);
        let base_r = raw.to_register(6);
        let offset6 = raw.to_immediate(6);
        Ldr {
            dr,
            base_r,
            offset6,
        }
    }

    fn from_lea(raw: u16) -> Self {
        let dr = raw.to_register(9);
        let offset9 = raw.to_immediate(9);
        Lea { dr, offset9 }
    }

    fn from_not(raw: u16) -> Self {
        let dr = raw.to_register(9);
        let sr = raw.to_register(6);
        Not { dr, sr }
    }

    fn from_st(raw: u16) -> Self {
        let sr = raw.to_register(9);
        let pc_offset9 = raw.to_immediate(9);
        St { sr, pc_offset9 }
    }

    fn from_sti(raw: u16) -> Self {
        let sr = raw.to_register(9);
        let pc_offset9 = raw.to_immediate(9);
        Sti { sr, pc_offset9 }
    }

    fn from_str(raw: u16) -> Self {
        let sr = raw.to_register(9);
        let base_r = raw.to_register(6);
        let offset6 = raw.to_immediate(6);
        Str {
            sr,
            base_r,
            offset6,
        }
    }

    fn from_trap(raw: u16) -> Self {
        let trapvect8 = raw.to_immediate(8);
        Trap { trapvect8 }
    }
}
