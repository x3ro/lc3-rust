use tokens::*;
use std::fmt::Debug;
use ::{Lc3State, Offset};

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
        match &self.instruction {
            Instruction { opcode: Opcode::Ld, operands} => {
                let opcode:u16 = 0b0010;

                let (register, offset) = match self.instruction.operands.as_slice() {
                    [Operand::Register {r}, Operand::Label {name}] => (r, state.relative_offset(self.offset, name)),
                    _ => panic!("Unsupported {:?}", self.instruction)
                };

                vec![(opcode << 12) | ((register.to_owned() as u16) << 9) | offset]
            }

            Instruction { opcode: Opcode::Fill, operands} => {
                self.instruction.operands
                    .iter()
                    .map(|x| match x {
                        Operand::Immediate { value } => *value as u16,
                        _ => panic!("Only immediate operands are allowed for fill in {:?}", self.instruction)
                    })
                    .collect()
            }
            _ => panic!("Can't emit unknown instruction {:?}", self.instruction)
        }
    }

    pub fn from(instruction: Instruction, offset: Offset) -> Self {
        Emittable { offset, instruction }
    }
}
