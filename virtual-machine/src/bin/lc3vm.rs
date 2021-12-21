use std::fs::{read, File};
use std::io::prelude::*;
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use clap::{App, Arg};
use rustyline::error::ReadlineError;
use rustyline::Editor;
use termion::{color, style};

use lc3vm::peripheral::{TerminalDisplay, TerminalKeyboard};

use lc3vm::state::{Registers, VmState};
use lc3vm::{load_object, run, tick, VmOptions};

fn load_object_file(filename: &str, state: &mut VmState) -> Result<u16> {
    let mut f = File::open(filename)?;

    let mut buffer: Vec<u8> = vec![];
    f.read_to_end(&mut buffer)?;

    load_object(buffer.as_slice(), state)
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

enum Cmd {
    Run { orig: u16 },
    Load { path: String },
    Unknown { line: String },
    Help,
}

fn parse_command(line: String) -> Result<Cmd> {
    use Cmd::*;

    let x: Vec<_> = line.split_whitespace().collect();
    match x.as_slice() {
        ["load", path] => Ok(Load {
            path: path.to_string(),
        }),
        ["run", orig, ..] => {
            let orig = u16::from_str_radix(orig.trim_start_matches("0x"), 16)?;
            Ok(Run { orig })
        }
        ["?", ..] => Ok(Help),
        _ => Ok(Unknown { line }),
    }
}

fn eval_line(
    rl: &mut Editor<()>,
    state: &mut VmState,
    opts: &VmOptions,
    ctrl_c_pressed: Arc<AtomicBool>,
    line: String,
) -> Result<()> {
    rl.add_history_entry(&line);
    let cmd = parse_command(line)?;

    match cmd {
        Cmd::Load { path } => {
            let orig = load_object_file(path.as_str(), state)
                .with_context(|| format!("Failed to read from path '{}'", path))?;

            println!(
                "{}Loaded file into memory at origin address 0x{:x}",
                color::Fg(color::Blue),
                orig
            );
        }

        Cmd::Run { orig } => {
            state.registers()[Registers::PC] = orig;
            loop {
                tick(state, opts)?;
                if ctrl_c_pressed.load(Ordering::Relaxed) {
                    println!(
                        "\n{}Execution paused (PC = 0x{:x})",
                        color::Fg(color::Blue),
                        state.registers()[Registers::PC]
                    );
                    break;
                }
            }
        }

        Cmd::Help => println!("TODO print help"),
        Cmd::Unknown { line } => {
            println!("Unknown command '{}'", line)
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    pretty_env_logger::init();

    // let mut opts = parse_options();
    // let mut state = VmState::new();
    //
    // let display = TerminalDisplay {};
    // let keyboard = TerminalKeyboard::new();
    // opts.peripherals.push(&display);
    // opts.peripherals.push(&keyboard);
    //
    // state.registers()[Registers::PC] = opts.entry_point;
    // for filename in &opts.filenames {
    //     load_object_file(filename, &mut state)?;
    // }
    //
    // run(&mut state, &opts)

    let mut rl = Editor::<()>::new();
    if rl.load_history("history.txt").is_err() {
        println!("{}No previous history.", color::Fg(color::Blue));
        // TODO: Do I need to do anything here?
    }

    let ctrl_c_pressed = Arc::new(AtomicBool::new(false));
    let r = ctrl_c_pressed.clone();
    ctrlc::set_handler(move || {
        r.store(true, Ordering::Relaxed);
    })
    .expect("Error setting Ctrl-C handler");

    println!("lc3vm interactive mode. Type ? to get help!");

    let mut opts = VmOptions {
        peripherals: vec![],
        filenames: vec![],
        throttle: None,
        entry_point: 0,
    };
    let mut state = VmState::new();
    let display = TerminalDisplay {};
    //let keyboard = TerminalKeyboard::new();
    opts.peripherals.push(&display);
    //opts.peripherals.push(&keyboard);

    loop {
        let readline = rl.readline(format!("{}>> ", style::Reset).as_str());

        match readline {
            Ok(line) => {
                if line.is_empty() {
                    continue;
                }
                let res = eval_line(&mut rl, &mut state, &opts, ctrl_c_pressed.clone(), line);
                if let Err(err) = res {
                    println!("{}{:?}", color::Fg(color::Red), err);
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }

    rl.save_history("history.txt")?;
    Ok(())
}
