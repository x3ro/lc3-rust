use std::cell::RefCell;

use lc3vm::peripheral::{AutomatedKeyboard, CapturingDisplay};
use lc3vm::state::{ConditionFlags, Registers, VmState};
use lc3vm::{load_words, run};

// Utility functions

#[inline]
fn assert_cc_positive(state: &mut VmState) {
    assert_eq!(
        state.registers()[Registers::PSR] & (ConditionFlags::Positive as u16),
        ConditionFlags::Positive as u16
    );
    assert_eq!(
        state.registers()[Registers::PSR] & (ConditionFlags::Zero as u16),
        0
    );
    assert_eq!(
        state.registers()[Registers::PSR] & (ConditionFlags::Negative as u16),
        0
    );
}

#[inline]
fn assert_cc_zero(state: &mut VmState) {
    assert_eq!(
        state.registers()[Registers::PSR] & (ConditionFlags::Positive as u16),
        0
    );
    assert_eq!(
        state.registers()[Registers::PSR] & (ConditionFlags::Zero as u16),
        ConditionFlags::Zero as ./u16
    );
    assert_eq!(
        state.registers()[Registers::PSR] & (ConditionFlags::Negative as u16),
        0
    );
}

#[inline]
fn assert_cc_negative(state: &mut VmState) {
    assert_eq!(
        state.registers()[Registers::PSR] & (ConditionFlags::Positive as u16),
        0
    );
    assert_eq!(
        state.registers()[Registers::PSR] & (ConditionFlags::Zero as u16),
        0
    );
    assert_eq!(
        state.registers()[Registers::PSR] & (ConditionFlags::Negative as u16),
        ConditionFlags::Negative as u16
    );
}

fn assert_supervisor_mode(state: &mut VmState, enabled: bool) {
    if enabled {
        assert_eq!(
            state.registers()[Registers::PSR] & 0b1000_0000_0000_0000,
            0b1000_0000_0000_0000
        );
    } else {
        assert_eq!(state.registers()[Registers::PSR] & 0b1000_0000_0000_0000, 0);
    }
}

// Tests

macro_rules! prepare_test {
    ($file:expr) => {{
        let _ = pretty_env_logger::try_init();
        let mut state = VmState::new();
        let source = include_str!($file);
        let data = lc3as::assemble(source).unwrap();
        load_words(&data, &mut state).unwrap();
        state
    }};
    ($file:expr, $entrypoint:expr) => {{
        let _ = pretty_env_logger::try_init();
        let mut state = VmState::new();
        let source = include_str!($file);
        let data = lc3as::assemble(source).unwrap();
        load_words(&data, &mut state).unwrap();
        state.set_pc($entrypoint);
        state
    }};
}

#[test]
fn test_br() {
    let mut state = prepare_test!("../../testcases/assembly/br.asm");
    let result = run(&mut state);

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
    let mut state = prepare_test!("../../testcases/assembly/lea.asm");
    let result = run(&mut state);
    assert!(result.is_ok());
    assert_eq!(state.registers()[Registers::R0], 0x3002);
}

#[test]
fn test_add_immediate() {
    let mut state = prepare_test!("../../testcases/assembly/add_immediate.asm");
    let result = run(&mut state);
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
    let mut state = prepare_test!("../../testcases/assembly/add_register.asm");
    let result = run(&mut state);
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
    let mut state = prepare_test!("../../testcases/assembly/ld.asm");
    let result = run(&mut state);
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
    let mut state = prepare_test!("../../testcases/assembly/jmp.asm");
    let result = run(&mut state);
    assert!(result.is_ok(), "{}", result.unwrap_err());

    assert_eq!(state.registers()[Registers::PC], 0x3005);
    assert_eq!(state.registers()[Registers::R0], 1);
}

#[test]
fn test_jsr_immediate() {
    let mut state = prepare_test!("../../testcases/assembly/jsr_immediate.asm");
    let result = run(&mut state);
    assert!(result.is_ok(), "{}", result.unwrap_err());

    assert_eq!(state.registers()[Registers::PC], 0x3002);
    assert_eq!(state.registers()[Registers::R0], 1);
    assert_eq!(state.registers()[Registers::R1], 0);
}

#[test]
fn test_jsr_register() {
    let mut state = prepare_test!("../../testcases/assembly/jsr_register.asm");
    let result = run(&mut state);
    assert!(result.is_ok(), "{}", result.unwrap_err());

    assert_eq!(state.registers()[Registers::PC], 0x3003);
    assert_eq!(state.registers()[Registers::R7], 0x3002);
    assert_eq!(state.registers()[Registers::R0], 0x3005);
    assert_eq!(state.registers()[Registers::R1], 0);
    assert_eq!(state.registers()[Registers::R2], 1);
}

#[test]
fn test_ldi() {
    let mut state = prepare_test!("../../testcases/assembly/ldi.asm");
    let result = run(&mut state);
    assert!(result.is_ok(), "{}", result.unwrap_err());

    assert_eq!(state.registers()[Registers::R0], 0xFFFF);
    assert_cc_negative(&mut state);
}

