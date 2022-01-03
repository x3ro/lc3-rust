use std::collections::HashMap;
use std::io::Write;
use std::ops::Range;
use anyhow::{bail, Context, Result};
use crate::{AstNode, Modifiers, Opcode, Register};

#[derive(Debug)]
pub enum Emittable {
    AddImmediate { dr: Register, sr: Register, imm5: ImmediateValue },
    AddRegister { dr: Register, sr1: Register, sr2: Register },

    AndRegister { dr: Register, sr1: Register, sr2: Register},
    AndImmediate { dr: Register, sr: Register, imm5: ImmediateValue },

    Br { modifiers: Modifiers, target: Label },

    Jmp(Register),
    Jsr(Label),
    Jsrr(Register),

    Ld { dr: Register, source: Label },
    Ldi { dr: Register, source: Label },
    Ldr { dr: Register, base_r: Register, imm6: ImmediateValue },
    Lea { dr: Register, source: Label },

    Not { dr: Register, sr: Register },

    St { dr: Register, source: Label },
    Sti { dr: Register, source: Label },
    Str { dr: Register, base_r: Register, imm6: ImmediateValue },

    Trap(u16),
    Rti,

    FillImmediate(u16), // One specific value at the memory location
    FillLabel(Label), // The address of the given label
    Stringz(String), // A null-terminated string
    Zeroes(u16), // The given number of words as zeroes (reserved space)
}

#[derive(Debug)]
pub struct Label {
    name: String,
}

impl Label {
    pub fn relative_offset(&self, bits: u8, offset: u16, labels: &HashMap<String, u16>) -> Result<ImmediateValue> {
        let label_location = self.address(labels)?;
        let relative = ((label_location as i32) - (offset as i32 + 1)) as i16;
        Ok(ImmediateValue { value: relative, bits })
    }

