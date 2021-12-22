use std::fs::File;
use std::io::prelude::*;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use clap::{App, Arg, Values};
use rustyline::error::ReadlineError;
use rustyline::Editor;
use termion::{color, style};

use lc3vm::peripheral::{Peripheral, TerminalDisplay};

use lc3vm::{load_object, tick};
use lc3vm::debug::fmt_instruction;
use lc3vm::opcodes::next_instruction;
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
        .unwrap_or_default()
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

#[derive(Clone)]
enum Cmd {
    Continue,
    Step { count: u64 },
    Load { path: String },
    Empty,
    Help,
}

fn parse_command(line: String) -> Result<Cmd> {
    use Cmd::*;

    let x: Vec<_> = line.split_whitespace().collect();
    match x.as_slice() {
        ["load", path] => Ok(Load {
            path: path.to_string(),
        }),
        ["c"] | ["continue"] => Ok(Continue),
        ["s", args @ .. ] | ["step", args @ .. ] => parse_command_step(args),
        ["?", ..] => Ok(Help),
        _ => Err(anyhow!("Unknown command '{}'", line)),
    }
}

fn parse_command_step(args: &[&str]) -> Result<Cmd> {
    match args {
        [] => {
            Ok(Cmd::Step { count: 1 })
        }
        [count] => {
            Ok(Cmd::Step { count: u64::from_str_radix(count, 10)? })
        }
        x => Err(anyhow!("Unknown arguments '{}' for 'step' command", x.join(" ")))
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

        Cmd::Step { count }  => {
            for _ in 0..count {
                let next = next_instruction(state);
                println!("{}{}", style::Reset, fmt_instruction(state, &next)?);

                tick(state)?;

                if !state.running() {
                    println!("\n{}VM has been halted", color::Fg(color::Blue));
                    break
                }

                if repl_state.pause_after_tick.load(Ordering::Relaxed) {
                    repl_state.pause_after_tick.store(false, Ordering::Relaxed);
                    println!(
                        "\n{}Execution paused by user (CTRL-C)", color::Fg(color::Blue),
                    );
                    break;
                }
            }

            let next = next_instruction(state);
            println!("{}{}", style::Reset, fmt_instruction(state, &next)?);
        },

        Cmd::Continue => {
            eval_line(state, repl_state, Cmd::Step { count: u64::MAX })?
        }

        Cmd::Empty => () /* Do nothing */,
        Cmd::Help => println!("TODO print help"),
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

    for p in &parameters.programs {
        let orig = load_object_file(p, &mut vm_state)?;
        println!("{}Loaded '{}' at '0x{:x}'", color::Fg(color::Blue), p, orig);
    }

    vm_state.set_pc(parameters.entrypoint);
    println!("{}Set program counter to start at '0x{:x}'", color::Fg(color::Blue), parameters.entrypoint);

    let mut rl = Editor::<()>::new();
    if rl.load_history("history.txt").is_err() {
        println!("{}No previous history.", color::Fg(color::Blue));
        // TODO: Do I need to do anything here?
    }

    let mut last_command = Cmd::Empty;

    println!("{}lc3vm interactive mode. Type ? to get help!", style::Reset);
    loop {
        let readline = rl.readline(format!("{}>> ", style::Reset).as_str());

        match readline {
            Ok(line) => {
                let cmd = if line.is_empty() {
                    last_command.clone()
                } else {
                    rl.add_history_entry(&line);
                    let cmd = parse_command(line);
                    if let Err(err) = &cmd {
                        println!("{}{:?}", color::Fg(color::Red), err);
                        continue
                    }
                    cmd.unwrap()
                };

                last_command = cmd.clone();
                let res = eval_line(&mut vm_state, &repl_state, cmd);
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
