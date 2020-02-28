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

#[derive(Debug,PartialEq,Clone)]
pub enum Operand {
    Register { r: Registers },
    Immediate { value: i64 },
    Label { name: String },
    String { value: String },
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
    Ld,
    And,
    Br { modifiers: Option<String> },

    // Traps with explicit names
    Halt,

    // Pseudo-opcodes
    Fill,
    Stringz,
}

impl Opcode {
    pub fn from(value: &String, modifiers: &Option<String>) -> Result<Self, String> {
        match value.to_lowercase().as_ref() {
            "add" => Ok(Opcode::Add),
            "ld" => Ok(Opcode::Ld),
            "and" => Ok(Opcode::And),
            "br" => Ok(Opcode::Br { modifiers: modifiers.clone() }),

            "halt" => Ok(Opcode::Halt),

            ".fill" => Ok(Opcode::Fill),
            ".stringz" => Ok(Opcode::Stringz),

            _ => Err(format!("Unknown opcode '{}'", value))
        }
    }
}

impl Opcode {
    // Shorthand for instantiating Instructions, e.g. Add.instruction(operands)
    pub fn instruction(self, operands: Vec<Operand>) -> Instruction {
        Instruction { opcode: self, operands }
    }
}

#[derive(Debug,PartialEq)]
pub struct Lc3File {
    pub origin: u16,
    pub lines: Vec<Line>,
}

#[derive(Debug,PartialEq)]
pub struct Instruction {
    pub opcode: Opcode,
    pub operands: Vec<Operand>,
}

#[derive(Debug,PartialEq)]
pub struct Line {
    pub label: Option<String>,
    pub instruction: Option<Instruction>,
    pub comment: Option<String>,
}