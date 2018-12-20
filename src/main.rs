use std::fs::File;
use std::io;
use std::io::prelude::*;

#[macro_use]
extern crate num_derive;
extern crate num_traits;

mod state;
mod opcodes;
mod util;

use state::VmState;
use state::Registers;
use opcodes::*;

fn load_object_file(filename: &str, state: &mut VmState) -> io::Result<()> {
    let mut f = File::open(filename).expect(&format!("File <{}> not found", filename));

    let mut buffer: Vec<u8> = vec![];
    f.read_to_end(&mut buffer)?;

    // LC3 uses 16-bit words, so we need to combine two bytes into one word of memory
    let even = buffer.iter().step_by(2);
    let odd = buffer.iter().skip(1).step_by(2);
    let zipped = even.zip(odd);

    let data: Vec<u16> = zipped.map(|(&high, &low)| {
        (high as u16) << 8 | low as u16
    }).collect();

    // The first two bytes of the object file indicate where to load the program
    let orig = data[0];
    let program = &data[1..];
    println!("Loaded object file at <0x{:x}>", orig);

    let memory_area = (orig as usize)..((orig as usize) + program.len());
    state.memory[memory_area].copy_from_slice(program);
    state.registers[Registers::PC as usize] = orig;

    Ok(())
}

fn run(state: &mut VmState) {
    while state.running {
        let pc = state.registers[Registers::PC as usize] as usize;
        let opcode = Opcode::from_instruction(state.memory[pc]);
        
        match opcode {
            Opcode::LEA => op_lea(state, pc),
            Opcode::TRAP => op_trap(state, pc),
            _ => panic!("Unrecognized opcode <0x{:x}> at pc <0x{:x}>", opcode as u16, pc),
        }
    }
}

fn run_file(filename: &str) -> io::Result<VmState> {
    let mut state = VmState::new();

    load_object_file(filename, &mut state)?;
    run(&mut state);

    Ok(state)
}

fn main() -> io::Result<()> {
    match run_file("asm-test/test.obj") {
        Ok(_) => Ok(()),
        Err(x) => Err(x),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lea() {
        let state = run_file("tests/lea.obj").unwrap();
        assert_eq!(state.registers[Registers::R0 as usize], 0x3002);
    }
}