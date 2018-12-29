use std::fs::File;
use std::io;
use std::io::prelude::*;

type Result<T> = std::result::Result<T, String>;

#[macro_use]
extern crate num_derive;
extern crate num_traits;

mod state;
mod opcodes;
mod util;
mod parser;

use state::VmState;
use state::MyVmState;
use state::Registers;
use parser::Instruction;
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

fn run(state: &mut VmState) -> Result<()> {
    while state.running() {
        execute_next_instruction(state)?;
    }
    Ok(())
}

fn run_file(state: &mut VmState, filename: &str) -> io::Result<()> {
    load_object_file(filename, state)?;
    match run(state) {
        Ok(x) => Ok(x),
        Err(x) => Err(io::Error::new(io::ErrorKind::Other, x)),
    }
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


    // Test doubles

    pub struct TestVmDisplay<'a> {
        pub output: &'a mut String
    }

    impl<'a> state::VmDisplay for TestVmDisplay<'a> {
        fn print(&mut self, c: u8) -> () {
            self.output.push(c as char)
        }
    }


    // Utility functions

    #[inline]
    fn assert_cc_positive(state: &mut VmState) {
        assert_eq!(state.registers()[Registers::PSR] & (ConditionFlags::Positive as u16), ConditionFlags::Positive as u16);
        assert_eq!(state.registers()[Registers::PSR] & (ConditionFlags::Zero as u16), 0);
        assert_eq!(state.registers()[Registers::PSR] & (ConditionFlags::Negative as u16), 0);
    }

    #[inline]
    fn assert_cc_zero(state: &mut VmState) {
        assert_eq!(state.registers()[Registers::PSR] & (ConditionFlags::Positive as u16), 0);
        assert_eq!(state.registers()[Registers::PSR] & (ConditionFlags::Zero as u16), ConditionFlags::Zero as u16);
        assert_eq!(state.registers()[Registers::PSR] & (ConditionFlags::Negative as u16), 0);
    }

    #[inline]
    fn assert_cc_negative(state: &mut VmState) {
        assert_eq!(state.registers()[Registers::PSR] & (ConditionFlags::Positive as u16), 0);
        assert_eq!(state.registers()[Registers::PSR] & (ConditionFlags::Zero as u16), 0);
        assert_eq!(state.registers()[Registers::PSR] & (ConditionFlags::Negative as u16), ConditionFlags::Negative as u16);
    }


    // Tests

    #[test]
    fn test_br() {
        let mut state = MyVmState::new();
        let result = run_file(&mut state, "tests/br.obj");
        assert!(result.is_ok());
        assert_eq!(state.registers()[Registers::R2], 1);

        state.resume();
        run(&mut state).unwrap();
        assert_eq!(state.registers()[Registers::R2], 2);

        state.resume();
        run(&mut state).unwrap();
        assert_eq!(state.registers()[Registers::R2], 3);

        state.resume();
        run(&mut state).unwrap();
        assert_eq!(state.registers()[Registers::R2], 4);

        state.resume();
        run(&mut state).unwrap();
        assert_eq!(state.registers()[Registers::R2], 5);

        state.resume();
        run(&mut state).unwrap();
        assert_eq!(state.registers()[Registers::R2], 6);

        state.resume();
        run(&mut state).unwrap();
        assert_eq!(state.registers()[Registers::R2], 7);

        state.resume();
        run(&mut state).unwrap();
        assert_eq!(state.registers()[Registers::R2], 8);

        state.resume();
        run(&mut state).unwrap();
        assert_eq!(state.registers()[Registers::R2], 9);

        state.resume();
        run(&mut state).unwrap();
        assert_eq!(state.registers()[Registers::R2], 10);

        state.resume();
        run(&mut state).unwrap();
        assert_eq!(state.registers()[Registers::R2], 11);

        state.resume();
        run(&mut state).unwrap();
        assert_eq!(state.registers()[Registers::R2], 12);

        // This is only incremented on wrong branch, so should remain zero
        assert_eq!(state.registers()[Registers::R1], 0x0);
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
        assert_cc_positive(&mut state);

        state.resume();
        run(&mut state).unwrap();
        assert_eq!(state.registers()[Registers::R0], 0x0);
        assert_cc_zero(&mut state);

        state.resume();
        run(&mut state).unwrap();
        assert_eq!(state.registers()[Registers::R0], 0xFFFF);
        assert_cc_negative(&mut state);
    }

    #[test]
    fn test_add_register() {
        let mut state = MyVmState::new();
        let result = run_file(&mut state, "tests/add_register.obj");
        assert!(result.is_ok());

        assert_eq!(state.registers()[Registers::R0], 0x10);
        assert_cc_positive(&mut state);

        state.resume();
        run(&mut state).unwrap();
        assert_eq!(state.registers()[Registers::R0], 0x0);
        assert_cc_zero(&mut state);

        state.resume();
        run(&mut state).unwrap();
        assert_eq!(state.registers()[Registers::R0], 0xFFF0);
        assert_cc_negative(&mut state);
    }

    #[test]
    fn test_ld() {
        let mut state = MyVmState::new();
        let result = run_file(&mut state, "tests/ld.obj");
        assert!(result.is_ok());

        assert_eq!(state.registers()[Registers::R0], 0x4242);
        assert_cc_positive(&mut state);

        state.resume();
        run(&mut state).unwrap();
        assert_eq!(state.registers()[Registers::R0], 0x0);
        assert_cc_zero(&mut state);

        state.resume();
        run(&mut state).unwrap();
        assert_eq!(state.registers()[Registers::R0], 0xFFFF);
        assert_cc_negative(&mut state);
    }

    #[test]
    fn test_jmp() {
        let mut state = MyVmState::new();
        let result = run_file(&mut state, "tests/jmp.obj");
        assert!(result.is_ok(), "{}", result.unwrap_err());

        assert_eq!(state.registers()[Registers::PC], 0x3005);
        assert_eq!(state.registers()[Registers::R0], 1);
    }

    #[test]
    fn test_jsr_immediate() {
        let mut state = MyVmState::new();
        let result = run_file(&mut state, "tests/jsr_immediate.obj");
        assert!(result.is_ok(), "{}", result.unwrap_err());

        assert_eq!(state.registers()[Registers::PC], 0x3002);
        assert_eq!(state.registers()[Registers::R7], 0x3001);
        assert_eq!(state.registers()[Registers::R0], 1);
        assert_eq!(state.registers()[Registers::R1], 0);
    }

    #[test]
    fn test_jsr_register() {
        let mut state = MyVmState::new();
        let result = run_file(&mut state, "tests/jsr_register.obj");
        assert!(result.is_ok(), "{}", result.unwrap_err());

        assert_eq!(state.registers()[Registers::PC], 0x3003);
        assert_eq!(state.registers()[Registers::R7], 0x3002);
        assert_eq!(state.registers()[Registers::R0], 0x3005);
        assert_eq!(state.registers()[Registers::R1], 0);
        assert_eq!(state.registers()[Registers::R2], 1);
    }

    #[test]
    fn test_ldi() {
        let mut state = MyVmState::new();
        let result = run_file(&mut state, "tests/ldi.obj");
        assert!(result.is_ok(), "{}", result.unwrap_err());

        assert_eq!(state.registers()[Registers::R0], 0xFFFF);
        assert_cc_negative(&mut state);
    }

    #[test]
    fn test_ldr() {
        let mut state = MyVmState::new();
        let result = run_file(&mut state, "tests/ldr.obj");
        assert!(result.is_ok(), "{}", result.unwrap_err());

        assert_eq!(state.registers()[Registers::R0], 0x3004);
        assert_eq!(state.registers()[Registers::R1], 0xFFFF);
        assert_cc_negative(&mut state);
    }

    #[test]
    fn test_and() {
        let mut state = MyVmState::new();
        let result = run_file(&mut state, "tests/and.obj");
        assert!(result.is_ok(), "{}", result.unwrap_err());

        assert_eq!(state.registers()[Registers::R2], 0x1200);
        assert_cc_positive(&mut state);

        state.resume();
        run(&mut state).unwrap();
        assert_eq!(state.registers()[Registers::R2], 0);
        assert_cc_zero(&mut state);

        state.resume();
        run(&mut state).unwrap();
        assert_eq!(state.registers()[Registers::R2], 15);
        assert_cc_positive(&mut state);
    }

    #[test]
    fn test_not() {
        let mut state = MyVmState::new();
        let result = run_file(&mut state, "tests/not.obj");
        assert!(result.is_ok(), "{}", result.unwrap_err());

        assert_eq!(state.registers()[Registers::R1], 0xEDCB);
        assert_cc_negative(&mut state);
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
