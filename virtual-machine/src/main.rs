use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time;
use std::time::Duration;

type Result<T> = std::result::Result<T, String>;

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

#[macro_use]
extern crate num_derive;
extern crate num_traits;
extern crate termios;
use termios::{tcsetattr, Termios, ECHO, ICANON, TCSANOW};

extern crate clap;
use clap::{App, Arg};

#[macro_use]
mod util;
mod opcodes;
mod parser;
mod state;
mod peripheral;

use opcodes::*;
use parser::Instruction;
use peripheral::{Peripheral, TerminalDisplay};
use state::MyVmState;
use state::Registers;
use state::VmState;

struct VmOptions<'a> {
    pub throttle: Option<Duration>,
    pub peripherals: Vec<&'a Peripheral>,
}

fn load_object_file(filename: &str, state: &mut VmState) -> io::Result<()> {
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

fn run(state: &mut VmState, opts: &VmOptions) -> Result<()> {
    while state.running() {
        execute_next_instruction(state)?;

        for p in &opts.peripherals {
            p.run(state);
        }

        if opts.throttle.is_some() {
            thread::sleep(opts.throttle.unwrap());
        }
    }
    Ok(())
}

fn run_file(
    state: &mut VmState,
    filenames: Vec<&str>,
    start_pc: u16,
    opts: &VmOptions,
) -> io::Result<()> {
    for filename in filenames {
        load_object_file(filename, state)?;
    }

    state.registers()[Registers::PC] = start_pc;
    match run(state, opts) {
        Ok(x) => Ok(x),
        Err(x) => Err(io::Error::new(io::ErrorKind::Other, x)),
    }
}

fn start_input_thread(tx: Sender<u16>) -> std::thread::JoinHandle<()> {
    let handle = thread::spawn(move || {
        let stdin = 0; // couldn't get std::os::unix::io::FromRawFd to work
                       // on /dev/stdin or /dev/tty
        let termios = Termios::from_fd(stdin).unwrap();
        let mut new_termios = termios.clone(); // make a mutable copy of termios
                                               // that we will modify
        new_termios.c_lflag &= !(ICANON | ECHO); // no echo and canonical mode
        tcsetattr(stdin, TCSANOW, &mut new_termios).unwrap();

        let stdin = io::stdin();
        let mut handle = stdin.lock();
        let mut buffer: [u8; 1] = [0; 1];

        debug!("Starting input thread");

        loop {
            handle.read_exact(&mut buffer).unwrap();
            debug!("Input thread received 0x{:x}", buffer[0] as u16);
            tx.send(buffer[0] as u16);
        }
    });

    handle
}

fn main() -> io::Result<()> {
    pretty_env_logger::init();

    let matches = App::new("My Super Program")
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

    let filenames: Vec<_> = matches.values_of("programs").unwrap().collect();
    let entry_point = matches.value_of("entry_point").unwrap_or("0x3000");
    let throttle = matches
        .value_of("throttle")
        .and_then(|x| x.parse::<u64>().ok())
        .map(Duration::from_millis);

    let e = u16::from_str_radix(entry_point.trim_start_matches("0x"), 16).unwrap();

    let (_tx, rx): (Sender<u16>, Receiver<u16>) = mpsc::channel();
    let mut state = MyVmState::new(rx);

    let (input_tx, input_rx): (Sender<u16>, Receiver<u16>) = mpsc::channel();
    let in_handle = start_input_thread(input_tx); // TODO handle


    let display = TerminalDisplay{};
    let opts = VmOptions {
        throttle,
        peripherals: vec![&display]
    };

    let res = match run_file(&mut state, filenames, e, &opts) {
        Ok(_) => Ok(()),
        Err(x) => Err(x),
    };

    res
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use peripheral::CapturingDisplay;
    use super::*;
    use state::ConditionFlags;


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

    fn assert_supervisor_mode(state: &mut VmState, enabled: bool) {
        if enabled {
            assert_eq!(state.registers()[Registers::PSR] & 0b1000_0000_0000_0000, 0b1000_0000_0000_0000);
        } else {
            assert_eq!(state.registers()[Registers::PSR] & 0b1000_0000_0000_0000, 0);
        }
    }


    // Tests

    const DEFAULT_OPTS: VmOptions = VmOptions{throttle:None, peripherals: vec![]};

    #[test]
    fn test_br() {
        let (_tx, rx): (Sender<u16>, Receiver<u16>) = mpsc::channel();
        let mut state = MyVmState::new(rx);
        let result = run_file(&mut state, vec!("tests/br.obj"), 0x3000, &DEFAULT_OPTS);
        assert!(result.is_ok());
        assert_eq!(state.registers()[Registers::R2], 1);

        state.resume();
        run(&mut state, &DEFAULT_OPTS).unwrap();
        assert_eq!(state.registers()[Registers::R2], 2);

        state.resume();
        run(&mut state, &DEFAULT_OPTS).unwrap();
        assert_eq!(state.registers()[Registers::R2], 3);

        state.resume();
        run(&mut state, &DEFAULT_OPTS).unwrap();
        assert_eq!(state.registers()[Registers::R2], 4);

        state.resume();
        run(&mut state, &DEFAULT_OPTS).unwrap();
        assert_eq!(state.registers()[Registers::R2], 5);

        state.resume();
        run(&mut state, &DEFAULT_OPTS).unwrap();
        assert_eq!(state.registers()[Registers::R2], 6);

        state.resume();
        run(&mut state, &DEFAULT_OPTS).unwrap();
        assert_eq!(state.registers()[Registers::R2], 7);

        state.resume();
        run(&mut state, &DEFAULT_OPTS).unwrap();
        assert_eq!(state.registers()[Registers::R2], 8);

        state.resume();
        run(&mut state, &DEFAULT_OPTS).unwrap();
        assert_eq!(state.registers()[Registers::R2], 9);

        state.resume();
        run(&mut state, &DEFAULT_OPTS).unwrap();
        assert_eq!(state.registers()[Registers::R2], 10);

        state.resume();
        run(&mut state, &DEFAULT_OPTS).unwrap();
        assert_eq!(state.registers()[Registers::R2], 11);

        state.resume();
        run(&mut state, &DEFAULT_OPTS).unwrap();
        assert_eq!(state.registers()[Registers::R2], 12);

        // This is only incremented on wrong branch, so should remain zero
        assert_eq!(state.registers()[Registers::R1], 0x0);
    }

    #[test]
    fn test_lea() {
        let (_tx, rx): (Sender<u16>, Receiver<u16>) = mpsc::channel();
        let mut state = MyVmState::new(rx);
        let result = run_file(&mut state, vec!("tests/lea.obj"), 0x3000, &DEFAULT_OPTS);
        assert!(result.is_ok());
        assert_eq!(state.registers()[Registers::R0], 0x3002);
    }

    #[test]
    fn test_add_immediate() {
        let (_tx, rx): (Sender<u16>, Receiver<u16>) = mpsc::channel();
        let mut state = MyVmState::new(rx);
        let result = run_file(&mut state, vec!("tests/add_immediate.obj"), 0x3000, &DEFAULT_OPTS);
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
        let (_tx, rx): (Sender<u16>, Receiver<u16>) = mpsc::channel();
        let mut state = MyVmState::new(rx);
        let result = run_file(&mut state, vec!("tests/add_register.obj"), 0x3000, &DEFAULT_OPTS);
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
        let (_tx, rx): (Sender<u16>, Receiver<u16>) = mpsc::channel();
        let mut state = MyVmState::new(rx);
        let result = run_file(&mut state, vec!("tests/ld.obj"), 0x3000, &DEFAULT_OPTS);
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
        let (_tx, rx): (Sender<u16>, Receiver<u16>) = mpsc::channel();
        let mut state = MyVmState::new(rx);
        let result = run_file(&mut state, vec!("tests/jmp.obj"), 0x3000, &DEFAULT_OPTS);
        assert!(result.is_ok(), "{}", result.unwrap_err());

        assert_eq!(state.registers()[Registers::PC], 0x3005);
        assert_eq!(state.registers()[Registers::R0], 1);
    }

    #[test]
    fn test_jsr_immediate() {
        let (_tx, rx): (Sender<u16>, Receiver<u16>) = mpsc::channel();
        let mut state = MyVmState::new(rx);
        let result = run_file(&mut state, vec!("tests/jsr_immediate.obj"), 0x3000, &DEFAULT_OPTS);
        assert!(result.is_ok(), "{}", result.unwrap_err());

        assert_eq!(state.registers()[Registers::PC], 0x3002);
        assert_eq!(state.registers()[Registers::R0], 1);
        assert_eq!(state.registers()[Registers::R1], 0);
    }

    #[test]
    fn test_jsr_register() {
        let (_tx, rx): (Sender<u16>, Receiver<u16>) = mpsc::channel();
        let mut state = MyVmState::new(rx);
        let result = run_file(&mut state, vec!("tests/jsr_register.obj"), 0x3000, &DEFAULT_OPTS);
        assert!(result.is_ok(), "{}", result.unwrap_err());

        assert_eq!(state.registers()[Registers::PC], 0x3003);
        assert_eq!(state.registers()[Registers::R7], 0x3002);
        assert_eq!(state.registers()[Registers::R0], 0x3005);
        assert_eq!(state.registers()[Registers::R1], 0);
        assert_eq!(state.registers()[Registers::R2], 1);
    }

    #[test]
    fn test_ldi() {
        let (_tx, rx): (Sender<u16>, Receiver<u16>) = mpsc::channel();
        let mut state = MyVmState::new(rx);
        let result = run_file(&mut state, vec!("tests/ldi.obj"), 0x3000, &DEFAULT_OPTS);
        assert!(result.is_ok(), "{}", result.unwrap_err());

        assert_eq!(state.registers()[Registers::R0], 0xFFFF);
        assert_cc_negative(&mut state);
    }

    #[test]
    fn test_ldr() {
        let (_tx, rx): (Sender<u16>, Receiver<u16>) = mpsc::channel();
        let mut state = MyVmState::new(rx);
        let result = run_file(&mut state, vec!("tests/ldr.obj"), 0x3000, &DEFAULT_OPTS);
        assert!(result.is_ok(), "{}", result.unwrap_err());

        assert_eq!(state.registers()[Registers::R0], 0x3004);
        assert_eq!(state.registers()[Registers::R1], 0xFFFF);
        assert_cc_negative(&mut state);
    }

    #[test]
    fn test_and() {
        let (_tx, rx): (Sender<u16>, Receiver<u16>) = mpsc::channel();
        let mut state = MyVmState::new(rx);
        let result = run_file(&mut state, vec!("tests/and.obj"), 0x3000, &DEFAULT_OPTS);
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
        let (_tx, rx): (Sender<u16>, Receiver<u16>) = mpsc::channel();
        let mut state = MyVmState::new(rx);
        let result = run_file(&mut state, vec!("tests/not.obj"), 0x3000, &DEFAULT_OPTS);
        assert!(result.is_ok(), "{}", result.unwrap_err());

        assert_eq!(state.registers()[Registers::R1], 0xEDCB);
        assert_cc_negative(&mut state);
    }

    #[test]
    fn test_st() {
        let (_tx, rx): (Sender<u16>, Receiver<u16>) = mpsc::channel();
        let mut state = MyVmState::new(rx);
        let result = run_file(&mut state, vec!("tests/st.obj"), 0x3000, &DEFAULT_OPTS);
        assert!(result.is_ok(), "{}", result.unwrap_err());

        assert_eq!(state.memory()[0x3003], (-7i16) as u16);
    }

    #[test]
    fn test_sti() {
        let (_tx, rx): (Sender<u16>, Receiver<u16>) = mpsc::channel();
        let mut state = MyVmState::new(rx);
        let result = run_file(&mut state, vec!("tests/sti.obj"), 0x3000, &DEFAULT_OPTS);
        assert!(result.is_ok(), "{}", result.unwrap_err());

        assert_eq!(state.memory()[0x3003], (-8i16) as u16);
    }

    #[test]
    fn test_str() {
        let (_tx, rx): (Sender<u16>, Receiver<u16>) = mpsc::channel();
        let mut state = MyVmState::new(rx);
        let result = run_file(&mut state, vec!("tests/str.obj"), 0x3000, &DEFAULT_OPTS);
        assert!(result.is_ok(), "{}", result.unwrap_err());

        assert_eq!(state.memory()[0x3004], (-9i16) as u16);
    }

    #[test]
    fn test_trap() {
        let (_tx, rx): (Sender<u16>, Receiver<u16>) = mpsc::channel();
        let mut state = MyVmState::new(rx);
        let result = run_file(&mut state, vec!("tests/trap.obj"), 0x200, &DEFAULT_OPTS);
        assert!(result.is_ok(), "{}", result.unwrap_err());

        assert_eq!(state.registers()[Registers::R0], 15);
    }

    #[test]
    fn test_br_backwards() {
        let (_tx, rx): (Sender<u16>, Receiver<u16>) = mpsc::channel();
        let mut state = MyVmState::new(rx);
        let result = run_file(&mut state, vec!("tests/br_backwards.obj"), 0x3000, &DEFAULT_OPTS);
        assert!(result.is_ok(), "{}", result.unwrap_err());

        assert_eq!(state.registers()[Registers::R0], 10);
    }

    #[test]
    fn test_rti() {
        let (tx, rx): (Sender<u16>, Receiver<u16>) = mpsc::channel();
        let mut state = MyVmState::new(rx);
        let result = run_file(&mut state, vec!("tests/rti.obj"), 0x200, &DEFAULT_OPTS);
        assert!(result.is_ok(), "{}", result.unwrap_err());

        assert_supervisor_mode(&mut state, false);
        assert_eq!(state.registers()[Registers::R0], (-1i16) as u16);
        assert_cc_negative(&mut state);

        tx.send(0x42).unwrap(); // Send an interrupt defined in test file
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

    // #[test]
    // fn test_memory_mapped_io() {
    //     let (_tx, rx): (Sender<u16>, Receiver<u16>) = mpsc::channel();
    //     let mut state = MyVmState::new(rx);
    //     let mutex = state.memory_mutex();
    //
    //     // This thread simulates a memory-mapped I/O device that, upon
    //     // writing something != 0 into 0xFE01 sets 0xFE00 to 42 and then
    //     // terminates.
    //     let handle = thread::spawn(move || {
    //         let one_millis = time::Duration::from_millis(1);
    //         loop {
    //             let memory = mutex.lock().unwrap();
    //             if memory[0xFE01] > 0 {
    //                 break;
    //             }
    //             thread::sleep(one_millis);
    //         }
    //         let mut memory = mutex.lock().unwrap();
    //         memory[0xFE00] = 42;
    //     });
    //
    //     let result = run_file(&mut state, vec!("tests/memory_mapped_io.obj"), 0x3000, &DEFAULT_OPTS);
    //     assert!(result.is_ok(), "{}", result.unwrap_err());
    //     handle.join().unwrap();
    //
    //     assert_eq!(state.memory()[0xFE00], 42);
    //     assert_eq!(state.memory()[0xFE01], 1);
    //     assert_eq!(state.registers()[Registers::R0], 42);
    // }

    #[test]
    fn test_puts() {
        let mut display = CapturingDisplay { output: RefCell::new("".into()) };

        {
            let opts = VmOptions {
                throttle: None,
                peripherals: vec![&display],
            };

            let (_tx, rx): (Sender<u16>, Receiver<u16>) = mpsc::channel();
            let mut state = MyVmState::new( rx);
            let result = run_file(&mut state, vec!("tests/puts.obj"), 0x100, &opts);
            assert!(result.is_ok());
        }

        assert_eq!("Hello World!\n", display.output.borrow().as_str());
    }
}
