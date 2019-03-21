use num_traits::FromPrimitive;

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

#[derive(Debug,PartialEq,Copy,Clone)]
pub enum Operand {
    Register { r: Registers },
    Immediate { value: i64 },
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
}

impl Opcode {
    // Shorthand for instantiating Instructions, e.g. Add.instruction(operands)
    pub fn instruction(self, operands: Vec<Operand>) -> Instruction {
        Instruction { opcode: self, operands }
    }
}

impl Opcode {
    pub fn from_string(s: String) -> Result<Self, String> {
        match s.as_ref() {
            "ADD" => Ok(Opcode::Add),
            _ => Err(format!("invalid opcode {}", s))
        }
    }
}



#[derive(Debug,PartialEq)]
pub struct Instruction {
    pub opcode: Opcode,
    pub operands: Vec<Operand>,
}