use tokens::*;
use std::fmt::Debug;
use ::{Lc3State, Offset};

pub fn lol(offset: u16, instruction: Instruction) -> Emittable {
    match instruction.opcode {
        Opcode::Ld => Emittable { offset, instruction },
        Opcode::Fill => Emittable { offset, instruction },
        _ => panic!("Unknown {:?}", instruction)
    }
}

#[derive(Debug)]
pub struct Emittable {
    offset: Offset,
    instruction: Instruction,
}

impl Emittable {
    pub fn size(&self) -> u16 {
        16
    }

    pub fn emit(&self, state: &Lc3State) -> Vec<u16> {
        vec![]
    }
}

//struct Emittable {
//
//}

//pub trait Emittable : Debug {
//    fn from(instruction:Instruction) -> Box<Self>;
//
//    fn size(&self) -> u16 {
//        16 // TODO: Not all emittables are 2 bytes
//    }
//
//    fn emit(&self, state: &Lc3State) -> Vec<u16>;
//}

//#[derive(Debug)]
//pub struct Load { offset: u16, instruction: Instruction }

//impl Emittable for Load {
//    fn from(instruction: Instruction) -> Box<Self> {
//        unimplemented!()
//    }
//
//    fn emit(&self, state: &Lc3State) -> Vec<u16> {
//        let opcode:u16 = 0b0010;
//
//        let (register, offset) = match self.instruction.operands.as_slice() {
//            [Operand::Register {r}, Operand::Label {name}] => (r, state.relative_offset(self.offset, name)),
//            _ => panic!("Unsupported {:?}", self.instruction)
//        };
//
//        vec![(opcode << 12) | ((register.to_owned() as u16) << 9) | offset]
//    }
//}
//
//#[derive(Debug)]
//pub struct Fill { offset: u16, instruction: Instruction }
//impl Emittable for Fill {
//    fn emit(&self, _: &Lc3State) -> Vec<u16> {
//        self.instruction.operands
//            .iter()
//            .map(|x| match x {
//                Operand::Immediate { value } => *value as u16,
//                _ => panic!("Only immediate operands are allowed for fill in {:?}", self.instruction)
//            })
//            .collect()
//
//    }
//}
