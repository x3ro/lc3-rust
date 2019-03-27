use num_traits::FromPrimitive;
use std::convert::TryFrom;

#[derive(Debug,PartialEq,Copy,Clone,num_derive::FromPrimitive)]
pub enum Registers {
    R0 = 0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
}

#[derive(Debug,PartialEq,Clone)]
pub enum Operand {
    Register { r: Registers },
    Immediate { value: i64 },
    Label { name: String },
}

impl Operand {
    pub fn register(index: u16) -> Self {
        Operand::Register { r: Registers::from_u16(index).unwrap() }
    }

    pub fn immediate(value: i64) -> Self {
        Operand::Immediate { value }
    }
}

#[derive(Debug,PartialEq)]
pub enum Opcode {
    Add,
    Halt,
    Ld,

    // Pseudo-opcodes
    Fill
}

impl TryFrom<&String> for Opcode {
    type Error = String;
    fn try_from(value: &String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_ref() {
            "add" => Ok(Opcode::Add),
            "halt" => Ok(Opcode::Halt),
            "ld" => Ok(Opcode::Ld),
            ".fill" => Ok(Opcode::Fill),
            x => Err(format!("Unknown opcode '{}'", x))
        }
    }
}

//#[derive(Debug,PartialEq)]
//pub enum PseudoOpcode {
//    Fill,
//}
//
//impl TryFrom<&String> for PseudoOpcode {
//    type Error = String;
//    fn try_from(value: &String) -> Result<Self, Self::Error> {
//        match value.to_lowercase().as_ref() {
//            ".fill" => Ok(PseudoOpcode::Fill),
//            x => Err(format!("Unknown pseudo operation '{}'", x))
//        }
//    }
//}


impl Opcode {
    // Shorthand for instantiating Instructions, e.g. Add.instruction(operands)
    pub fn instruction(self, operands: Vec<Operand>) -> Instruction {
        Instruction { opcode: self, operands }
    }
}

#[derive(Debug,PartialEq)]
pub struct Lc3File {
    pub origin: i64,
    pub lines: Vec<Line>,
}

#[derive(Debug,PartialEq)]
pub struct Instruction {
    pub opcode: Opcode,
    pub operands: Vec<Operand>,
}

//pub enum Instruction1 {
//    Operation { opcode: Opcode, operands: Vec<Operand> },
//    Pseudo { asd: PseudoOpcode, operands: Vec<Operand> }
//}

#[derive(Debug,PartialEq)]
pub struct Line {
    pub label: Option<String>,
    pub instruction: Option<Instruction>,
    pub comment: Option<String>,
}