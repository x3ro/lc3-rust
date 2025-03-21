use crate::debug::{fmt_instruction, fmt_psr};
use anyhow::Result;

use crate::parser::Instruction;
use crate::state::ConditionFlags;
use crate::state::Registers;
use crate::state::VmState;
use crate::util::binary_add;

fn update_condition_codes(state: &mut VmState, value: u16) {
    state.registers()[Registers::PSR] &= 0b1111_1111_1111_1000;
    match value as i16 {
        x if x < 0 => state.registers()[Registers::PSR] |= ConditionFlags::Negative as u16,
        x if x > 0 => state.registers()[Registers::PSR] |= ConditionFlags::Positive as u16,
        _ => state.registers()[Registers::PSR] |= ConditionFlags::Zero as u16,
    }

    trace!("{}", fmt_psr(state));
}

pub fn next_instruction(state: &mut VmState) -> Instruction {
    let pc = state.registers()[Registers::PC];
    Instruction::from_raw(state.memory()[pc as u16])
}

pub fn execute_next_instruction(state: &mut VmState) -> Result<()> {
    let pc = state.registers()[Registers::PC];
    let instruction = next_instruction(state);

    debug!("{}", fmt_instruction(state, &instruction)?);

    match &instruction {
        Instruction::AddRegister { dr, sr1, sr2 } => {
            let sr1_val = state.registers()[sr1];
            let sr2_val = state.registers()[sr2];
            let result = binary_add(sr1_val, sr2_val);

            state.registers()[dr] = result;
            update_condition_codes(state, result);
        }

        Instruction::AddImmediate { dr, sr1, imm5 } => {
            let sr1_val = state.registers()[sr1];
            let result = binary_add(sr1_val, *imm5);

            state.registers()[dr] = result;
            update_condition_codes(state, result);
        }

        Instruction::AndRegister { dr, sr1, sr2 } => {
            let sr1_val = state.registers()[sr1];
            let sr2_val = state.registers()[sr2];
            let result = sr1_val & sr2_val;

            state.registers()[dr] = result;
            update_condition_codes(state, result);
        }

        Instruction::AndImmediate { dr, sr1, imm5 } => {
            let sr1_val = state.registers()[sr1];
            let result = sr1_val & imm5;

            state.registers()[dr] = result;
            update_condition_codes(state, result);
        }

        Instruction::Br {
            n,
            z,
            p,
            pc_offset9,
        } => {
            let mem_n: bool =
                (state.registers()[Registers::PSR] & ConditionFlags::Negative as u16) > 0;
            let mem_z: bool = (state.registers()[Registers::PSR] & ConditionFlags::Zero as u16) > 0;
            let mem_p: bool =
                (state.registers()[Registers::PSR] & ConditionFlags::Positive as u16) > 0;

            // If n, z, and p are set we want to unconditionally branch
            if (*n && *z && *p) || (*n && mem_n) || (*z && mem_z) || (*p && mem_p) {
                let pc = state.registers()[Registers::PC];
                state.registers()[Registers::PC] = binary_add(pc, *pc_offset9);
            }
        }

        Instruction::Jmp { base_r } => {
            // -1 because we increment the PC at the end of execute_next_instruction
            state.registers()[Registers::PC] = state.registers()[base_r] - 1;
        }

        Instruction::JsrImmediate { pc_offset11 } => {
            state.registers()[Registers::R7] = state.registers()[Registers::PC] + 1;
            state.registers()[Registers::PC] += pc_offset11;
        }

        Instruction::JsrRegister { base_r } => {
            state.registers()[Registers::R7] = state.registers()[Registers::PC] + 1;
            state.registers()[Registers::PC] = state.registers()[base_r] - 1;
        }

        Instruction::Ld { dr, offset9 } => {
            let address = binary_add(pc + 1, *offset9);
            let value = state.memory_mut()[address];

            state.registers()[dr] = value;
            update_condition_codes(state, value);
        }

        Instruction::Ldi { dr, offset9 } => {
            let address1 = binary_add(pc + 1, *offset9);
            let address2 = state.memory_mut()[address1];
            let result = state.memory_mut()[address2];

            state.registers()[dr] = result;
            update_condition_codes(state, result);
        }

        Instruction::Ldr {
            dr,
            base_r,
            offset6,
        } => {
            let address1 = state.registers()[base_r];
            let address2 = binary_add(address1, *offset6);
            let value = state.memory_mut()[address2];

            state.registers()[dr] = value;
            update_condition_codes(state, value);
        }

        Instruction::Lea { dr, offset9 } => {
            let address = binary_add(pc + 1, *offset9);
            state.registers()[dr] = address;
            update_condition_codes(state, address);
        }

        Instruction::Not { dr, sr } => {
            let value = !state.registers()[sr];
            state.registers()[dr] = value;
            update_condition_codes(state, value);
        }

        Instruction::Rti {} => {
            let psr = state.registers()[Registers::PSR];
            if (psr & 0b1000_0000_0000_0000) == 0 {
                unimplemented!("This should yield a privilege exception");
            }

            let ssp = state.registers()[Registers::R6];
            let pc = state.memory_mut()[ssp];
            let psr = state.memory_mut()[ssp + 1];
            let usp = state.registers()[Registers::USP];

            // -1 because we increment the PC at the end of execute_next_instruction
            state.registers()[Registers::PC] = pc - 1;
            state.registers()[Registers::PSR] = psr;
            state.registers()[Registers::R6] = usp;
        }

        Instruction::St { sr, pc_offset9 } => {
            let addr = binary_add(pc + 1, *pc_offset9);
            let value = state.registers()[sr];
            state.memory_mut()[addr] = value;
        }

        Instruction::Sti { sr, pc_offset9 } => {
            let addr1 = binary_add(pc + 1, *pc_offset9);
            let addr2 = state.memory_mut()[addr1];
            let value = state.registers()[sr];
            state.memory_mut()[addr2] = value;
        }

        Instruction::Str {
            sr,
            base_r,
            offset6,
        } => {
            let base_addr = state.registers()[base_r];
            let addr = binary_add(base_addr, *offset6);
            let value = state.registers()[sr];
            state.memory_mut()[addr] = value;
        }

        Instruction::Trap { trapvect8 } => {
            match trapvect8 {
                // TODO: Remove these super-hacky VM-handled traps
                // 0x22 => trap_puts(state),
                // Hacky halt TRAP catch to make writing test cases easier, i.e. not
                // having to load HALT routine code for every test-case, and being
                // able to observe condition codes / register states just before the HALT
                // TODO: Think of a better way to handle this
                0x25 => {
                    info!("Halting!");
                    state.memory_mut()[0xFFFE] = 0;
                }
                _ => {
                    let pc = state.registers()[Registers::PC];
                    state.registers()[Registers::R7] = pc + 1;
                    // -1 because we increment the PC at the end of execute_next_instruction
                    let new_pc = state.memory_mut()[*trapvect8] - 1;
                    state.registers()[Registers::PC] = new_pc;
                }
            }
        }
    }

    state.registers()[Registers::PC] += 1;
    Ok(())
}

