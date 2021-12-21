use std::time::Instant;

#[macro_use]
extern crate log;
#[macro_use]
extern crate num_derive;
use anyhow::Result;

#[macro_use]
pub mod util;
pub mod debug;
pub mod opcodes;
pub mod parser;
pub mod peripheral;
pub mod state;

use opcodes::*;
use peripheral::Peripheral;
use state::VmState;

pub fn tick(state: &mut VmState) -> Result<()> {
    state.tick();
    execute_next_instruction(state)?;

    let memory = &mut state.memory;
    for p in &state.peripherals {
        p.run(memory);
    }

    Ok(())
}

pub fn run(state: &mut VmState) -> Result<()> {
    let mut ticks = 0;
    let start = Instant::now();

    while state.running() {
        tick(state)?;
        ticks += 1;
    }

    let elapsed = start.elapsed();
    info!(
        "Ran {:?} instructions in {:?}ms ({:?} kHz)",
        ticks,
        elapsed.as_millis(),
        (ticks as f64 / elapsed.as_secs_f64() / 1000.0) as u64
    );

    Ok(())
}

pub fn load_object(bytes: &[u8], state: &mut VmState) -> Result<u16> {
    // LC3 uses 16-bit words, so we need to combine two bytes into one word of memory
    let even = bytes.iter().step_by(2);
    let odd = bytes.iter().skip(1).step_by(2);
    let zipped = even.zip(odd);

    let data: Vec<u16> = zipped
        .map(|(&high, &low)| (high as u16) << 8 | low as u16)
        .collect();

    // The first two bytes of the object file indicate where to load the program
    let orig = data[0];
    let program = &data[1..];

    let memory_area = (orig as usize)..((orig as usize) + program.len());
    state.memory_mut()[memory_area].copy_from_slice(program);

    Ok(orig)
}
