use std::fmt::Write;

use crate::parser::Instruction;
use crate::state::{ConditionFlags, Registers};
use crate::VmState;

fn fmt_register(s: &mut String, state: &mut dyn VmState, r: &Registers) {
    write!(s, "{:?} (=#{:?})", r, state.registers()[r] as i16);
}

fn fmt_immediate(s: &mut String, imm: &u16) {
    write!(s, "#{}", *imm as i16);
}

pub fn fmt_psr(state: &mut dyn VmState) -> String {
    let psr = state.registers()[Registers::PSR];
    let n = psr & ConditionFlags::Negative as u16;
    let z = psr & ConditionFlags::Zero as u16;
    let p = psr & ConditionFlags::Positive as u16;
    format!("           ⮑  Updated PSR n = {:?} z = {:?} p = {:?}", n, z, p)
}

pub fn fmt_instruction(state: &mut dyn VmState, instruction: &Instruction) -> String {
    let mut s = String::new();

    write!(s, "PC<0x{:X}> ", state.registers()[Registers::PC]).unwrap();

    match instruction {
        Instruction::AddRegister { dr, sr1, sr2 } => {
            write!(s, "ADD ");
            fmt_register(&mut s, state, dr);
            write!(s, ", ");
            fmt_register(&mut s, state, sr1);
            write!(s, ", ");
            fmt_register(&mut s, state, sr2);
        }

        Instruction::AddImmediate { dr, sr1, imm5 } => {
            write!(s, "ADD ");
            fmt_register(&mut s, state, dr);
            write!(s, ", ");
            fmt_register(&mut s, state, sr1);
            write!(s, ", ");
            fmt_immediate(&mut s, imm5);
        }

        _ => write!(s, "{:?}", instruction).unwrap(),
    };

    s
}
