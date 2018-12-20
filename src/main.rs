use std::fs::File;
use std::io;
use std::io::prelude::*;

const MEM_SIZE: usize = 65535;
const REGISTER_COUNT: usize = 10;

enum Registers {
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

enum Opcodes {
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

enum ConditionFlags {
    Positive = 1 << 0,
    Zero = 1 << 1,
    Negative = 1 << 2,
}

struct VmState {
    memory: [u16; MEM_SIZE],
    registers: [u16; REGISTER_COUNT],
    running: bool,   
}

fn load_object_file(filename: &str, state: &mut VmState) -> io::Result<()> {
    let mut f = File::open(filename).expect(&format!("File <{}> not found", filename));

    let mut buffer: Vec<u8> = vec![];
    f.read_to_end(&mut buffer)?;

    // LC3 uses 16-bit words, so we need to combine two bytes into one word of memory
    let even = buffer.iter().step_by(2);
    let odd = buffer.iter().skip(1).step_by(2);
    let zipped = even.zip(odd);

    let data: Vec<u16> = zipped.map(|(&high, &low)| {
        (high as u16) << 8 | low as u16
    }).collect();

    // The first two bytes of the object file indicate where to load the program
    let orig: usize = data[0] as usize;
    let program = &data[1..];
    println!("Loaded object file at <0x{:x}>", orig);

    state.memory[orig..(orig + program.len())].copy_from_slice(program);
    state.registers[Registers::PC as usize] = orig as u16;

    Ok(())
}

fn sign_extend(x: u16, msb: u16) -> u16 {
    // Left-pads `x` with the bit value at the bit-position indicated by `msb`. 
    if (x >> (msb - 1)) == 0 {
        return x;
    }
    return !((2 as u16).pow(9)-1) | x;
}

fn op_lea(state: &mut VmState, pc: usize) {
    let instruction = state.memory[pc];
    let dr = ((instruction >> 9) & 0b111) as usize;
    let imm = sign_extend(instruction & 0b111111111, 9);
    state.registers[dr] = ((pc+1) as u16) + imm;
    state.registers[Registers::PC as usize] += 1
    // println!("imm <0x{:x}>", imm);
    // println!("reg <{}> val <0x{:x}>", dr, state.registers[dr]);
    // println!("value at address <0x{:x}> is <0x{:x}>", state.registers[dr], state.memory[state.registers[dr] as usize]);
}

fn op_trap(state: &mut VmState, pc: usize) {
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

fn run(state: &mut VmState) {
    while state.running {
        let pc = state.registers[Registers::PC as usize] as usize;
        let opcode = state.memory[pc] >> 12;
        
        match opcode {
            0xE => op_lea(state, pc),
            0xF => op_trap(state, pc),
            _ => panic!("Unrecognized opcode <0x{:x}> at pc <0x{:x}>", opcode, pc),
        }
    }
}

fn main() -> io::Result<()> {
    let mut state = VmState {
        memory: [0; MEM_SIZE],
        registers: [0; REGISTER_COUNT],
        running: true,
    };

    load_object_file("asm-test/test.obj", &mut state)?;
    run(&mut state);

    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_extend() {
        assert_eq!(sign_extend(0b0000_0001_0000_0000, 9), 0b1111_1111_0000_0000);
        assert_eq!(sign_extend(0b0000_0000_0101_0101, 9), 0b0000_0000_0101_0101);
    }
}