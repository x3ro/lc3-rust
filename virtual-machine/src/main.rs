use std::fs::File;
use std::io::prelude::*;

use std::thread;
use std::time::{Duration, Instant};

#[macro_use]
extern crate log;
extern crate pretty_env_logger;
#[macro_use]
extern crate num_derive;
extern crate anyhow;
extern crate clap;
extern crate num_traits;

use anyhow::Result;
use clap::{App, Arg};

#[macro_use]
mod util;
mod opcodes;
mod parser;
mod peripheral;
mod state;

use opcodes::*;
use parser::Instruction;
use peripheral::{Peripheral, TerminalDisplay, TerminalKeyboard};
use state::MyVmState;
use state::Registers;
use state::VmState;

#[derive(Clone)]
pub struct VmOptions<'a> {
    pub throttle: Option<Duration>,
    pub peripherals: Vec<&'a dyn Peripheral>,
    pub entry_point: u16,
    pub filenames: Vec<String>,
}

impl<'a> VmOptions<'a> {
    pub fn with_entrypoint(&self, entry_point: u16) -> Self {
        VmOptions {
            entry_point,
            ..self.clone()
        }
    }

    pub fn with_filenames(&self, filenames: Vec<String>) -> Self {
        VmOptions {
            filenames,
            ..self.clone()
        }
    }

    pub fn with_filename(&self, filename: &str) -> Self {
        VmOptions {
            filenames: vec![filename.into()],
            ..self.clone()
        }
    }
}

fn load_object_file(filename: &str, state: &mut dyn VmState) -> Result<()> {
    let mut f = File::open(filename).expect(&format!("File <{}> not found", filename));

    let mut buffer: Vec<u8> = vec![];
    f.read_to_end(&mut buffer)?;

    // LC3 uses 16-bit words, so we need to combine two bytes into one word of memory
    let even = buffer.iter().step_by(2);
    let odd = buffer.iter().skip(1).step_by(2);
    let zipped = even.zip(odd);

    let data: Vec<u16> = zipped
        .map(|(&high, &low)| (high as u16) << 8 | low as u16)
        .collect();

    // The first two bytes of the object file indicate where to load the program
    let orig = data[0];
    let program = &data[1..];
    debug!("Loaded <{}> at <0x{:x}>", filename, orig);

    let memory_area = (orig as usize)..((orig as usize) + program.len());
    state.memory()[memory_area].copy_from_slice(program);

    Ok(())
}

fn run(state: &mut dyn VmState, opts: &VmOptions) -> Result<()> {
    let mut ticks = 0;
    let start = Instant::now();

    while state.running() {
        state.tick();
        execute_next_instruction(state)?;

        for p in &opts.peripherals {
            p.run(state);
        }

        if opts.throttle.is_some() {
            thread::sleep(opts.throttle.unwrap());
        }

        ticks += 1;
    }

    let elapsed = start.elapsed();
    info!(
        "Ran {:?} instructions in {:?}ms ({:?} kHz)",
        ticks,
        elapsed.as_millis(),
        (ticks as f64 / elapsed.as_secs_f64() / 1000.0) as u64
    );

    Ok(())
}

fn load_files(state: &mut dyn VmState, opts: &VmOptions) -> Result<()> {
    state.registers()[Registers::PC] = opts.entry_point;

    for filename in &opts.filenames {
        load_object_file(filename, state)?;
    }

    Ok(())
}

fn parse_options<'a>() -> VmOptions<'a> {
    let matches = App::new("Rust LC3 simulator")
        .arg(
            Arg::with_name("programs")
                .short("p")
                .long("program")
                .value_name("FILE")
                .multiple(true)
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("entry_point")
                .short("e")
                .long("entry-point")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("throttle")
                .long("throttle")
                .value_name("MILLISECONDS")
                .takes_value(true),
        )
        .get_matches();

    let filenames: Vec<String> = matches
        .values_of("programs")
        .unwrap()
        .map(|s| s.into())
        .collect();

    let entry_point = matches.value_of("entry_point").unwrap_or("0x3000");

    let throttle = matches
        .value_of("throttle")
        .and_then(|x| x.parse::<u64>().ok())
        .map(Duration::from_millis);

    let entry_point = u16::from_str_radix(entry_point.trim_start_matches("0x"), 16).unwrap();

    VmOptions {
        throttle,
        peripherals: vec![],
        filenames,
        entry_point,
    }
}

