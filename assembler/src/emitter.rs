use std::collections::HashMap;
use std::io::Write;
use std::ops::Range;
use anyhow::bail;
use crate::{AstNode, Modifiers, Opcode, Register};

#[derive(Debug)]
pub enum Emittable {
    AddImmediate { dr: Register, sr: Register, imm5: ImmediateValue },
    AddRegister { dr: Register, sr1: Register, sr2: Register },

    AndRegister { dr: Register, sr1: Register, sr2: Register},
    AndImmediate { dr: Register, sr: Register, imm5: ImmediateValue },

    Br { modifiers: Modifiers, target: Label },

    Ld { dr: Register, source: Label },
    Trap(u16),

    Fill(u16), // One specific value at the memory location
    Stringz(String), // A null-terminated string
    Zeroes(u16), // The given number of words as zeroes (reserved space)
}

#[derive(Debug)]
pub struct Label {
    name: String,
}

impl Label {
    pub fn relative_offset(&self, bits: u8, offset: u16, labels: &HashMap<String, u16>) -> ImmediateValue {
        let label_location = *labels.get(&self.name).unwrap();
        let relative = ((label_location as i32) - (offset as i32 + 1)) as i16;
        ImmediateValue { value: relative, bits }
    }
}

#[derive(Debug)]
pub struct ImmediateValue {
    value: i16,
    bits: u8,
}

impl ImmediateValue {
    pub fn from_i16(value: i16, bits: u8) -> anyhow::Result<Self> {
        let range = Self::range(bits);
        if ! range.contains(&i32::from(value)) {
            bail!("Immediate value {} is too large. {} bits are available, for a range of {}..{}.", value, bits, range.start, range.end - 1);
        }

        Ok(ImmediateValue { value, bits })
    }

    fn as_u16(&self) -> u16 {
        let mask = (1 << self.bits) - 1;
        self.value as u16 & mask
    }

    fn range(bits: u8) -> Range<i32> {
        // An n-bit two's-complement number has one bit reserved for the
        // sign, which we have to take into account when we calculate the range
        let bits = bits - 1;

        let lower = -1 * 1 << bits;
        let upper = -1 * lower;

        lower..upper
    }
}

impl Emittable {
    pub fn from(opcode: Opcode, mut operands: Vec<AstNode>) -> anyhow::Result<Self> {
        match (opcode, operands.as_slice()) {
            (Opcode::Add, [
                AstNode::RegisterOperand(dr),
                AstNode::RegisterOperand(sr),
                AstNode::ImmediateOperand(imm),
            ]) => {
                Ok(Emittable::AddImmediate {
                    dr: *dr,
                    sr: *sr,
                    imm5: ImmediateValue::from_i16(*imm as i16, 5)?
                })
            }

            (Opcode::Add, [
                AstNode::RegisterOperand(dr),
                AstNode::RegisterOperand(sr1),
                AstNode::RegisterOperand(sr2),
            ]) => {
                Ok(Emittable::AddRegister {
                    dr: *dr,
                    sr1: *sr1,
                    sr2: *sr2,
                })
            }

            (Opcode::And, [
                AstNode::RegisterOperand(dr),
                AstNode::RegisterOperand(sr1),
                AstNode::RegisterOperand(sr2),
            ]) => {
                Ok(Emittable::AndRegister {
                    dr: *dr,
                    sr1: *sr1,
                    sr2: *sr2,
                })
            }

            (Opcode::And, [
                AstNode::RegisterOperand(dr),
                AstNode::RegisterOperand(sr),
                AstNode::ImmediateOperand(imm),
            ]) => {
                Ok(Emittable::AndImmediate {
                    dr: *dr,
                    sr: *sr,
                    imm5: ImmediateValue::from_i16(*imm as i16, 5)?
                })
            }

            (Opcode::Br{ modifiers }, [
                AstNode::Label(name)
            ]) => {
                Ok(Emittable::Br {
                    modifiers,
                    target: Label { name: name.clone() }
                })
            }

            (Opcode::Ld, [
                AstNode::RegisterOperand(dr),
                AstNode::Label(name)
            ]) => {
                Ok(Emittable::Ld {
                    dr: *dr,
                    source: Label { name: name.clone() }
                })
            }

            (Opcode::Halt, []) => {
                Ok(Emittable::Trap(0x25))
            },

            (Opcode::Fill, [
                AstNode::ImmediateOperand(value)
            ]) => {
                Ok(Emittable::Fill(*value))
            },

            (Opcode::Stringz, [
                AstNode::StringLiteral(str)
            ]) => {
                Ok(Emittable::Stringz(str.clone()))
            }

            x => todo!("Opcode missing: {:?}", x),
        }
    }

    pub fn size(&self) -> usize {
        use Emittable::*;

        match self {
            Stringz(str) => str.len() + 1,
            Zeroes(len) => *len as usize,
            _ => 1,
        }
    }

    pub fn emit(&self, offset: u16, labels: &HashMap<String, u16>) -> Vec<u16> {
        use Emittable::*;

        match self {
            AddImmediate { dr, sr, imm5 } => {
                const OPCODE: u16 = 0b0001;
                let mut result: u16 = OPCODE << 12;
                result |= (*dr as u16) << 9;
                result |= (*sr as u16) << 6;
                result |= 1 << 5;
                result |= imm5.as_u16();
                vec![result]
            },

            AddRegister { dr, sr1, sr2 } => {
                const OPCODE: u16 = 0b0001;
                let mut result: u16 = OPCODE << 12;
                result |= (*dr as u16) << 9;
                result |= (*sr1 as u16) << 6;
                result |= (*sr2 as u16);
                vec![result]
            },

            AndRegister { dr, sr1, sr2 } => {
                const OPCODE: u16 = 0b0101;
                let mut result: u16 = OPCODE << 12;
                result |= (*dr as u16) << 9;
                result |= (*sr1 as u16) << 6;
                result |= (*sr2 as u16);
                vec![result]
            },

            AndImmediate { dr, sr, imm5 } => {
                const OPCODE: u16 = 0b0101;
                let mut result: u16 = OPCODE << 12;
                result |= (*dr as u16) << 9;
                result |= (*sr as u16) << 6;
                result |= 1 << 5;
                result |= imm5.as_u16();
                vec![result]
            },

            Ld { dr, source } => {
                const OPCODE: u16 = 0b0010;
                let mut result: u16 = OPCODE << 12;
                result |= (dr.to_owned() as u16) << 9;
                result |= source.relative_offset(9, offset, labels).as_u16();
                vec![result]
            }

            Trap(addr) => {
                let mut result: u16 = 0b1111_0000_0000_0000;
                result |= addr;
                vec![result]
            },

            Fill(value) => {
                vec![*value]
            }

            Stringz(str) => {
                let mut data: Vec<_> = str.chars().map(|c| c as u16).collect();
                data.push(0);
                data
            }

            x => todo!("missing: {:?}", x),
        }
    }
}
