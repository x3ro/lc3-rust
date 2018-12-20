use num_traits::FromPrimitive;

use state::VmState;
use state::Registers;
use util::sign_extend;

#[derive(FromPrimitive)]
pub enum Opcode {
    BR   = 0x0, /* branch */
    ADD  = 0x1, /* add  */
    LD   = 0x2, /* load */
    ST   = 0x3, /* store */
    JS   = 0x4, /* jump register */
    AND  = 0x5, /* bitwise and */
    LDR  = 0x6, /* load register */
    STR  = 0x7, /* store register */
    RTI  = 0x8, /* unused */
    NOT  = 0x9, /* bitwise not */
    LDI  = 0xA, /* load indirect */
    STI  = 0xB, /* store indirect */
    JMP  = 0xC, /* jump */
    RES  = 0xD, /* reserved (unused) */
    LEA  = 0xE, /* load effective address */
    TRAP = 0xF, /* execute trap */
}

impl Opcode {
    pub fn from_instruction(instruction: u16) -> Self {
        // The upper three bits of an instruction are the opcode
        let opcode = instruction >> 12;
        match Opcode::from_u16(opcode) {
            Some(x) => x,
            None => panic!("Could not instantiate opcode from <0x{:X}>", opcode)
        }
    }
}

pub fn op_lea(state: &mut VmState, pc: usize) {
    let instruction = state.memory[pc];
    let dr = ((instruction >> 9) & 0b111) as usize;
    let imm = sign_extend(instruction & 0b111111111, 9);
    state.registers[dr] = ((pc+1) as u16) + imm;
    state.registers[Registers::PC as usize] += 1
    // TODO: set condition flags!

    // println!("imm <0x{:x}>", imm);
    // println!("reg <{}> val <0x{:x}>", dr, state.registers[dr]);
    // println!("value at address <0x{:x}> is <0x{:x}>", state.registers[dr], state.memory[state.registers[dr] as usize]);
}

pub fn op_trap(state: &mut VmState, pc: usize) {
    // R7 is where we jump to upon completion of the handler. In the current implementation,
    // where we handle the traps in the VM, setting this is not necessary, but it's in the spec
    state.registers[Registers::R7 as usize] = (pc+1) as u16;

    let trap_type = state.memory[pc] & 0b1111_1111;
    match trap_type {
        0x22 => trap_puts(state),
        0x25 => trap_halt(state),
        _ => panic!("Unimplemented trap vector <0x{:x}> at pc <0x{:x}>", trap_type, pc),
    }

    state.registers[Registers::PC as usize] = state.registers[Registers::R7 as usize]
}

fn trap_halt(state: &mut VmState) {
    state.running = false
}

fn trap_puts(state: &mut VmState) {
    let mut start = state.registers[Registers::R0 as usize] as usize;
    while state.memory[start] != 0 {
        print!("{}", ((state.memory[start] & 0xFF) as u8) as char);
        start += 1;
    }
}