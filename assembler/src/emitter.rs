use tokens::*;
use std::fmt::Debug;
use ::{Lc3State, Offset};

#[derive(Debug)]
pub struct Emittable {
    offset: Offset,
    instruction: Instruction,
}

impl Emittable {
    // This is the size in LC3 words, which are 16 bits in size.
    // So 16 bits = size 1, 32 bits = size 2, ...
    pub fn size(&self) -> u16 {
        match &self.instruction {
            Instruction { opcode: Opcode::Stringz, operands} => {
                match operands.as_slice() {
                    [Operand::String { value }] =>
                        // +1 because STRINGZ emits a zero-terminated string, and value.len()
                        // will not include the \0 at the end (this is added when emitting)
                        return value.len() as u16 + 1,
                    _ =>
                        panic!("Only one string operand is .STRINGZ in {:?}", self.instruction)
                }
            },
            // TODO: not all emittables are one word long (e.g. stringz or fill)
            _ => 1
        }
    }

    pub fn emit(&self, state: &Lc3State) -> Result<Vec<u16>, String> {
        match &self.instruction {
            Instruction { opcode: Opcode::Ld | Opcode::Ldi, operands} => {
                let opcode:u16 = if self.instruction.opcode == Opcode::Ld {
                    0b0010
                } else {
                    0b1010
                };

                let mut result: u16 = opcode << 12;

                match operands.as_slice() {
                    [Operand::Register {r: dr}, Operand::Label {name}] => {
                        result |= (dr.to_owned() as u16) << 9;

                        let offset = state.relative_offset(self.offset, name, 9);
                        match offset {
                            Ok(x) => result |= x as u16,
                            Err(x) => return Err(x)
                        }
                    },

                    _ => return Err(format!("Unsupported operands for {:?}: {:?}", self.instruction.opcode, self.instruction.operands))
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

            Instruction { opcode: Opcode::Jmp, operands} => {
                const OPCODE:u16 = 0b1100;

                let mut result: u16 = 0b0000_0000_0000_0000;
                result |= OPCODE << 12;

                match operands.as_slice() {
                    [Operand::Register {r: base_r }] => {
                        result |= (base_r.to_owned() as u16 & 0b111) << 6;
                    }
                    _ => return Err(format!("Unsupported operands for JMP: {:?}", self.instruction))
                }

                Ok(vec![result])
            }

            // RET is just an alias for `JMP R7`
            Instruction { opcode: Opcode::Ret, .. } => {
                Emittable::from(
                 Instruction {
                        opcode: Opcode::Jmp,
                        operands: vec![Operand::Register { r: Registers::R7 }],
                    },
                    self.offset
                ).emit(state)
            }

            Instruction { opcode: Opcode::Jsr | Opcode::Jsrr, operands} => {
                const OPCODE:u16 = 0b0100;
                let mut result: u16 = 0b0000_0000_0000_0000;

                result |= OPCODE << 12;

                match operands.as_slice() {
                    [Operand::Label { name}] => {
                        let offset =
                            state.relative_offset(self.offset, name, 11)
                                .map_err(|x| format!("Did not find label '{}' in JSR call", x))?;

                        // This flag indicates that it's a relative jump (i.e. jump target _not_ from a register)
                        result |= 1 << 11;
                        result |= offset;
                    },
                    [Operand::Register { r}] => {
                        result |= (r.to_owned() as u16) << 6;
                    },
                    _ => return Err(format!("Unsupported operands for {:?}: {:?}", self.instruction.opcode, self.instruction.operands))

                }
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

            Instruction { opcode: Opcode::Br { modifiers }, operands} => {
                const OPCODE:u16 = 0b0000;
                let mut result: u16 = 0b0000_0000_0000_0000;
                result |= OPCODE << 12;

                match operands.as_slice() {
                    [Operand::Label { name }] => {
                        let imm9_opt = state.labels.get(name);
                        if imm9_opt.is_none() {
                            return Err(format!("Label used for BR instruction not found: {}", name).into());
                        }

                        if let Some(mods) = modifiers {
                            result |= (mods.contains("n") as u16) << 11;
                            result |= (mods.contains("z") as u16) << 10;
                            result |= (mods.contains("p") as u16) << 9;
                        }

                        let offset = state.relative_offset(self.offset, name, 9);
                        match offset {
                            Ok(x) => result |= x as u16,
                            Err(x) => return Err(x)
                        }
                    }

                    _ => return Err(format!("Unsupported operands for BR: {:?}", self.instruction))
                }
                Ok(vec![result])
            }

            Instruction { opcode: Opcode::Fill, operands} => {
                let res = operands
                    .iter()
                    .map(|x| match x {
                        Operand::Immediate { value } => *value as u16,
                        Operand::Label { name } => {
                            match state.labels.get(name) {
                                Some(x) => (x.to_owned() as u16) & 0b111111111,
                                _ => panic!("Did not find label with name '{:?}' in .FILL", name)
                            }
                        },
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

            // _ => Err(format!("Can't emit unknown instruction {:?}", self.instruction))
        }
    }

    pub fn from(instruction: Instruction, offset: Offset) -> Self {
        Emittable { offset, instruction }
    }
}
