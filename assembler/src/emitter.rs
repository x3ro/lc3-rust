use std::collections::HashMap;
use std::io::Write;
use std::ops::Range;
use crate::{AstNode, Opcode, Register};

#[derive(Debug)]
pub enum Emittable {
    AddImmediate { dr: Register, sr: Register, imm5: ImmediateValue },
    AddRegister { dr: Register, sr1: Register, sr2: Register },
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
    pub fn from_i16(value: i16, bits: u8) -> Self {
        let range = Self::range(bits);
        if ! range.contains(&i32::from(value)) {
            panic!("wat {}", value)
        }

        ImmediateValue { value, bits }
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
    pub fn from(opcode: Opcode, mut operands: Vec<AstNode>) -> Self {
        use Emittable::*;

        match (opcode, operands.as_slice()) {
            (Opcode::Add, [
                AstNode::RegisterOperand(dr),
                AstNode::RegisterOperand(sr),
                AstNode::ImmediateOperand(imm),
            ]) => {
                Emittable::AddImmediate {
                    dr: *dr,
                    sr: *sr,
                    imm5: ImmediateValue::from_i16(*imm as i16, 5)
                }
            }

            (Opcode::Add, [
                AstNode::RegisterOperand(dr),
                AstNode::RegisterOperand(sr1),
                AstNode::RegisterOperand(sr2),
            ]) => {
                Emittable::AddRegister {
                    dr: *dr,
                    sr1: *sr1,
                    sr2: *sr2,
                }
            }

            (Opcode::Ld, [
                AstNode::RegisterOperand(dr),
                AstNode::Label(name)
            ]) => {
                Emittable::Ld {
                    dr: *dr,
                    source: Label { name: name.clone() }
                }
            }

            (Opcode::Halt, []) => {
                Emittable::Trap(0x25)
            },

            (Opcode::Fill, [AstNode::ImmediateOperand(value)]) => {
                Emittable::Fill(*value)
            },

            (Opcode::Stringz, [AstNode::StringLiteral(str)]) => {
                Emittable::Stringz(str.clone())
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
