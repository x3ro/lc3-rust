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

                let (register, offset) = match operands.as_slice() {
                    [Operand::Register {r}, Operand::Label {name}] => (r, state.relative_offset(self.offset, name)),
                    _ => panic!("Unsupported {:?}", self.instruction)
                };

                vec![(opcode << 12) | ((register.to_owned() as u16) << 9) | offset]
            }

            Instruction { opcode: Opcode::Add, operands} => {
                const OPCODE:u16 = 0b0001;

                let mut result: u16 = 0b0000_0000_0000_0000;
                result |= OPCODE << 12;

                match operands.as_slice() {
                    [Operand::Register {r: dr}, Operand::Register {r: sr1}, Operand::Register {r: sr2}] => {
                        result |= (dr.to_owned() as u16) << 9;
                        result |= (sr1.to_owned() as u16) << 6;
                        result |= sr2.to_owned() as u16;
                    }

                    [Operand::Register {r: dr}, Operand::Register {r: sr1}, Operand::Immediate {value: imm5}] => {
                        if imm5 > &31 {
                            panic!("Immediate value too large, must fit into 5 bits");
                        }
                        result |= (dr.to_owned() as u16) << 9;
                        result |= (sr1.to_owned() as u16) << 6;
                        result |= 1 << 5;
                        result |= (imm5 & 0b11111) as u16;
                    }
                    _ => panic!("Unsupported {:?}", self.instruction)
                };

                vec![result]
            }

            Instruction { opcode: Opcode::Halt, operands} => {
                const OPCODE:u16 = 0b1111;

                if operands.len() > 0 {
                    panic!("HALT was used with operands, but does not take any")
                }

                let mut result: u16 = 0b0000_0000_0000_0000;
                result |= OPCODE << 12;
                result |= 0x25;

                vec![result]
            }

            Instruction { opcode: Opcode::Fill, operands} => {
                operands
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
