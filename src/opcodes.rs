use num_traits::FromPrimitive;

use state::VmState;
use state::Registers;
use state::ConditionFlags;
use parser::Instruction;
use util::binary_add;

#[derive(FromPrimitive)]
pub enum Opcode {
    BR   = 0x0, /* branch */
    ADD  = 0x1, /* add  */
    LD   = 0x2, /* load */
    ST   = 0x3, /* store */
    JSR  = 0x4, /* jump register */
    AND  = 0x5, /* bitwise and */
    LDR  = 0x6, /* load register */
    STR  = 0x7, /* store register */
    RTI  = 0x8, /* unused */
    NOT  = 0x9, /* bitwise not */
    LDI  = 0xA, /* load indirect */
    STI  = 0xB, /* store indirect */
    JMP  = 0xC, /* jump */
    RES  = 0xD, /* reserved (unused) */
    LEA  = 0xE, /* load effective address */
    TRAP = 0xF, /* execute trap */
}

impl Opcode {
    pub fn from_instruction(instruction: u16) -> Self {
        // The upper three bits of an instruction are the opcode
        let opcode = instruction >> 12;
        match Opcode::from_u16(opcode) {
            Some(x) => x,
            None => panic!("Could not instantiate opcode from <0x{:X}>", opcode)
        }
    }
}

fn update_condition_codes(state: &mut VmState, value: u16) {
    state.registers()[Registers::PSR] &= 0b1111_1111_1111_1000;
    match value as i16 {
        x if x < 0 => state.registers()[Registers::PSR] |= ConditionFlags::Negative as u16,
        x if x > 0 => state.registers()[Registers::PSR] |= ConditionFlags::Positive as u16,
        _ => state.registers()[Registers::PSR] |= ConditionFlags::Zero as u16,
    }
}

pub fn execute_next_instruction(state: &mut VmState) -> Result<(), String> {
    let pc = state.registers()[Registers::PC];
    let instruction = Instruction::from_raw(state.memory()[pc as u16])?;

    // println!("PC<0x{:X}> {:?}", pc, instruction);

    match instruction {
            Instruction::Br { n, z, p, pc_offset9 } => {
                let mem_n: bool = (state.registers()[Registers::PSR] & ConditionFlags::Negative as u16) > 0;
                let mem_z: bool = (state.registers()[Registers::PSR] & ConditionFlags::Zero as u16) > 0;
                let mem_p: bool = (state.registers()[Registers::PSR] & ConditionFlags::Positive as u16) > 0;

                // If n, z, and p are set we want to unconditionally branch
                if (n && z && p) || (n && mem_n) || (z && mem_z) || (p && mem_p) {
                    state.registers()[Registers::PC] += pc_offset9
                }
            },

            Instruction::Jmp { base_r } => {
                // -1 because we increment the PC at the end of execute_next_instruction
                state.registers()[Registers::PC] = state.registers()[base_r] - 1;
            },

            Instruction::Jsr { pc_offset11 } => {
                state.registers()[Registers::R7] = state.registers()[Registers::PC] + 1;
                state.registers()[Registers::PC] += pc_offset11;
            },

            Instruction::AddImmediate { dr, sr1, imm5 } => {
                let sr1_val = state.registers()[sr1];
                let result = binary_add(sr1_val, imm5);
                state.registers()[dr] = result;
                update_condition_codes(state, result);
            },

            Instruction::AddRegister { dr, sr1, sr2 } => {
                let sr1_val = state.registers()[sr1];
                let sr2_val = state.registers()[sr2];
                let result = binary_add(sr1_val, sr2_val);
                state.registers()[dr] = result;
                update_condition_codes(state, result);
            },

            Instruction::Ld { dr, offset9 } => {
                let address = binary_add(pc + 1, offset9);
                let value = state.memory()[address];
                state.registers()[dr] = value;
                update_condition_codes(state, value);
            },

            Instruction::Lea { dr, offset9 } => {
                let address = binary_add(pc + 1, offset9);
                state.registers()[dr] = address;
                update_condition_codes(state, address);
            },

            Instruction::Trap { trapvect8 } => {
                op_trap(state, trapvect8);
            },
        }

        state.registers()[Registers::PC] += 1;
        Ok(())
}

pub fn op_trap(state: &mut VmState, trapvect8: u16) {
    // R7 is where we jump to upon completion of the handler. In the current implementation,
    // where we handle the traps in the VM, setting this is not necessary, but it's in the spec
    // let pc = state.registers()[Registers::PC];
    // state.registers()[Registers::R7] = (pc+1) as u16;

    match trapvect8 {
        0x22 => trap_puts(state),
        0x25 => trap_halt(state),
        _ => panic!("Unimplemented trap vector <0x{:x}>", trapvect8),
    }
}

fn trap_halt(state: &mut VmState) {
    state.halt()
}

fn trap_puts(state: &mut VmState) {
    let mut start = state.registers()[Registers::R0];
    while state.memory()[start] != 0 {
        let character = (state.memory()[start] & 0xFF) as u8;
        state.display().print(character);
        start += 1;
    }
}