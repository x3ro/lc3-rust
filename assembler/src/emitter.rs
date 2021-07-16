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

    pub fn clamp(&self, value: i64, n_bits: u8) -> Result<i64, String> {
        let upper_limit = (2 as i64).pow(n_bits as u32 - 1) - 1;
        let lower_limit = -1 * upper_limit - 1;
        if value < lower_limit || value > upper_limit {
            Err(format!("Value '{}' was not within valid bounds: [{}, {}]", value, lower_limit, upper_limit))
        } else {
            let mask = (1 << n_bits) - 1;
            Ok(value & mask)
        }
    }

    pub fn unsupported_operands_err<T>(&self) -> Result<T, String> {
        return Err(format!("Unsupported operands for {:?}: {:?}", self.instruction.opcode, self.instruction.operands))
    }

    pub fn emit(&self, state: &Lc3State) -> Result<Vec<u16>, String> {
        match &self.instruction {
            Instruction { opcode: Opcode::Ld | Opcode::Ldi | Opcode::Lea, operands} => {
                let opcode:u16 = if self.instruction.opcode == Opcode::Ld {
                    0b0010
                } else if self.instruction.opcode == Opcode::Ldi {
                    0b1010
                } else {
                    0b1110
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

                    _ => return self.unsupported_operands_err()
                };

                Ok(vec![result])
            }

            Instruction { opcode: Opcode::Ldr, operands} => {
                const OPCODE:u16 = 0b0110;
                let mut result: u16 = OPCODE << 12;

                match operands.as_slice() {
                    [Operand::Register { r: dr }, Operand::Register { r: base_r }, Operand::Immediate { value: offset6}] => {
                        result |= (dr.to_owned() as u16) << 9;
                        result |= (base_r.to_owned() as u16) << 6;
                        result |= self.clamp(offset6.to_owned(), 6)? as u16;
                    }
                    _ => return self.unsupported_operands_err()
                }

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
                    _ => return self.unsupported_operands_err()
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
                    _ => return self.unsupported_operands_err()
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
                    _ => return self.unsupported_operands_err()

                }
                Ok(vec![result])
            }

            Instruction { opcode: Opcode::Not, operands} => {
                // For some reason the six least significant bits for the NOT opcode are
                // set according to the ISA 🤔
                let mut result: u16 = 0b1001_0000_0011_1111;

                match operands.as_slice() {
                    [Operand::Register { r: dr }, Operand::Register { r: sr }] => {
                        result |= (dr.to_owned() as u16) << 9;
                        result |= (sr.to_owned() as u16) << 6;
                    },
                    _ => return self.unsupported_operands_err()
                }

                Ok(vec![result])
            }

            Instruction { opcode: Opcode::Rti, .. } => {
                Ok(vec![0b1000_0000_0000_0000])
            }

            Instruction { opcode: Opcode::St | Opcode::Sti, operands} => {
                let opcode = if self.instruction.opcode == Opcode::St {
                    0b0011
                } else {
                    0b1011
                };

                let mut result: u16 = 0b0000_0000_0000_0000;
                result |= opcode << 12;

                match operands.as_slice() {
                    [Operand::Register { r: sr }, Operand::Label { name}] => {
                        result |= (sr.to_owned() as u16) << 9;
                        result |= state.relative_offset(self.offset, name, 9)?  ;
                    },
                    _ => return self.unsupported_operands_err()
                }

                Ok(vec![result])
            }

            Instruction { opcode: Opcode::Str, operands} => {
                let mut result: u16 = 0b0111_0000_0000_0000;

                match operands.as_slice() {
                    [Operand::Register { r: sr }, Operand::Register { r: base_r }, Operand::Immediate { value}] => {
                        result |= (sr.to_owned() as u16) << 9;
                        result |= (base_r.to_owned() as u16) << 6;
                        result |= self.clamp(value.to_owned(), 6)? as u16;
                    },
                    _ => return self.unsupported_operands_err()
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

                    _ => return self.unsupported_operands_err()
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

                    _ => return self.unsupported_operands_err()
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
        }
    }

    pub fn from(instruction: Instruction, offset: Offset) -> Self {
        Emittable { offset, instruction }
    }
}
