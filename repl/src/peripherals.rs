use std::io::prelude::*;
use std::io::{self, Write};

use lc3vm::state::VmMemory;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

//extern crate termios;
use lc3vm::peripheral::*;
use termios::{tcsetattr, Termios, ECHO, ICANON, TCSANOW};

pub struct TerminalDisplay {}
impl Peripheral for TerminalDisplay {
    fn run(&self, memory: &mut VmMemory) {
        // Setting bit[15] on the DSR indicates the display is ready
        // We can always set this, since we're running in sync with the VM
        // (that is, before a new VM instruction we're always done printing)
        memory[OS_DSR] = 0b1000_0000_0000_0000;

        let character = (memory[OS_DDR] & 0xFF) as u8;
        if character == 0 {
            return;
        }

        print!("{}", character as char);
        io::stdout().flush().unwrap();

        memory[OS_DDR] = 0;
    }
}

pub struct TerminalKeyboard {
    rx: Receiver<char>,
}

impl TerminalKeyboard {
    pub fn new() -> TerminalKeyboard {
        let (tx, rx): (Sender<char>, Receiver<char>) = mpsc::channel();
        Self::start_input_thread(tx);
        return TerminalKeyboard { rx };
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
                tx.send(buffer[0] as char).unwrap();
            }
        });

        handle
    }
}

impl Peripheral for TerminalKeyboard {
    fn run(&self, memory: &mut VmMemory) {
        let kbdr_access = memory.was_accessed(OS_KBDR);
        if kbdr_access {
            trace!("Resetting KBSR because KBDR was accessed last tick");
            memory[OS_KBSR] = 0x0;
            return;
        }

        let data = self.rx.try_recv().ok();
        if let Some(char) = data {
            // Setting bit[15] on the KBSR indicates the a new character is ready
            memory[OS_KBSR] = 0b1000_0000_0000_0000;
            memory[OS_KBDR] = char as u16;
            trace!("Wrote character '{:?}' into memory", char);
        }
    }
}
