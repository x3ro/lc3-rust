use num_traits::FromPrimitive;

use state::VmState;
use state::Registers;
use state::ConditionFlags;
use util::sign_extend;
use util::binary_add;

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

fn update_condition_codes(state: &mut VmState, value: u16) {
    state.registers()[Registers::PSR] &= 0b1111_1111_1111_1000;
    match value as i16 {
        x if x < 0 => state.registers()[Registers::PSR] |= ConditionFlags::Negative as u16,
        x if x > 0 => state.registers()[Registers::PSR] |= ConditionFlags::Positive as u16,
        _ => state.registers()[Registers::PSR] |= ConditionFlags::Zero as u16,
    }
}

pub fn op_add(state: &mut VmState, pc: usize) {
    let instruction = state.memory()[pc as u16];
    let dr = Registers::from_u16_or_panic((instruction >> 9) & 0b111);
    let sr1 = Registers::from_u16_or_panic((instruction >> 6) & 0b111);

    if ((instruction >> 5) & 0x1) == 0 {
        let sr2 = Registers::from_u16_or_panic(instruction & 0b111);
        let result = binary_add(state.registers()[sr1], state.registers()[sr2]);
        state.registers()[dr] = result;
        update_condition_codes(state, result);
    } else {
        let imm = sign_extend((instruction & 0b11111) as u16, 5);
        let result = binary_add(state.registers()[sr1], imm);
        state.registers()[dr] = result;
        update_condition_codes(state, result);
    }
    state.registers()[Registers::PC] += 1;
}

pub fn op_ld(state: &mut VmState, pc: usize) {
    let instruction = state.memory()[pc as u16];
    let dr = Registers::from_u16_or_panic((instruction >> 9) & 0b111);
    let imm = sign_extend(instruction & 0b111111111, 9);
    let address = binary_add(state.registers()[Registers::PC] + 1, imm);
    let value = state.memory()[address];
    update_condition_codes(state, value);
    state.registers()[dr] = state.memory()[address];
    state.registers()[Registers::PC] += 1;
}

pub fn op_lea(state: &mut VmState, pc: usize) {
    let instruction = state.memory()[pc as u16];
    let dr = Registers::from_usize_or_panic(((instruction >> 9) & 0b111) as usize);
    let imm = sign_extend(instruction & 0b111111111, 9);
    state.registers()[dr] = ((pc+1) as u16) + imm;
    let pc = state.registers()[Registers::PC] + 1;
    state.registers()[Registers::PC] = pc;
    // TODO: set condition flags!

    // println!("imm <0x{:x}>", imm);
    // println!("reg <{}> val <0x{:x}>", dr, state.registers[dr]);
    // println!("value at address <0x{:x}> is <0x{:x}>", state.registers[dr], state.memory[state.registers[dr] as usize]);
}

pub fn op_trap(state: &mut VmState, pc: usize) {
    // R7 is where we jump to upon completion of the handler. In the current implementation,
    // where we handle the traps in the VM, setting this is not necessary, but it's in the spec
    //state.registers[Registers::R7 as usize] = (pc+1) as u16;

    let trap_type = state.memory()[pc as u16] & 0b1111_1111;
    match trap_type {
        0x22 => trap_puts(state),
        0x25 => trap_halt(state),
        _ => panic!("Unimplemented trap vector <0x{:x}> at pc <0x{:x}>", trap_type, pc),
    }

    //state.registers[Registers::PC as usize] = state.registers[Registers::R7 as usize]
    let pc = state.registers()[Registers::PC] + 1;
    state.registers()[Registers::PC] = pc;
}

fn trap_halt(state: &mut VmState) {
    state.halt()
}

fn trap_puts(state: &mut VmState) {
    let mut start = state.registers()[Registers::R0];
    while state.memory()[start] != 0 {
        let character = (state.memory()[start] & 0xFF) as u8;
        state.display().print(character);
        start += 1;
    }
}