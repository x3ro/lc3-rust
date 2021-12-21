use num_traits::FromPrimitive;
use std::cell::RefCell;

use crate::Peripheral;
use std::ops::Index;
use std::ops::IndexMut;
use std::ops::Range;

const MEM_SIZE: usize = 65535;
const REGISTER_COUNT: usize = 12;

#[derive(FromPrimitive, Debug, Clone)]
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

impl Index<&Registers> for VmRegisters {
    type Output = u16;
    fn index(&self, index: &Registers) -> &u16 {
        // TODO: This is not very efficient (cloning here reduced performance by ~5%)
        //      Is there another way we can index into the `registers` here, without having to
        //      copy the value?
        &self.registers[index.clone() as usize]
    }
}

impl IndexMut<&Registers> for VmRegisters {
    fn index_mut(&mut self, index: &Registers) -> &mut u16 {
        &mut self.registers[index.clone() as usize]
    }
}

impl Index<Registers> for VmRegisters {
    type Output = u16;
    fn index(&self, index: Registers) -> &u16 {
        &self[&index]
    }
}

impl IndexMut<Registers> for VmRegisters {
    fn index_mut(&mut self, index: Registers) -> &mut u16 {
        &mut self[&index]
    }
}

pub struct VmState<'a> {
    pub memory: VmMemory,
    pub registers: VmRegisters,
    pub running: bool,
    pub error: Option<String>,
    pub peripherals: Vec<&'a dyn Peripheral>,
}

impl<'a> VmState<'a> {
    pub fn new() -> Self {
        let mut x = Self {
            memory: VmMemory {
                memory: [0; MEM_SIZE],
                accesses: RefCell::new(vec![]),
            },
            registers: VmRegisters {
                registers: [0; REGISTER_COUNT],
            },
            running: true,
            error: None,
            peripherals: vec![],
        };

        // Highest bit of the machine control register MCR indicates
        // whether or not we're running.
        x.memory_mut()[0xFFFE] = 0x8000;

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

impl<'a> VmState<'a> {
    pub fn tick(&self) {
        self.memory.reset_accesses();
    }

    pub fn running(&self) -> bool {
        self.memory[0xFFFE] > 0
    }

    pub fn memory_mut(&mut self) -> &mut VmMemory {
        &mut self.memory
    }

    pub fn memory(&self) -> &VmMemory {
        &self.memory
    }

    pub fn registers(&mut self) -> &mut VmRegisters {
        &mut self.registers
    }

    pub fn increment_pc(&mut self) {
        self.registers()[Registers::PC] += 1;
    }

    pub fn set_pc(&mut self, pc: u16) {
        self.registers()[Registers::PC] = pc;
    }

    pub fn resume(&mut self) {
        self.memory[0xFFFE] |= 0x8000
    }
}
