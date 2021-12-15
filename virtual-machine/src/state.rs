use num_traits::FromPrimitive;
use std::borrow::BorrowMut;
use std::cell::RefCell;

use std::io::{self, Write};
use std::ops::Index;
use std::ops::IndexMut;
use std::ops::Range;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex, MutexGuard};

const MEM_SIZE: usize = 65535;
const REGISTER_COUNT: usize = 12;

#[derive(FromPrimitive, Debug)]
pub enum Registers {
    R0 = 0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,

    // This is where the internal registers start, i.e. the ones
    // that should not be accessible from the code running in the
    // VM. For convenience, they are currently defined in the same
    // enum as the general purpose registers.
    // TODO: evaluate whether a separation of internal / external
    //       registers makes sense
    PC,
    PSR,
    SSP,
    USP,
}

impl Registers {
    pub fn from_u16_or_panic(index: u16) -> Self {
        match Registers::from_u16(index) {
            Some(x) => x,
            None => panic!("Register with u16 index <0x{:X}> does not exist", index),
        }
    }
}

pub enum ConditionFlags {
    Positive = 1 << 0,
    Zero = 1 << 1,
    Negative = 1 << 2,
}

pub struct VmMemory {
    memory: [u16; MEM_SIZE],
    accesses: RefCell<Vec<u16>>,
}

impl VmMemory {
    pub fn was_accessed(&self, index: u16) -> bool {
        self.accesses.borrow().contains(&index)
    }

    pub fn reset_accesses(&self) {
        self.accesses.borrow_mut().clear();
    }
}

impl Index<u16> for VmMemory {
    type Output = u16;
    fn index(&self, index: u16) -> &u16 {
        self.accesses.borrow_mut().push(index);
        &self.memory[index as usize]
    }
}

impl IndexMut<u16> for VmMemory {
    fn index_mut(&mut self, index: u16) -> &mut u16 {
        self.accesses.borrow_mut().push(index);
        &mut self.memory[index as usize]
    }
}

impl Index<Range<usize>> for VmMemory {
    type Output = [u16];
    fn index(&self, index: Range<usize>) -> &[u16] {
        &self.memory[index]
    }
}

impl IndexMut<Range<usize>> for VmMemory {
    fn index_mut(&mut self, index: Range<usize>) -> &mut [u16] {
        &mut self.memory[index]
    }
}

pub struct VmRegisters {
    registers: [u16; REGISTER_COUNT],
}

impl Index<Registers> for VmRegisters {
    type Output = u16;
    fn index(&self, index: Registers) -> &u16 {
        &self.registers[index as usize]
    }
}

impl IndexMut<Registers> for VmRegisters {
    fn index_mut(&mut self, index: Registers) -> &mut u16 {
        &mut self.registers[index as usize]
    }
}

pub trait VmDisplay {
    fn print(&mut self, u8);
}

pub struct DefaultVmDisplay {}

impl VmDisplay for DefaultVmDisplay {
    fn print(&mut self, c: u8) -> () {
        print!("{}", c as char);
        io::stdout().flush().unwrap();
    }
}

pub trait VmState {
    fn tick(&mut self);
    fn running(&mut self) -> bool;
    fn memory(&self) -> MutexGuard<VmMemory>;
    fn registers(&mut self) -> &mut VmRegisters;
    fn display(&mut self) -> &mut VmDisplay;
    fn increment_pc(&mut self);
    fn resume(&mut self);
    fn interrupt_channel(&mut self) -> &Receiver<u16>;
    fn memory_mutex(&self) -> Arc<Mutex<VmMemory>>;
}

pub struct MyVmState<'a> {
    pub memory: Arc<Mutex<VmMemory>>,
    pub registers: VmRegisters,
    pub display: Box<VmDisplay + 'a>,
    pub running: bool,
    pub error: Option<String>,
    pub interrupt_channel: Receiver<u16>,
}

impl<'a> MyVmState<'a> {
    pub fn new(interrupt_channel: Receiver<u16>) -> Self {
        return MyVmState::new_with_display(Box::new(DefaultVmDisplay {}), interrupt_channel);
    }

    pub fn new_with_display(d: Box<VmDisplay + 'a>, interrupt_channel: Receiver<u16>) -> Self {
        let mut x = Self {
            memory: Arc::new(Mutex::new(VmMemory {
                memory: [0; MEM_SIZE],
                accesses: RefCell::new(vec![]),
            })),
            registers: VmRegisters {
                registers: [0; REGISTER_COUNT],
            },
            running: true,
            display: d,
            error: None,
            interrupt_channel: interrupt_channel,
        };

        // Highest bit of the machine control register MCR indicates
        // whether or not we're running.
        x.memory()[0xFFFE] = 0x8000;

        // The supervisor stack starts at the high-end of the operating
        // system memory segment. This is, as far as I can see, not
        // explicitly defined in the LC3 ISA, but it seems to be implicitly
        // assumed that the internal SSP register is initialized, and since
        // this is not possible from code running inside the VM it needs to
        // happen here.
        // Supervisor stack base is 0x3000, the topmost value of the stack
        // is stored at 0x2FFF (push -> mem[SSP-1] = val)
        x.registers()[Registers::SSP] = 0x3000;

        return x;
    }
}

impl<'a> VmState for MyVmState<'a> {
    fn tick(&mut self) {
        self.memory().reset_accesses();
    }

    fn running(&mut self) -> bool {
        self.memory()[0xFFFE] > 0
    }

    fn memory(&self) -> MutexGuard<VmMemory> {
        self.memory.lock().unwrap()
    }

    fn registers(&mut self) -> &mut VmRegisters {
        &mut self.registers
    }

    fn display(&mut self) -> &mut VmDisplay {
        &mut *self.display
    }

    fn increment_pc(&mut self) {
        self.registers()[Registers::PC] += 1;
    }

    // If the VM is halted, this was caused by a HALT trap
    // We need to increment the PC to resume, otherwise the
    // VM would simply execute HALT again
    fn resume(&mut self) {
        self.memory()[0xFFFE] |= 0x8000
    }

    fn interrupt_channel(&mut self) -> &Receiver<u16> {
        &self.interrupt_channel
    }

    fn memory_mutex(&self) -> Arc<Mutex<VmMemory>> {
        let foo = &self.memory;
        Arc::clone(foo)
    }
}
