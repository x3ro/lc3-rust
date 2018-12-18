const MEM_SIZE: usize = 65535;

struct Registers {
    r0: u16,
    r1: u16,
    r2: u16,
    r3: u16,
    r4: u16,
    r5: u16,
    r6: u16,
    r7: u16,
    pc: u16,
    cond: u16,
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

use std::fs::File;
use std::io;
use std::io::prelude::*;

fn load_object_file(filename: &str, mem: &mut [u8]) -> io::Result<()> {
    let mut f = File::open(filename).expect(&format!("File <{}> not found", filename));

    let mut buffer: Vec<u8> = vec![];
    f.read_to_end(&mut buffer)?;

    // The first two bytes of the object file indicate where to load the program
    let orig: usize = (buffer[0] as usize) << 8 | buffer[1] as usize;
    let program = &buffer[2..];
    println!("Origin = 0x{:x}", orig);

    mem[orig..(orig + program.len())].copy_from_slice(program);
    Ok(())
}

fn main() -> io::Result<()> {
    let mut mem: [u8; MEM_SIZE] = [0; MEM_SIZE];
    let mut reg = Registers {
        r0: 0,
        r1: 0,
        r2: 0,
        r3: 0,
        r4: 0,
        r5: 0,
        r6: 0,
        r7: 0,
        pc: 0x3000,
        cond: 0,
    };

    println!("Before {:x}", mem[0x3000]);
    load_object_file("asm-test/test.obj", &mut mem)?;
    println!("After {:x}", mem[0x3000]);

    println!("Bye!");
    Ok(())
}