fn main() -> Result<()> {
    pretty_env_logger::init();

    let mut opts = parse_options();
    let mut state = MyVmState::new();

    let display = TerminalDisplay {};
    let keyboard = TerminalKeyboard::new();
    opts.peripherals.push(&display);
    opts.peripherals.push(&keyboard);

    load_files(&mut state, &opts)?;
    run(&mut state, &opts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use peripheral::{AutomatedKeyboard, CapturingDisplay};
    use state::ConditionFlags;
    use std::cell::RefCell;

    // Utility functions

    #[inline]
    fn assert_cc_positive(state: &mut dyn VmState) {
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
    fn assert_cc_zero(state: &mut dyn VmState) {
        assert_eq!(
            state.registers()[Registers::PSR] & (ConditionFlags::Positive as u16),
            0
        );
        assert_eq!(
            state.registers()[Registers::PSR] & (ConditionFlags::Zero as u16),
            ConditionFlags::Zero as u16
        );
        assert_eq!(
            state.registers()[Registers::PSR] & (ConditionFlags::Negative as u16),
            0
        );
    }

    #[inline]
    fn assert_cc_negative(state: &mut dyn VmState) {
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

    fn assert_supervisor_mode(state: &mut dyn VmState, enabled: bool) {
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

    const DEFAULT_OPTS: VmOptions = VmOptions {
        throttle: None,
        peripherals: vec![],
        filenames: vec![],
        entry_point: 0x3000,
    };

    macro_rules! prepare_test {
        ($file:expr) => {{
            let _ = pretty_env_logger::try_init();
            let opts = DEFAULT_OPTS.with_filename($file);
            let mut state = MyVmState::new();
            load_files(&mut state, &opts).unwrap();
            (state, opts)
        }};
        ($file:expr, $entrypoint:expr) => {{
            let _ = pretty_env_logger::try_init();
            let opts = DEFAULT_OPTS
                .with_filename($file)
                .with_entrypoint($entrypoint);
            let mut state = MyVmState::new();
            load_files(&mut state, &opts).unwrap();
            (state, opts)
        }};
    }

    #[test]
    fn test_br() {
        let (mut state, opts) = prepare_test!("tests/br.obj");
        let result = run(&mut state, &opts);

        assert!(result.is_ok());
        assert_eq!(state.registers()[Registers::R2], 1);

        state.resume();
        run(&mut state, &opts).unwrap();
        assert_eq!(state.registers()[Registers::R2], 2);

        state.resume();
        run(&mut state, &opts).unwrap();
        assert_eq!(state.registers()[Registers::R2], 3);

        state.resume();
        run(&mut state, &opts).unwrap();
        assert_eq!(state.registers()[Registers::R2], 4);

        state.resume();
        run(&mut state, &opts).unwrap();
        assert_eq!(state.registers()[Registers::R2], 5);

        state.resume();
        run(&mut state, &opts).unwrap();
        assert_eq!(state.registers()[Registers::R2], 6);

        state.resume();
        run(&mut state, &opts).unwrap();
        assert_eq!(state.registers()[Registers::R2], 7);

        state.resume();
        run(&mut state, &opts).unwrap();
        assert_eq!(state.registers()[Registers::R2], 8);

        state.resume();
        run(&mut state, &opts).unwrap();
        assert_eq!(state.registers()[Registers::R2], 9);

        state.resume();
        run(&mut state, &opts).unwrap();
        assert_eq!(state.registers()[Registers::R2], 10);

        state.resume();
        run(&mut state, &opts).unwrap();
        assert_eq!(state.registers()[Registers::R2], 11);

        state.resume();
        run(&mut state, &opts).unwrap();
        assert_eq!(state.registers()[Registers::R2], 12);

        // This is only incremented on wrong branch, so should remain zero
        assert_eq!(state.registers()[Registers::R1], 0x0);
    }

    #[test]
    fn test_lea() {
        let (mut state, opts) = prepare_test!("tests/lea.obj");
        let result = run(&mut state, &opts);
        assert!(result.is_ok());
        assert_eq!(state.registers()[Registers::R0], 0x3002);
    }

    #[test]
    fn test_add_immediate() {
        let (mut state, opts) = prepare_test!("tests/add_immediate.obj");
        let result = run(&mut state, &opts);
        assert!(result.is_ok());

        assert_eq!(state.registers()[Registers::R0], 0x7);
        assert_cc_positive(&mut state);

        state.resume();
        run(&mut state, &DEFAULT_OPTS).unwrap();
        assert_eq!(state.registers()[Registers::R0], 0x0);
        assert_cc_zero(&mut state);

        state.resume();
        run(&mut state, &DEFAULT_OPTS).unwrap();
        assert_eq!(state.registers()[Registers::R0], 0xFFFF);
        assert_cc_negative(&mut state);
    }

    #[test]
    fn test_add_register() {
        let (mut state, opts) = prepare_test!("tests/add_register.obj");
        let result = run(&mut state, &opts);
        assert!(result.is_ok());

        assert_eq!(state.registers()[Registers::R0], 0x10);
        assert_cc_positive(&mut state);

        state.resume();
        run(&mut state, &DEFAULT_OPTS).unwrap();
        assert_eq!(state.registers()[Registers::R0], 0x0);
        assert_cc_zero(&mut state);

        state.resume();
        run(&mut state, &DEFAULT_OPTS).unwrap();
        assert_eq!(state.registers()[Registers::R0], 0xFFF0);
        assert_cc_negative(&mut state);
    }

    #[test]
    fn test_ld() {
        let (mut state, opts) = prepare_test!("tests/ld.obj");
        let result = run(&mut state, &opts);
        assert!(result.is_ok());

        assert_eq!(state.registers()[Registers::R0], 0x4242);
        assert_cc_positive(&mut state);

        state.resume();
        run(&mut state, &DEFAULT_OPTS).unwrap();
        assert_eq!(state.registers()[Registers::R0], 0x0);
        assert_cc_zero(&mut state);

        state.resume();
        run(&mut state, &DEFAULT_OPTS).unwrap();
        assert_eq!(state.registers()[Registers::R0], 0xFFFF);
        assert_cc_negative(&mut state);
    }

    #[test]
    fn test_jmp() {
        let (mut state, opts) = prepare_test!("tests/jmp.obj");
        let result = run(&mut state, &opts);
        assert!(result.is_ok(), "{}", result.unwrap_err());

        assert_eq!(state.registers()[Registers::PC], 0x3005);
        assert_eq!(state.registers()[Registers::R0], 1);
    }

    #[test]
    fn test_jsr_immediate() {
        let (mut state, opts) = prepare_test!("tests/jsr_immediate.obj");
        let result = run(&mut state, &opts);
        assert!(result.is_ok(), "{}", result.unwrap_err());

        assert_eq!(state.registers()[Registers::PC], 0x3002);
        assert_eq!(state.registers()[Registers::R0], 1);
        assert_eq!(state.registers()[Registers::R1], 0);
    }

    #[test]
    fn test_jsr_register() {
        let (mut state, opts) = prepare_test!("tests/jsr_register.obj");
        let result = run(&mut state, &opts);
        assert!(result.is_ok(), "{}", result.unwrap_err());

        assert_eq!(state.registers()[Registers::PC], 0x3003);
        assert_eq!(state.registers()[Registers::R7], 0x3002);
        assert_eq!(state.registers()[Registers::R0], 0x3005);
        assert_eq!(state.registers()[Registers::R1], 0);
        assert_eq!(state.registers()[Registers::R2], 1);
    }

    #[test]
    fn test_ldi() {
        let (mut state, opts) = prepare_test!("tests/ldi.obj");
        let result = run(&mut state, &opts);
        assert!(result.is_ok(), "{}", result.unwrap_err());

        assert_eq!(state.registers()[Registers::R0], 0xFFFF);
        assert_cc_negative(&mut state);
    }

    #[test]
    fn test_ldr() {
        let (mut state, opts) = prepare_test!("tests/ldr.obj");
        let result = run(&mut state, &opts);
        assert!(result.is_ok(), "{}", result.unwrap_err());

        assert_eq!(state.registers()[Registers::R0], 0x3004);
        assert_eq!(state.registers()[Registers::R1], 0xFFFF);
        assert_cc_negative(&mut state);
    }

    #[test]
    fn test_and() {
        let (mut state, opts) = prepare_test!("tests/and.obj");
        let result = run(&mut state, &opts);
        assert!(result.is_ok(), "{}", result.unwrap_err());

        assert_eq!(state.registers()[Registers::R2], 0x1200);
        assert_cc_positive(&mut state);

        state.resume();
        run(&mut state, &DEFAULT_OPTS).unwrap();
        assert_eq!(state.registers()[Registers::R2], 0);
        assert_cc_zero(&mut state);

        state.resume();
        run(&mut state, &DEFAULT_OPTS).unwrap();
        assert_eq!(state.registers()[Registers::R2], 15);
        assert_cc_positive(&mut state);
    }

    #[test]
    fn test_not() {
        let (mut state, opts) = prepare_test!("tests/not.obj");
        let result = run(&mut state, &opts);
        assert!(result.is_ok(), "{}", result.unwrap_err());

        assert_eq!(state.registers()[Registers::R1], 0xEDCB);
        assert_cc_negative(&mut state);
    }

    #[test]
    fn test_st() {
        let (mut state, opts) = prepare_test!("tests/st.obj");
        let result = run(&mut state, &opts);
        assert!(result.is_ok(), "{}", result.unwrap_err());

        assert_eq!(state.memory()[0x3003], (-7i16) as u16);
    }

    #[test]
    fn test_sti() {
        let (mut state, opts) = prepare_test!("tests/sti.obj");
        let result = run(&mut state, &opts);
        assert!(result.is_ok(), "{}", result.unwrap_err());

        assert_eq!(state.memory()[0x3003], (-8i16) as u16);
    }

    #[test]
    fn test_str() {
        let (mut state, opts) = prepare_test!("tests/str.obj");
        let result = run(&mut state, &opts);
        assert!(result.is_ok(), "{}", result.unwrap_err());

        assert_eq!(state.memory()[0x3004], (-9i16) as u16);
    }

    #[test]
    fn test_trap() {
        let (mut state, opts) = prepare_test!("tests/trap.obj", 0x200);
        let result = run(&mut state, &opts);
        assert!(result.is_ok(), "{}", result.unwrap_err());

        assert_eq!(state.registers()[Registers::R0], 15);
    }

    #[test]
    fn test_br_backwards() {
        let (mut state, opts) = prepare_test!("tests/br_backwards.obj");
        let result = run(&mut state, &opts);
        assert!(result.is_ok(), "{}", result.unwrap_err());

        assert_eq!(state.registers()[Registers::R0], 10);
    }

    #[test]
    #[ignore] // Interrupts do not currently work
    fn test_rti() {
        let (mut state, opts) = prepare_test!("tests/rti.obj");
        let result = run(&mut state, &opts);
        assert!(result.is_ok(), "{}", result.unwrap_err());

        assert_supervisor_mode(&mut state, false);
        assert_eq!(state.registers()[Registers::R0], (-1i16) as u16);
        assert_cc_negative(&mut state);

        //tx.send(0x42).unwrap(); // Send an interrupt defined in test file
        state.resume();
        run(&mut state, &DEFAULT_OPTS).unwrap();

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
        run(&mut state, &DEFAULT_OPTS).unwrap();
        assert_eq!(state.registers()[Registers::R5], 5);

        // Test that we're returning to the correct position after RTI
        state.resume();
        run(&mut state, &DEFAULT_OPTS).unwrap();
        assert_eq!(state.registers()[Registers::PC], 0x204);
        assert_eq!(state.registers()[Registers::R0], (-2i16) as u16);
    }

    #[test]
    fn test_puts() {
        let display = CapturingDisplay {
            output: RefCell::new("".into()),
        };

        {
            let (mut state, mut opts) = prepare_test!("tests/puts.obj", 0x100);
            opts.peripherals.push(&display);
            let result = run(&mut state, &opts);
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
            let (mut state, mut opts) = prepare_test!("tests/os.obj", 0x200);
            opts.peripherals.push(&display);
            opts.peripherals.push(&keyboard);
            let result = run(&mut state, &opts);
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
}
