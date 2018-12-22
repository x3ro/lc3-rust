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

pub struct VmState<'a> {
    pub memory: [u16; MEM_SIZE],
    pub registers: [u16; REGISTER_COUNT],
    pub running: bool,
    pub print: Box<FnMut(u8) -> () + 'a>,
}


impl<'a> VmState<'a> {
    pub fn new() -> Self {
        return VmState {
            memory: [0; MEM_SIZE],
            registers: [0; REGISTER_COUNT],
            running: true,
            print: Box::new(|x| print!("{}", x as char)),
        };
    }
}