use std::ops::Index;
use std::ops::IndexMut;

const MEM_SIZE: usize = 65535;
const REGISTER_COUNT: usize = 10;

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

pub enum ConditionFlags {
    Positive = 1 << 0,
    Zero = 1 << 1,
    Negative = 1 << 2,
}

pub struct VmMemory {
    pub memory: [u16; MEM_SIZE],
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


pub trait VmState {
    fn get_mem(&self, u16) -> u16;
    fn set_mem(&mut self, u16, u16);
    fn set_mem_range(&mut self, std::ops::Range<usize>, &[u16]);
    fn get_reg(&self, Registers) -> u16;
    fn set_reg(&mut self, Registers, u16);
    fn set_reg1(&mut self, usize, u16);
    fn print(&self, u8);
    fn halt(&mut self);
    fn running(&self) -> bool;
    
}

pub struct MyVmState {
    pub memory: VmMemory,
    pub registers: [u16; REGISTER_COUNT],
    pub running: bool,
}

impl MyVmState {
    pub fn new() -> Self {
        return Self {
            memory: VmMemory{memory: [0; MEM_SIZE]},
            registers: [0; REGISTER_COUNT],
            running: true,
        };
    }
}

impl VmState for MyVmState {
    fn get_mem(&self, i: u16) -> u16 {
        self.memory[i]
    }

    fn set_mem(&mut self, i: u16, val: u16) -> () {
        self.memory[i] = val
    }

    fn set_mem_range(&mut self, range: std::ops::Range<usize>, vals: &[u16]) -> () {
        self.memory.memory[range].copy_from_slice(vals);
    }

    fn get_reg(&self, r: Registers) -> u16 {
        self.registers[r as usize]
    }

    fn set_reg(&mut self, r: Registers, val: u16) {
        self.registers[r as usize] = val
    }

    fn set_reg1(&mut self, r: usize, val: u16) {
        self.registers[r] = val
    }

    fn print(&self, c: u8) -> () {
        print!("{}", c as char)
    }

    fn halt(&mut self) {
        self.running = false
    }

    fn running(&self) -> bool {
        self.running
    }
}