use std::fs::File;
use std::io::prelude::*;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use clap::{App, Arg};
use rustyline::error::ReadlineError;
use rustyline::Editor;
use termion::{color, style};

use lc3vm::peripheral::{Peripheral, TerminalDisplay};

use lc3vm::load_object;
use lc3vm::state::{Registers, VmState};

fn load_object_file(filename: &str, state: &mut VmState) -> Result<u16> {
    let mut f = File::open(filename)?;

    let mut buffer: Vec<u8> = vec![];
    f.read_to_end(&mut buffer)?;

    load_object(buffer.as_slice(), state)
}

struct ReplState<'a> {
    throttle: Option<Duration>,
    pause_after_tick: Arc<AtomicBool>,
    peripherals: Vec<&'a dyn Peripheral>,
}

struct ReplParameters {
    throttle: Option<Duration>,
    programs: Vec<String>,
    entrypoint: u16,
}

fn parse_cli_parameters() -> ReplParameters {
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
            Arg::with_name("entrypoint")
                .short("e")
                .long("entrypoint")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("throttle")
                .long("throttle")
                .value_name("MILLISECONDS")
                .takes_value(true),
        )
        .get_matches();

    let programs: Vec<String> = matches
        .values_of("programs")
        .unwrap()
        .map(|s| s.into())
        .collect();

    let entrypoint = matches.value_of("entrypoint").unwrap_or("0x3000");
    let entrypoint = u16::from_str_radix(entrypoint.trim_start_matches("0x"), 16).unwrap();

    let throttle = matches
        .value_of("throttle")
        .and_then(|x| x.parse::<u64>().ok())
        .map(Duration::from_millis);

    ReplParameters {
        throttle,
        programs,
        entrypoint,
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

fn eval_line(state: &mut VmState, repl_state: &ReplState, cmd: Cmd) -> Result<()> {
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
                //tick(state, opts)?;
                if repl_state.pause_after_tick.load(Ordering::Relaxed) {
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
            println!("{}Unknown command '{}'", color::Fg(color::Yellow), line)
        }
    }

    Ok(())
}

fn spawn_ctrlc_listener() -> Arc<AtomicBool> {
    let ctrl_c_pressed = Arc::new(AtomicBool::new(false));
    let r = ctrl_c_pressed.clone();

    ctrlc::set_handler(move || {
        r.store(true, Ordering::Relaxed);
    })
    .expect("Error setting Ctrl-C handler");

    ctrl_c_pressed
}

fn main() -> Result<()> {
    pretty_env_logger::init();
    let parameters = parse_cli_parameters();

    let mut repl_state = ReplState {
        throttle: parameters.throttle,
        pause_after_tick: spawn_ctrlc_listener(),
        peripherals: vec![],
    };

    let mut vm_state = VmState::new();
    let display = TerminalDisplay {};
    //let keyboard = TerminalKeyboard::new();
    repl_state.peripherals.push(&display);
    //opts.peripherals.push(&keyboard);

    let mut rl = Editor::<()>::new();
    if rl.load_history("history.txt").is_err() {
        println!("{}No previous history.", color::Fg(color::Blue));
        // TODO: Do I need to do anything here?
    }

    println!("lc3vm interactive mode. Type ? to get help!");
    loop {
        let readline = rl.readline(format!("{}>> ", style::Reset).as_str());

        match readline {
            Ok(line) => {
                if line.is_empty() {
                    continue;
                }

                rl.add_history_entry(&line);
                let cmd = parse_command(line);
                if let Err(err) = &cmd {
                    println!("{}{:?}", color::Fg(color::Red), err);
                }

                let res = eval_line(&mut vm_state, &repl_state, cmd.unwrap());
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