fn _handle_interrupt(state: &mut VmState, interrupt: u16) {
    // The processor sets the privilege mode to Supervisor mode (PSR[15] = 0)
    state.registers()[Registers::PSR] |= 0b1000_0000_0000_0000;

    // (?) The processor sets the priority level to PL4, the priority level of the interrupting device (PSR[10:8] = 100)

    // The PSR and PC of the interrupted process are pushed onto the Supervisor Stack.
    let ssp = state.registers()[Registers::SSP];
    state.memory_mut()[ssp - 1] = state.registers()[Registers::PSR];
    state.memory_mut()[ssp - 2] = state.registers()[Registers::PC];

    // R6 is loaded with the Supervisor Stack Pointer (SSP) if it does not already contain the SSP.
    state.registers()[Registers::R6] = ssp - 2; // -2 because we just pushed PSR and PC

    // Expand the vector to the corresponding 16-bit address in the interrupt vector table
    // The PC is loaded with the contents of memory at that address
    let interrupt_table_addr = 0x100 + interrupt;
    let routine_addr = state.memory_mut()[interrupt_table_addr];
    state.registers()[Registers::PC] = routine_addr;

    debug!(
        "Interrupt vector <0x{:X}> pointing to addr <0x{:X}>",
        interrupt, routine_addr
    );
}
