use std::fs::File;
use std::io::prelude::*;
use std::time::Duration;

#[macro_use]
extern crate log;
use anyhow::Result;
use clap::{App, Arg};

use lc3vm::peripheral::{TerminalDisplay, TerminalKeyboard};

use lc3vm::state::{Registers, VmState};
use lc3vm::{run, VmOptions};

fn load_files(state: &mut VmState, opts: &VmOptions) -> Result<()> {
    state.registers()[Registers::PC] = opts.entry_point;

    for filename in &opts.filenames {
        load_object_file(filename, state)?;
    }

    Ok(())
}

fn load_object_file(filename: &str, state: &mut VmState) -> Result<()> {
    let mut f = File::open(filename).expect(&format!("File <{}> not found", filename));

    let mut buffer: Vec<u8> = vec![];
    f.read_to_end(&mut buffer)?;

    // LC3 uses 16-bit words, so we need to combine two bytes into one word of memory
    let even = buffer.iter().step_by(2);
    let odd = buffer.iter().skip(1).step_by(2);
    let zipped = even.zip(odd);

    let data: Vec<u16> = zipped
        .map(|(&high, &low)| (high as u16) << 8 | low as u16)
        .collect();

    // The first two bytes of the object file indicate where to load the program
    let orig = data[0];
    let program = &data[1..];
    debug!("Loaded <{}> at <0x{:x}>", filename, orig);

    let memory_area = (orig as usize)..((orig as usize) + program.len());
    state.memory()[memory_area].copy_from_slice(program);

    Ok(())
}

fn parse_options<'a>() -> VmOptions<'a> {
    let matches = App::new("Rust LC3 simulator")
        .arg(
            Arg::with_name("programs")
                .short("p")
                .long("program")
                .value_name("FILE")
                .multiple(true)
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("entry_point")
                .short("e")
                .long("entry-point")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("throttle")
                .long("throttle")
                .value_name("MILLISECONDS")
                .takes_value(true),
        )
        .get_matches();

    let filenames: Vec<String> = matches
        .values_of("programs")
        .unwrap()
        .map(|s| s.into())
        .collect();

    let entry_point = matches.value_of("entry_point").unwrap_or("0x3000");

    let throttle = matches
        .value_of("throttle")
        .and_then(|x| x.parse::<u64>().ok())
        .map(Duration::from_millis);

    let entry_point = u16::from_str_radix(entry_point.trim_start_matches("0x"), 16).unwrap();

    VmOptions {
        throttle,
        peripherals: vec![],
        filenames,
        entry_point,
    }
}

fn main() -> Result<()> {
    pretty_env_logger::init();

    let mut opts = parse_options();
    let mut state = VmState::new();

    let display = TerminalDisplay {};
    let keyboard = TerminalKeyboard::new();
    opts.peripherals.push(&display);
    opts.peripherals.push(&keyboard);

    load_files(&mut state, &opts)?;
    run(&mut state, &opts)
}
