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
    And,
    Br { modifiers: Option<String> },
    Jmp,
    Jsr,
    Jsrr,
    Ld,
    Ldi,
    Ldr,
    Lea,
    Not,
    Ret,
    Rti,
    St,
    Sti,
    // Str,
    // Trap,

    // Traps with explicit names
    // Getc,
    // Out,
    // Puts,
    // In,
    // Putsp,
    Halt,

    // Pseudo-opcodes
    Fill,
    Stringz,
}

impl Opcode {
    pub fn from(value: &String, modifiers: &Option<String>) -> Result<Self, String> {
        match value.to_lowercase().as_ref() {
            "add" => Ok(Opcode::Add),
            "and" => Ok(Opcode::And),
            "br" => Ok(Opcode::Br { modifiers: modifiers.clone() }),
            "jmp" => Ok(Opcode::Jmp),
            "jsr" => Ok(Opcode::Jsr),
            "jsrr" => Ok(Opcode::Jsrr),
            "ld" => Ok(Opcode::Ld),
            "ldi" => Ok(Opcode::Ldi),
            "ldr" => Ok(Opcode::Ldr),
            "lea" => Ok(Opcode::Lea),
            "not" => Ok(Opcode::Not),
            "ret" => Ok(Opcode::Ret),
            "rti" => Ok(Opcode::Rti),
            "st" => Ok(Opcode::St),
            "sti" => Ok(Opcode::Sti),

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