    pub fn address(&self, labels: &HashMap<String, u16>) -> Result<u16> {
        labels.get(&self.name)
            .map(|v| *v)
            .context(format!("Use of undefined label '{}'", &self.name))
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

            (Opcode::Jmp, [
                AstNode::RegisterOperand(dr),
            ]) => {
                Ok(Emittable::Jmp(*dr))
            }

            (Opcode::Jsr, [
                AstNode::Label(name),
            ]) => {
                Ok(Emittable::Jsr(Label { name: name.clone() }))
            }

            (Opcode::Jsrr, [
                AstNode::RegisterOperand(dr),
            ]) => {
                Ok(Emittable::Jsrr(*dr))
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

            (Opcode::Ldi, [
                AstNode::RegisterOperand(dr),
                AstNode::Label(name)
            ]) => {
                Ok(Emittable::Ldi {
                    dr: *dr,
                    source: Label { name: name.clone() }
                })
            }

            (Opcode::Ldr, [
                AstNode::RegisterOperand(dr),
                AstNode::RegisterOperand(base_r),
                AstNode::ImmediateOperand(imm)
            ]) => {
                Ok(Emittable::Ldr {
                    dr: *dr,
                    base_r: *base_r,
                    imm6: ImmediateValue::from_i16(*imm as i16, 6)?
                })
            }

            (Opcode::Lea, [
                AstNode::RegisterOperand(dr),
                AstNode::Label(name)
            ]) => {
                Ok(Emittable::Lea {
                    dr: *dr,
                    source: Label { name: name.clone() }
                })
            }

            (Opcode::Nop, []) => {
                // Nop (No-op) is just an empty word
                Ok(Emittable::FillImmediate(0))
            }

            (Opcode::Not, [
                AstNode::RegisterOperand(dr),
                AstNode::RegisterOperand(sr),
            ]) => {
                Ok(Emittable::Not {
                    dr: *dr,
                    sr: *sr,
                })
            }

            (Opcode::St, [
                AstNode::RegisterOperand(dr),
                AstNode::Label(name)
            ]) => {
                Ok(Emittable::St {
                    dr: *dr,
                    source: Label { name: name.clone() },
                })
            },

            (Opcode::Sti, [
                AstNode::RegisterOperand(dr),
                AstNode::Label(name)
            ]) => {
                Ok(Emittable::Sti {
                    dr: *dr,
                    source: Label { name: name.clone() },
                })
            },

            (Opcode::Str, [
                AstNode::RegisterOperand(dr),
                AstNode::RegisterOperand(base_r ),
                AstNode::ImmediateOperand(imm)
            ]) => {
                Ok(Emittable::Str {
                    dr: *dr,
                    base_r: *base_r,
                    imm6: ImmediateValue::from_i16(*imm as i16, 6)?
                })
            },

            (Opcode::Ret, []) => {
                // RET is just an alias for `JMP R7`
                Ok(Emittable::Jmp(Register::R7))
            }

            (Opcode::Rti, []) => {
                Ok(Emittable::Rti)
            }

            (Opcode::Trap, [
                AstNode::ImmediateOperand(imm)
            ]) => {
                Ok(Emittable::Trap(*imm))
            },

            (Opcode::Getc, []) => {
                Ok(Emittable::Trap(0x20))
            },

            (Opcode::Out, []) => {
                Ok(Emittable::Trap(0x21))
            },

            (Opcode::Puts, []) => {
                Ok(Emittable::Trap(0x22))
            },

            (Opcode::In, []) => {
                Ok(Emittable::Trap(0x23))
            },

            (Opcode::Putsp, []) => {
                Ok(Emittable::Trap(0x24))
            },

            (Opcode::Halt, []) => {
                Ok(Emittable::Trap(0x25))
            },

            (Opcode::Fill, [
                AstNode::ImmediateOperand(value)
            ]) => {
                Ok(Emittable::FillImmediate(*value))
            },

            (Opcode::Fill, [
                AstNode::Label(name)
            ]) => {
                //let amount = u16::from_str_radix(name, 10).ok();
                if let Some(value) = u16::from_str_radix(name, 10).ok() {
                    Ok(Emittable::FillImmediate(value))
                } else {
                    Ok(Emittable::FillLabel(Label { name: name.clone() }))
                }
            },

            (Opcode::Stringz, [
                AstNode::StringLiteral(str)
            ]) => {
                let str = str.clone()
                    .replace("\\n", "\n")
                    .replace("\\t", "\t");
                Ok(Emittable::Stringz(str))
            }

            (Opcode::Blkw, [
                // While BLKW takes a number as input, it does so in a different way than
                // other immediate arguments, without a prefix ('#' or 'x'). The grammar thus recog-
                // nizes the operand as a label.
                AstNode::Label(name)
            ]) => {
                let amount = u16::from_str_radix(name, 10)?;
                Ok(Emittable::Zeroes(amount))
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

    pub fn emit(&self, offset: u16, labels: &HashMap<String, u16>) -> Result<Vec<u16>> {
        use Emittable::*;

        match self {
            AddImmediate { dr, sr, imm5 } => {
                const OPCODE: u16 = 0b0001;
                let mut result: u16 = OPCODE << 12;
                result |= (*dr as u16) << 9;
                result |= (*sr as u16) << 6;
                result |= 1 << 5;
                result |= imm5.as_u16();
                Ok(vec![result])
            }

            AddRegister { dr, sr1, sr2 } => {
                const OPCODE: u16 = 0b0001;
                let mut result: u16 = OPCODE << 12;
                result |= (*dr as u16) << 9;
                result |= (*sr1 as u16) << 6;
                result |= (*sr2 as u16);
                Ok(vec![result])
            }

            AndRegister { dr, sr1, sr2 } => {
                const OPCODE: u16 = 0b0101;
                let mut result: u16 = OPCODE << 12;
                result |= (*dr as u16) << 9;
                result |= (*sr1 as u16) << 6;
                result |= (*sr2 as u16);
                Ok(vec![result])
            }

            AndImmediate { dr, sr, imm5 } => {
                const OPCODE: u16 = 0b0101;
                let mut result: u16 = OPCODE << 12;
                result |= (*dr as u16) << 9;
                result |= (*sr as u16) << 6;
                result |= 1 << 5;
                result |= imm5.as_u16();
                Ok(vec![result])
            }

            Br { modifiers, target } => {
                const OPCODE: u16 = 0b0000;
                let mut result: u16 = OPCODE << 12;
                result |= (modifiers.negative as u16) << 11;
                result |= (modifiers.zero as u16) << 10;
                result |= (modifiers.positive as u16) << 9;
                result |= target.relative_offset(9, offset, labels)?.as_u16();
                Ok(vec![result])
            }

            Jmp(dr) => {
                const OPCODE: u16 = 0b1100;
                let mut result: u16 = OPCODE << 12;
                result |= (dr.to_owned() as u16) << 6;
                Ok(vec![result])
            }

            Jsr(target) => {
                const OPCODE: u16 = 0b0100;
                let mut result: u16 = OPCODE << 12;
                result |= 1 << 11;
                result |= target.relative_offset(11, offset, labels)?.as_u16();
                Ok(vec![result])
            }

            Jsrr(dr) => {
                const OPCODE: u16 = 0b0100;
                let mut result: u16 = OPCODE << 12;
                result |= (dr.to_owned() as u16) << 6;
                Ok(vec![result])
            }

            Ld { dr, source } => {
                const OPCODE: u16 = 0b0010;
                let mut result: u16 = OPCODE << 12;
                result |= (dr.to_owned() as u16) << 9;
                result |= source.relative_offset(9, offset, labels)?.as_u16();
                Ok(vec![result])
            }

            Ldi { dr, source } => {
                const OPCODE: u16 = 0b1010;
                let mut result: u16 = OPCODE << 12;
                result |= (dr.to_owned() as u16) << 9;
                result |= source.relative_offset(9, offset, labels)?.as_u16();
                Ok(vec![result])
            }

            Ldr { dr, base_r, imm6 } => {
                const OPCODE: u16 = 0b0110;
                let mut result: u16 = OPCODE << 12;
                result |= (dr.to_owned() as u16) << 9;
                result |= (base_r.to_owned() as u16) << 6;
                result |= imm6.as_u16();
                Ok(vec![result])
            }

            Lea { dr, source } => {
                const OPCODE: u16 = 0b1110;
                let mut result: u16 = OPCODE << 12;
                result |= (dr.to_owned() as u16) << 9;
                result |= source.relative_offset(9, offset, labels)?.as_u16();
                Ok(vec![result])
            }

            Not { dr, sr } => {
                const OPCODE: u16 = 0b1001;
                let mut result: u16 = OPCODE << 12;
                result |= (dr.to_owned() as u16) << 9;
                result |= (sr.to_owned() as u16) << 6;
                result |= 0b111111;
                Ok(vec![result])
            }

            St { dr, source } => {
                const OPCODE: u16 = 0b0011;
                let mut result: u16 = OPCODE << 12;
                result |= (dr.to_owned() as u16) << 9;
                result |= source.relative_offset(9, offset, labels)?.as_u16();
                Ok(vec![result])
            }

            Sti { dr, source } => {
                const OPCODE: u16 = 0b1011;
                let mut result: u16 = OPCODE << 12;
                result |= (dr.to_owned() as u16) << 9;
                result |= source.relative_offset(9, offset, labels)?.as_u16();
                Ok(vec![result])
            }

            Str { dr, base_r, imm6 } => {
                const OPCODE: u16 = 0b0111;
                let mut result: u16 = OPCODE << 12;
                result |= (dr.to_owned() as u16) << 9;
                result |= (base_r.to_owned() as u16) << 6;
                result |= imm6.as_u16();
                Ok(vec![result])
            }

            Rti => {
                Ok(vec![0b1000_0000_0000_0000])
            },

            Trap(addr) => {
                let mut result: u16 = 0b1111_0000_0000_0000;
                result |= addr;
                Ok(vec![result])
            }

            FillImmediate(value) => {
                Ok(vec![*value])
            }

            FillLabel(label) => {
                let addr = label.address(labels)?;
                Ok(vec![addr])
            }

            Stringz(str) => {
                let mut data: Vec<_> = str.chars().map(|c| c as u16).collect();
                data.push(0);
                Ok(data)
            }

            Zeroes(amount) => {
                let data: Vec<u16> = vec![0; usize::from(*amount)];
                Ok(data)
            }

            x => todo!("missing: {:?}", x),
        }
    }
}
