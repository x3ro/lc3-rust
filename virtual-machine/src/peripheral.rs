use std::cell::RefCell;
use std::io::{self, Write};
use VmState;

pub trait Peripheral {
    fn run(&self, state: &mut VmState);
}

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

pub struct TerminalKeyboard {}
impl Peripheral for TerminalKeyboard {
    fn run(&self, _state: &mut VmState) {
        todo!()
    }
}
