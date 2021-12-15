use std::cell::{Ref, RefCell};
use std::io::prelude::*;
use std::io::{self, stdout, Write};
use std::ops::Add;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

extern crate termios;
use self::termios::{tcsetattr, Termios, ECHO, ICANON, TCSANOW};

use VmState;

pub trait Peripheral {
    fn run(&self, state: &mut VmState);
}

// Keyboard status and keyboard data register
const OS_KBSR: u16 = 0xFE00;
const OS_KBDR: u16 = 0xFE02;

// The LC3 I/O model described in the ISA is polling-based.
// In order to give the VM application time to process keyboard input, we have to wait
// a couple of instructions until we write the next character into memory. This constant
// indicates how many instructions we wait.
const KEYBOARD_UPDATE_SPEED: u8 = 20;

// Display status and display data register
const OS_DSR: u16 = 0xFE04;
const OS_DDR: u16 = 0xFE06;

pub struct TerminalDisplay {}
impl Peripheral for TerminalDisplay {
    fn run(&self, state: &mut VmState) {
        // Setting bit[15] on the DSR indicates the display is ready
        // We can always set this, since we're running in sync with the VM
        // (that is, before a new VM instruction we're always done printing)
        state.memory()[OS_DSR] = 0b1000_0000_0000_0000;

        let character = (state.memory()[OS_DDR] & 0xFF) as u8;
        if character == 0 {
            return;
        }

        print!("{}", character as char);
        io::stdout().flush().unwrap();

        state.memory()[OS_DDR] = 0;
    }
}

pub struct CapturingDisplay {
    pub output: RefCell<String>,
}

impl Peripheral for CapturingDisplay {
    fn run(&self, state: &mut VmState) {
        // Setting bit[15] on the DSR indicates the display is ready
        // We can always set this, since we're running in sync with the VM
        // (that is, before a new VM instruction we're always done printing)
        state.memory()[OS_DSR] = 0b1000_0000_0000_0000;

        let character = (state.memory()[OS_DDR] & 0xFF) as u8;
        if character == 0 {
            return;
        }

        self.output.borrow_mut().push(character as char);
        state.memory()[OS_DDR] = 0;
    }
}

pub struct TerminalKeyboard {
    rx: Receiver<char>,
    counter: RefCell<u8>,
}

impl TerminalKeyboard {
    pub fn new() -> TerminalKeyboard {
        let (tx, rx): (Sender<char>, Receiver<char>) = mpsc::channel();
        Self::start_input_thread(tx);
        return TerminalKeyboard {
            rx,
            counter: RefCell::new(KEYBOARD_UPDATE_SPEED),
        };
    }

    fn start_input_thread(tx: Sender<char>) -> std::thread::JoinHandle<()> {
        let handle = thread::spawn(move || {
            // couldn't get std::os::unix::io::FromRawFd to work
            // on /dev/stdin or /dev/tty
            let stdin = 0;
            let termios = Termios::from_fd(stdin).unwrap();
            let mut new_termios = termios.clone(); // make a mutable copy of termios
                                                   // that we will modify
            new_termios.c_lflag &= !(ICANON | ECHO); // no echo and canonical mode
            tcsetattr(stdin, TCSANOW, &mut new_termios).unwrap();

            let stdin = io::stdin();
            let mut handle = stdin.lock();
            let mut buffer: [u8; 1] = [0; 1];

            debug!("Starting keyboard input loop");

            loop {
                handle.read_exact(&mut buffer).unwrap();
                tx.send(buffer[0] as char);
            }
        });

        handle
    }
}

impl Peripheral for TerminalKeyboard {
    fn run(&self, state: &mut VmState) {
        let kbdr_access = state.memory().was_accessed(OS_KBDR);
        if kbdr_access {
            warn!("Resetting KBSR because KBDR was accessed last tick");
            state.memory()[OS_KBSR] = 0x0;
            return;
        }

        let ref mut counter = *self.counter.borrow_mut();
        if *counter > 0 {
            *counter -= 1;
            return;
        }
        *counter = KEYBOARD_UPDATE_SPEED;

        let data = self.rx.try_recv().ok();
        if let Some(char) = data {
            state.memory()[OS_KBDR] = char as u16;
            state.memory()[OS_KBSR] = 0b1000_0000_0000_0000;
            debug!("Wrote character '{:?}' into memory", char);
        }
    }
}

pub struct AutomatedKeyboard {
    output: RefCell<String>,
    counter: RefCell<u8>,
}

impl AutomatedKeyboard {
    pub fn new(output: String) -> Self {
        AutomatedKeyboard {
            counter: RefCell::new(KEYBOARD_UPDATE_SPEED),
            output: RefCell::new(output.chars().rev().collect()),
        }
    }
}

impl Peripheral for AutomatedKeyboard {
    fn run(&self, state: &mut VmState) {
        let kbdr_access = state.memory().was_accessed(OS_KBDR);
        if kbdr_access {
            warn!("Resetting KBSR because KBDR was accessed last tick");
            state.memory()[OS_KBSR] = 0x0;
            return;
        }

        let ref mut counter = *self.counter.borrow_mut();
        if *counter > 0 {
            *counter -= 1;
            return;
        }
        *counter = KEYBOARD_UPDATE_SPEED;

        let kbsr_access = state.memory().was_accessed(OS_KBSR);
        if kbsr_access {
            if let Some(char) = self.output.borrow_mut().pop() {
                state.memory()[OS_KBDR] = char as u16;
                state.memory()[OS_KBSR] = 0b1000_0000_0000_0000;
                warn!("Wrote character '{:?}' into memory", char);
            }
        }
    }
}
