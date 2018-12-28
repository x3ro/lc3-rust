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
use state::MyVmState;
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
    state.memory()[memory_area].copy_from_slice(program);
    state.registers()[Registers::PC] = orig;

    Ok(())
}

fn run(state: &mut VmState) {
    while state.running() {
        let pc = state.registers()[Registers::PC];
        let opcode = Opcode::from_instruction(state.memory()[pc as u16]);
        
        match opcode {
            Opcode::ADD => op_add(state, pc as usize),
            Opcode::LEA => op_lea(state, pc as usize),
            Opcode::TRAP => op_trap(state, pc as usize),
            _ => panic!("Unrecognized opcode <0x{:x}> at pc <0x{:x}>", opcode as u16, pc),
        }
    }
}

fn run_file(state: &mut VmState, filename: &str) -> io::Result<()> {
    load_object_file(filename, state)?;
    run(state);
    Ok(())
}

fn main() -> io::Result<()> {
    let mut state = MyVmState::new();

    match run_file(&mut state, "tests/puts.obj") {
        Ok(_) => Ok(()),
        Err(x) => Err(x),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use state::ConditionFlags;

    pub struct TestVmDisplay<'a> {
        pub output: &'a mut String
    }

    impl<'a> state::VmDisplay for TestVmDisplay<'a> {
        fn print(&mut self, c: u8) -> () {
            self.output.push(c as char)
        }
    }

    #[test]
    fn test_lea() {
        let mut state = MyVmState::new();
        let result = run_file(&mut state, "tests/lea.obj");
        assert!(result.is_ok());
        assert_eq!(state.registers()[Registers::R0], 0x3002);
    }

    #[test]
    fn test_add_immediate() {
        let mut state = MyVmState::new();
        let result = run_file(&mut state, "tests/add_immediate.obj");
        assert!(result.is_ok());

        assert_eq!(state.registers()[Registers::R0], 0x7);
        assert_eq!(state.registers()[Registers::PSR] & (ConditionFlags::Positive as u16) , ConditionFlags::Positive as u16);

        state.resume();
        run(&mut state);
        assert_eq!(state.registers()[Registers::R0], 0x0);
        assert_eq!(state.registers()[Registers::PSR] & (ConditionFlags::Zero as u16) , ConditionFlags::Zero as u16);

        state.resume();
        run(&mut state);
        assert_eq!(state.registers()[Registers::R0], 0xFFFF);
        assert_eq!(state.registers()[Registers::PSR] & (ConditionFlags::Negative as u16) , ConditionFlags::Negative as u16);
    }

    #[test]
    fn test_puts() {
        let mut output = String::new();
        {
            let d = TestVmDisplay{
                output: &mut output
            };

            let mut state = MyVmState::new_with_display(Box::new(d));
            let result = run_file(&mut state, "tests/puts.obj");
            assert!(result.is_ok());
        }
        assert_eq!("Hello World!", &mut output);
    }
}