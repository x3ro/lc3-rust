use std::thread;
use std::time::{Duration, Instant};

#[macro_use]
extern crate log;
// extern crate pretty_env_logger;
#[macro_use]
extern crate num_derive;
// extern crate anyhow;
// extern crate clap;
// extern crate num_traits;

use anyhow::Result;

#[macro_use]
pub mod util;
pub mod opcodes;
pub mod parser;
pub mod peripheral;
pub mod state;

use opcodes::*;

use peripheral::Peripheral;

use state::VmState;

#[derive(Clone)]
pub struct VmOptions<'a> {
    pub throttle: Option<Duration>,
    pub peripherals: Vec<&'a dyn Peripheral>,
    pub entry_point: u16,
    pub filenames: Vec<String>,
}

impl<'a> VmOptions<'a> {
    pub fn with_entrypoint(&self, entry_point: u16) -> Self {
        VmOptions {
            entry_point,
            ..self.clone()
        }
    }

    pub fn with_filenames(&self, filenames: Vec<String>) -> Self {
        VmOptions {
            filenames,
            ..self.clone()
        }
    }

    pub fn with_filename(&self, filename: &str) -> Self {
        VmOptions {
            filenames: vec![filename.into()],
            ..self.clone()
        }
    }
}

pub fn run(state: &mut dyn VmState, opts: &VmOptions) -> Result<()> {
    let mut ticks = 0;
    let start = Instant::now();

    while state.running() {
        state.tick();
        execute_next_instruction(state)?;

        for p in &opts.peripherals {
            p.run(state);
        }

        if opts.throttle.is_some() {
            thread::sleep(opts.throttle.unwrap());
        }

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

pub fn load_object(bytes: &[u8], state: &mut dyn VmState) -> Result<()> {
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
    debug!("Loaded an object at <0x{:x}>", orig);

    let memory_area = (orig as usize)..((orig as usize) + program.len());
    state.memory()[memory_area].copy_from_slice(program);

    Ok(())
}
