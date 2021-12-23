use std::cell::RefCell;

use crate::state::VmMemory;

pub trait Peripheral {
    fn run(&self, memory: &mut VmMemory);
}

// Keyboard status and keyboard data register
pub const OS_KBSR: u16 = 0xFE00;
pub const OS_KBDR: u16 = 0xFE02;

// The LC3 I/O model described in the ISA is polling-based.
// In order to give the VM application time to process keyboard input, we have to wait
// a couple of instructions until we write the next character into memory. This constant
// indicates how many instructions we wait.
pub const KEYBOARD_UPDATE_SPEED: u8 = 20;

// Display status and display data register
pub const OS_DSR: u16 = 0xFE04;
pub const OS_DDR: u16 = 0xFE06;

pub struct CapturingDisplay {
    pub output: RefCell<String>,
}

impl Peripheral for CapturingDisplay {
    fn run(&self, memory: &mut VmMemory) {
        // Setting bit[15] on the DSR indicates the display is ready
        // We can always set this, since we're running in sync with the VM
        // (that is, before a new VM instruction we're always done printing)
        memory[OS_DSR] = 0b1000_0000_0000_0000;

        let character = (memory[OS_DDR] & 0xFF) as u8;
        if character == 0 {
            return;
        }

        self.output.borrow_mut().push(character as char);
        memory[OS_DDR] = 0;
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
    fn run(&self, memory: &mut VmMemory) {
        let kbdr_access = memory.was_accessed(OS_KBDR);
        if kbdr_access {
            trace!("Resetting KBSR because KBDR was accessed last tick");
            memory[OS_KBSR] = 0x0;
            return;
        }

        let ref mut counter = *self.counter.borrow_mut();
        if *counter > 0 {
            *counter -= 1;
            return;
        }
        *counter = KEYBOARD_UPDATE_SPEED;

        let kbsr_access = memory.was_accessed(OS_KBSR);
        if kbsr_access {
            if let Some(char) = self.output.borrow_mut().pop() {
                // Setting bit[15] on the KBSR indicates the a new character is ready
                memory[OS_KBSR] = 0b1000_0000_0000_0000;
                memory[OS_KBDR] = char as u16;
                trace!("Wrote character '{:?}' into memory", char);
            }
        }
    }
}
