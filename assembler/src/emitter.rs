use std::ops::Range;
use crate::{AstNode, Opcode, Register};

#[derive(Debug)]
pub enum Emittable {
    AddImmediate { dr: Register, sr1: Register, imm5: u16 },

    Fill(u16), // One specific value at the memory location
    Stringz(String), // A null-terminated string
    Zeroes(u16), // The given number of words as zeroes (reserved space)
}

#[derive(Debug)]
pub struct ImmediateValue {
    value: i16,
    bits: u8,
}

impl ImmediateValue {
    pub fn from_u16(value: i16, bits: u8) -> Self {
        let range = Self::range(bits);
        if ! range.contains(&i32::from(value)) {
            panic!("wat {}", value)
        }

        ImmediateValue { value, bits }
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
        use Opcode::*;
        use Emittable::*;

        match (opcode, operands.as_slice()) {
            (Add, [
                AstNode::RegisterOperand(dr),
                AstNode::RegisterOperand(sr),
                AstNode::ImmediateOperand(imm),
            ]) => {
                println!("{:?}", ImmediateValue::from_u16(*imm as i16, 5));


            }

            x => todo!("Opcode missing: {:?}", x),
        }

        // match opcode {
        //     Add => {
        //         let mut ops = operands.drain(0..3);
        //         let dr = ops.next();
        //         let sr = ops.next();
        //         let imm_or_label = ops.next();
        //
        //         println!("{:?} {:?} {:?}", dr, sr, imm_or_label);
        //     }
        //     x => todo!("Opcode missing: {:?}", x),
        // }

        Zeroes(0)
    }
}