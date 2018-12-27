use num_traits::FromPrimitive;

use std::ops::Index;
use std::ops::IndexMut;
use std::ops::Range;

const MEM_SIZE: usize = 65535;
const REGISTER_COUNT: usize = 10;

#[derive(FromPrimitive)]
pub enum Registers {
    R0 = 0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    PC,
    COND,
}

impl Registers {
    pub fn from_usize_or_panic(index: usize) -> Self {
        match Registers::from_usize(index) {
            Some(x) => x,
            None => panic!("Register with index <0x{:X}> does not exist", index)
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
}

impl Index<u16> for VmMemory {
    type Output = u16;
    fn index(&self, index: u16) -> &u16 {
        &self.memory[index as usize]
    }
}

impl IndexMut<u16> for VmMemory {
    fn index_mut(&mut self, index: u16) -> &mut u16 {
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
        print!("{}", c as char)
    }
}

pub trait VmState {
    fn halt(&mut self);
    fn running(&self) -> bool;
    fn memory(&mut self) -> &mut VmMemory;
    fn registers(&mut self) -> &mut VmRegisters;
    fn display(&mut self) -> &mut VmDisplay;
}

pub struct MyVmState<'a> {
    pub memory: VmMemory,
    pub registers: VmRegisters,
    pub display: Box<VmDisplay + 'a>,
    pub running: bool,
}

impl<'a> MyVmState<'a> {
    pub fn new() -> Self {
        return Self {
            memory: VmMemory{memory: [0; MEM_SIZE]},
            registers: VmRegisters {registers: [0; REGISTER_COUNT]},
            running: true,
            display: Box::new(DefaultVmDisplay{})
        };
    }

    pub fn new_with_display(d: Box<VmDisplay + 'a>) -> Self {
        return Self {
            memory: VmMemory{memory: [0; MEM_SIZE]},
            registers: VmRegisters {registers: [0; REGISTER_COUNT]},
            running: true,
            display: d
        };
    }
}

impl<'a> VmState for MyVmState<'a> {
    fn halt(&mut self) {
        self.running = false
    }

    fn running(&self) -> bool {
        self.running
    }

    fn memory(&mut self) -> &mut VmMemory {
        &mut self.memory
    }

    fn registers(&mut self) -> &mut VmRegisters {
        &mut self.registers
    }

    fn display(&mut self) -> &mut VmDisplay {
        &mut *self.display
    }
}