#[test]
fn test_ldr() {
    let mut state = prepare_test!("../../testcases/assembly/ldr.asm");
    let result = run(&mut state);
    assert!(result.is_ok(), "{}", result.unwrap_err());

    assert_eq!(state.registers()[Registers::R0], 0x3004);
    assert_eq!(state.registers()[Registers::R1], 0xFFFF);
    assert_cc_negative(&mut state);
}

#[test]
fn test_and() {
    let mut state = prepare_test!("../../testcases/assembly/and.asm");
    let result = run(&mut state);
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
    let mut state = prepare_test!("../../testcases/assembly/not.asm");
    let result = run(&mut state);
    assert!(result.is_ok(), "{}", result.unwrap_err());

    assert_eq!(state.registers()[Registers::R1], 0xEDCB);
    assert_cc_negative(&mut state);
}

#[test]
fn test_st() {
    let mut state = prepare_test!("../../testcases/assembly/st.asm");
    let result = run(&mut state);
    assert!(result.is_ok(), "{}", result.unwrap_err());

    assert_eq!(state.memory()[0x3003], (-7i16) as u16);
}

#[test]
fn test_sti() {
    let mut state = prepare_test!("../../testcases/assembly/sti.asm");
    let result = run(&mut state);
    assert!(result.is_ok(), "{}", result.unwrap_err());

    assert_eq!(state.memory()[0x3003], (-8i16) as u16);
}

#[test]
fn test_str() {
    let mut state = prepare_test!("../../testcases/assembly/str.asm");
    let result = run(&mut state);
    assert!(result.is_ok(), "{}", result.unwrap_err());

    assert_eq!(state.memory()[0x3004], (-9i16) as u16);
}

#[test]
fn test_trap() {
    let mut state = prepare_test!("../../testcases/assembly/trap.asm", 0x200);
    let result = run(&mut state);
    assert!(result.is_ok(), "{}", result.unwrap_err());

    assert_eq!(state.registers()[Registers::R0], 15);
}

#[test]
fn test_br_backwards() {
    let mut state = prepare_test!("../../testcases/assembly/br_backwards.asm");
    let result = run(&mut state);
    assert!(result.is_ok(), "{}", result.unwrap_err());

    assert_eq!(state.registers()[Registers::R0], 10);
}

#[test]
#[ignore] // Interrupts do not currently work
fn test_rti() {
    let mut state = prepare_test!("../../testcases/assembly/rti.asm");
    let result = run(&mut state);
    assert!(result.is_ok(), "{}", result.unwrap_err());

    assert_supervisor_mode(&mut state, false);
    assert_eq!(state.registers()[Registers::R0], (-1i16) as u16);
    assert_cc_negative(&mut state);

    //tx.send(0x42).unwrap(); // Send an interrupt defined in test file
    state.resume();
    run(&mut state).unwrap();

    // Test supervisor mode
    assert_supervisor_mode(&mut state, true);

    // TODO: Test priority level (?)
    // How does it work?!?

    // Test supervisor stack pointer, should be base of the supervisor
    // stack minus space for saved PSR and PC
    assert_eq!(state.registers()[Registers::R6], 0x3000 - 2);

    // Test userland PC pushed onto supervisor stack
    assert_eq!(state.memory()[0x3000 - 2], 0x202);

    // Test userland PSR pushed onto supervisor stack
    state.resume();
    run(&mut state).unwrap();
    assert_eq!(state.registers()[Registers::R5], 5);

    // Test that we're returning to the correct position after RTI
    state.resume();
    run(&mut state).unwrap();
    assert_eq!(state.registers()[Registers::PC], 0x204);
    assert_eq!(state.registers()[Registers::R0], (-2i16) as u16);
}

#[test]
fn test_puts() {
    let display = CapturingDisplay {
        output: RefCell::new("".into()),
    };

    {
        let mut state = prepare_test!("../../testcases/assembly/puts.asm", 0x100);
        state.peripherals.push(&display);
        let result = run(&mut state);
        assert!(result.is_ok());
    }

    assert_eq!("Hello World!\n", display.output.borrow().as_str());
}

#[test]
fn test_os() {
    let display = CapturingDisplay {
        output: RefCell::new("".into()),
    };

    let keyboard = AutomatedKeyboard::new("merp".into());

    {
        let mut state = prepare_test!("../../testcases/assembly/os.asm", 0x200);
        state.peripherals.push(&display);
        state.peripherals.push(&keyboard);
        let result = run(&mut state);
        assert!(result.is_ok());
    }

    let expected = r#"
Welcome to the LC-3 simulator.

The contents of the LC-3 tools distribution, including sources, management
tools, and data, are Copyright (c) 2003 Steven S. Lumetta.

The LC-3 tools distribution is free software covered by the GNU General
Public License, and you are welcome to modify it and/or distribute copies
of it under certain conditions.  The file COPYING (distributed with the
tools) specifies those conditions.  There is absolutely no warranty for
the LC-3 tools distribution, as described in the file NO_WARRANTY (also
distributed with the tools).

Have fun.

Input a character> m

Input a character> e

Input a character> r

Input a character> p
"#;

    assert_eq!(expected, display.output.borrow().as_str());
}
