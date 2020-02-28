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
        // TODO: not all emittables are 16 bits (e.g. stringz or fill)
        16
    }

    pub fn emit(&self, state: &Lc3State) -> Result<Vec<u16>, String> {
        match &self.instruction {
            Instruction { opcode: Opcode::Ld, operands} => {
                const OPCODE:u16 = 0b0010;
                let mut result: u16 = OPCODE << 12;

                match operands.as_slice() {
                    [Operand::Register {r: dr}, Operand::Label {name}] => {
                        result |= (dr.to_owned() as u16) << 9;
                        result |= state.relative_offset(self.offset, name);
                    },

                    _ => return Err(format!("Unsupported operands for LD: {:?}", self.instruction))
                };

                Ok(vec![result])
            }

            Instruction { opcode: Opcode::Add, operands} => {
                const OPCODE:u16 = 0b0001;
                let mut result: u16 = OPCODE << 12;

                match operands.as_slice() {
                    [Operand::Register {r: dr}, Operand::Register {r: sr1}, Operand::Register {r: sr2}] => {
                        result |= (dr.to_owned() as u16) << 9;
                        result |= (sr1.to_owned() as u16) << 6;
                        result |= sr2.to_owned() as u16;
                    }

                    [Operand::Register {r: dr}, Operand::Register {r: sr1}, Operand::Immediate {value: imm5}] => {
                        if imm5 > &31 {
                            return Err("Immediate value too large, must fit into 5 bits".into());
                        }
                        result |= (dr.to_owned() as u16) << 9;
                        result |= (sr1.to_owned() as u16) << 6;
                        result |= 1 << 5;
                        result |= (imm5 & 0b11111) as u16;
                    }
                    _ => return Err(format!("Unsupported operands for ADD: {:?}", self.instruction))
                };

                Ok(vec![result])
            }

            Instruction { opcode: Opcode::Halt, operands} => {
                const OPCODE:u16 = 0b1111;

                if operands.len() > 0 {
                    return Err("HALT was used with operands, but does not take any".into())
                }

                let mut result: u16 = 0b0000_0000_0000_0000;
                result |= OPCODE << 12;
                result |= 0x25;

                Ok(vec![result])
            }

            Instruction { opcode: Opcode::And, operands} => {
                const OPCODE:u16 = 0b0101;
                let mut result: u16 = 0b0000_0000_0000_0000;
                result |= OPCODE << 12;

                match operands.as_slice() {
                    [Operand::Register { r: dr },
                     Operand::Register { r: sr1 },
                     Operand::Register { r: sr2 }] => {
                        result |= (dr.to_owned() as u16) << 9;
                        result |= (sr1.to_owned() as u16) << 6;
                        result |= sr2.to_owned() as u16;
                    }

                    [Operand::Register { r: dr },
                     Operand::Register { r: sr1 },
                     Operand::Immediate { value: imm5 }] => {
                        if imm5 > &31 {
                            return Err("Immediate value too large, must fit into 5 bits".into());
                        }

                        result |= (dr.to_owned() as u16) << 9;
                        result |= (sr1.to_owned() as u16) << 6;
                        result |= 1 << 5;
                        result |= (imm5 & 0b11111) as u16;
                    }

                    _ => return Err(format!("Unsupported operands for AND: {:?}", self.instruction))
                }
                Ok(vec![result])
            }

            Instruction { opcode: Opcode::Fill, operands} => {
                let res = operands
                    .iter()
                    .map(|x| match x {
                        Operand::Immediate { value } => *value as u16,
                        _ => panic!("Only immediate operands are allowed for fill in {:?}", self.instruction)
                    })
                    .collect();

                Ok(res)
            }

            Instruction { opcode: Opcode::Stringz, operands} => {
                let mut result: Vec<u16> = vec![];

                operands
                    .iter()
                    .for_each(|x| match x {
                        Operand::String { value} => {
                            for x in value.as_bytes() {
                                result.push(*x as u16)
                            }
                        }
                        _ => panic!("Only string operands are allowed for .STRINGZ in {:?}", self.instruction)
                    });

                result.push(0);
                Ok(result)
            }

            //_ => Err(format!("Can't emit unknown instruction {:?}", self.instruction))
        }
    }

    pub fn from(instruction: Instruction, offset: Offset) -> Self {
        Emittable { offset, instruction }
    }
}
