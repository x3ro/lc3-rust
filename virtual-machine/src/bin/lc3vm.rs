use std::fs::File;

use std::io::prelude::*;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{anyhow, Context, Result};
use clap::{App, Arg};
use rustyline::error::ReadlineError;
use rustyline::Editor;
use termion::{color, style};

use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};

use tui::text::Span;
use tui::widgets::{Block, Borders, Cell, Row, Table};
use tui::Terminal;

use lc3vm::peripheral::{Peripheral, TerminalDisplay};

use lc3vm::{load_object, tick};

use lc3vm::parser::Instruction;
use lc3vm::state::{ConditionFlags, Registers, VmState, MEM_SIZE};

const STYLE_INFO: Style = Style {
    fg: Some(Color::Blue),
    bg: None,
    add_modifier: Modifier::empty(),
    sub_modifier: Modifier::empty(),
};

const STYLE_ERROR: Style = Style {
    fg: Some(Color::Red),
    bg: None,
    add_modifier: Modifier::empty(),
    sub_modifier: Modifier::empty(),
};

const STYLE_WARN: Style = Style {
    fg: Some(Color::Yellow),
    bg: None,
    add_modifier: Modifier::empty(),
    sub_modifier: Modifier::empty(),
};

fn load_object_file(filename: &str, state: &mut VmState) -> Result<u16> {
    let mut f = File::open(filename)?;

    let mut buffer: Vec<u8> = vec![];
    f.read_to_end(&mut buffer)?;

    load_object(buffer.as_slice(), state)
}

struct ReplState<'a> {
    pause_after_tick: Arc<AtomicBool>,
    peripherals: Vec<&'a dyn Peripheral>,
    messages: Vec<Span<'a>>,
    source_context: u16,
    ticks_executed: u64,
    interactive: bool,
}

impl<'a> ReplState<'a> {
    fn from_params(params: &ReplParameters) -> Self {
        ReplState {
            pause_after_tick: spawn_ctrlc_listener(),
            peripherals: vec![],
            messages: vec![],
            source_context: 3,
            ticks_executed: 0,
            interactive: params.interactive,
        }
    }

    fn push_message(&mut self, msg: String, style: Style) {
        if self.interactive {
            self.messages.push(Span::styled(msg, style));
        } else {
            println!("{}", msg);
        }
    }
}

struct ReplParameters {
    programs: Vec<String>,
    entrypoint: u16,
    interactive: bool,
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
            Arg::with_name("interactive")
                .long("interactive")
                .short("i")
                .takes_value(false),
        )
        .get_matches();

    let programs: Vec<String> = matches
        .values_of("programs")
        .unwrap_or_default()
        .map(|s| s.into())
        .collect();

    let interactive = matches.is_present("interactive");
    let entrypoint = matches.value_of("entrypoint").unwrap_or("0x3000");
    let entrypoint = u16::from_str_radix(entrypoint.trim_start_matches("0x"), 16).unwrap();

    ReplParameters {
        interactive,
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
        ["s", args @ ..] | ["step", args @ ..] => parse_command_step(args),
        ["?", ..] => Ok(Help),
        _ => Err(anyhow!("Unknown command '{}'", line)),
    }
}

fn parse_command_step(args: &[&str]) -> Result<Cmd> {
    match args {
        [] => Ok(Cmd::Step { count: 1 }),
        [count] => Ok(Cmd::Step {
            count: u64::from_str_radix(count, 10)?,
        }),
        x => Err(anyhow!(
            "Unknown arguments '{}' for 'step' command",
            x.join(" ")
        )),
    }
}

fn eval_line(state: &mut VmState, repl_state: &mut ReplState, cmd: Cmd) -> Result<()> {
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

        Cmd::Step { count } => {
            for _ in 0..count {
                tick(state)?;
                repl_state.ticks_executed += 1;

                if !state.running() {
                    repl_state.push_message("VM has been halted".into(), STYLE_INFO);
                    break;
                }

                if repl_state.pause_after_tick.load(Ordering::Relaxed) {
                    repl_state.pause_after_tick.store(false, Ordering::Relaxed);
                    repl_state.push_message("Execution paused by user (CTRL-C)".into(), STYLE_INFO);
                    break;
                }
            }
        }

        Cmd::Continue => eval_line(state, repl_state, Cmd::Step { count: u64::MAX })?,

        Cmd::Empty => {
            repl_state.push_message(
                "You're in interactive mode. Type ? to get help or CTRL-C to exit.".into(),
                STYLE_INFO,
            );
        }
        Cmd::Help => {
            repl_state.push_message("Available commands:".into(), STYLE_INFO);
            repl_state.push_message(
                "(s)tep, (c)ontinue, load <path to obj file>".into(),
                STYLE_INFO,
            );
            repl_state.push_message("Press CTRL-C to stop VM while running".into(), STYLE_INFO);
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

fn create_registers_widget<'a>(vm_state: &VmState) -> Table<'a> {
    let regs = vec![
        format!(
            "R0: 0x{:04x} ({})",
            vm_state.registers[Registers::R0],
            vm_state.registers[Registers::R0] as i16
        ),
        format!(
            "R1: 0x{:04x} ({})",
            vm_state.registers[Registers::R1],
            vm_state.registers[Registers::R1] as i16
        ),
        format!(
            "R2: 0x{:04x} ({})",
            vm_state.registers[Registers::R2],
            vm_state.registers[Registers::R2] as i16
        ),
        format!(
            "R3: 0x{:04x} ({})",
            vm_state.registers[Registers::R3],
            vm_state.registers[Registers::R3] as i16
        ),
        format!(
            "R4: 0x{:04x} ({})",
            vm_state.registers[Registers::R4],
            vm_state.registers[Registers::R4] as i16
        ),
        format!(
            "R5: 0x{:04x} ({})",
            vm_state.registers[Registers::R5],
            vm_state.registers[Registers::R5] as i16
        ),
        format!(
            "R6: 0x{:04x} ({})",
            vm_state.registers[Registers::R6],
            vm_state.registers[Registers::R6] as i16
        ),
        format!(
            "R7: 0x{:04x} ({})",
            vm_state.registers[Registers::R7],
            vm_state.registers[Registers::R7] as i16
        ),
    ];

    let rows = regs.chunks(3).map(|items| {
        let cells = items.iter().map(|c| Cell::from(c.clone()));
        Row::new(cells) //.height(height as u16)
    });

    Table::new(rows)
        .block(
            Block::default()
                .borders(Borders::TOP)
                .title("─── Registers "),
        )
        .widths(&[
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
            // Constraint::Length(30),
            // Constraint::Min(10),
        ])
}

fn create_processor_state_widget<'a>(vm_state: &VmState) -> Table<'a> {
    let psr = vm_state.registers[Registers::PSR];
    let n = if (psr & ConditionFlags::Negative as u16) > 0 {
        "N"
    } else {
        "n"
    };
    let z = if (psr & ConditionFlags::Zero as u16) > 0 {
        "Z"
    } else {
        "z"
    };
    let p = if (psr & ConditionFlags::Positive as u16) > 0 {
        "P"
    } else {
        "p"
    };

    let regs = vec![
        format!("PC: 0x{:04x}", vm_state.registers[Registers::PC]),
        format!("PSR: 0x{:04x} ({}{}{})", psr, n, z, p),
        format!("SSP: 0x{:04x}", vm_state.registers[Registers::SSP]),
    ];

    let rows = regs.chunks(3).map(|items| {
        let cells = items.iter().map(|c| Cell::from(c.clone()));
        Row::new(cells) //.height(height as u16)
    });

    Table::new(rows)
        .block(
            Block::default()
                .borders(Borders::TOP)
                .title("─── Processor state "),
        )
        .widths(&[
            Constraint::Percentage(33),
            Constraint::Length(30),
            Constraint::Min(10),
        ])
}

fn create_assembly_widget<'a>(vm_state: &VmState, repl_state: &ReplState) -> Table<'a> {
    let pc = vm_state.registers[Registers::PC];
    let min = if (pc as usize - repl_state.source_context as usize) <= 0 {
        0
    } else {
        pc - repl_state.source_context
    };
    let max = if (pc as usize + repl_state.source_context as usize) >= MEM_SIZE {
        MEM_SIZE as u16
    } else {
        pc + repl_state.source_context
    };

    let regs: Vec<_> = (min..max)
        .map(|addr| {
            let value = vm_state.memory[addr];
            let marker = if addr == pc { "=> " } else { "" };
            let instruction = Instruction::from_raw(value);
            vec![
                marker.into(),
                format!("0x{:04x}", addr),
                format!("{:02x} {:02x}", (value >> 8) & 0xff, value & 0xff),
                format!("{:?}", instruction),
            ]
        })
        .collect();

    let rows = regs.iter().map(|items| {
        let cells = items.iter().map(|c| Cell::from(c.clone()));
        Row::new(cells) //.height(height as u16)
    });

    Table::new(rows)
        .block(Block::default().borders(Borders::TOP).title("─── Source "))
        .widths(&[
            Constraint::Min(3),
            Constraint::Min(7),
            Constraint::Min(7),
            Constraint::Percentage(100),
        ])
}

fn create_message_widget<'a>(repl_state: &'a ReplState) -> Table<'a> {
    let rows = repl_state.messages.iter().map(|msg| {
        let cells = vec![Cell::from(msg.clone())];
        Row::new(cells)
    });

    Table::new(rows)
        .block(
            Block::default()
                .borders(Borders::TOP)
                .title("─── Messages "),
        )
        .widths(&[Constraint::Percentage(100)])
}

fn print_ui(vm_state: &VmState, repl_state: &ReplState) -> Result<Vec<u8>> {
    let mut buf: Vec<u8> = Vec::new();
    {
        let mut backend = TermionBackend::new(Box::new(&mut buf));
        backend.write(termion::clear::All.as_ref())?;
        backend.write(termion::style::Reset.as_ref())?;

        let mut terminal = Terminal::new(backend)?;

        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Min(5),
                        Constraint::Min(3),
                        Constraint::Min((repl_state.source_context * 2 + 2) as u16),
                        Constraint::Percentage(100),
                    ]
                    .as_ref(),
                )
                .split(f.size());

            f.render_widget(create_registers_widget(vm_state), chunks[0]);
            f.render_widget(create_processor_state_widget(vm_state), chunks[1]);
            f.render_widget(create_assembly_widget(vm_state, repl_state), chunks[2]);
            f.render_widget(create_message_widget(repl_state), chunks[3]);

            // Position cursor at the bottom for prompt to be rendered
            let s = f.size();
            f.set_cursor(0, s.height - 1);
        })?;
    }

    Ok(buf)
}

fn loop_non_interactive(vm_state: &mut VmState, repl_state: &mut ReplState) -> Result<Duration> {
    let start = Instant::now();
    eval_line(vm_state, repl_state, Cmd::Continue)?;
    Ok(start.elapsed())
}

fn loop_interactive(vm_state: &mut VmState, repl_state: &mut ReplState) -> Result<()> {
    let mut rl = Editor::<()>::new();
    if rl.load_history("history.txt").is_err() {
        //println!("{}No previous history.", color::Fg(color::Blue));
        // TODO: Do I need to do anything here?
    }

    let mut last_command = Cmd::Empty;

    repl_state.push_message(
        "Welcome to lc3vm interactive mode. Type ? to get help or CTRL-C to exit.".into(),
        STYLE_INFO,
    );

    loop {
        std::io::stdout().write(print_ui(&vm_state, &repl_state)?.as_slice())?;
        repl_state.messages.clear();

        let readline = rl.readline(format!("{}>> ", style::Reset).as_str());

        match readline {
            Ok(line) => {
                let cmd = if line.is_empty() {
                    last_command.clone()
                } else {
                    rl.add_history_entry(&line);
                    let cmd = parse_command(line);
                    if let Err(err) = &cmd {
                        repl_state.push_message(format!("{:?}", err), STYLE_ERROR);
                        continue;
                    }
                    cmd.unwrap()
                };

                last_command = cmd.clone();
                let res = eval_line(vm_state, repl_state, cmd);
                if let Err(err) = res {
                    repl_state.push_message(format!("{:?}", err), STYLE_ERROR);
                    continue;
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

fn main() -> Result<()> {
    pretty_env_logger::init();
    let parameters = parse_cli_parameters();
    let mut repl_state = ReplState::from_params(&parameters);

    let mut vm_state = VmState::new();
    let display = TerminalDisplay {};
    //let keyboard = TerminalKeyboard::new();
    repl_state.peripherals.push(&display);
    //opts.peripherals.push(&keyboard);

    for p in &parameters.programs {
        let orig = load_object_file(p, &mut vm_state)?;
        repl_state.push_message(format!("Loaded '{}' at '0x{:x}'", p, orig), STYLE_INFO);
    }

    if parameters.programs.is_empty() {
        repl_state.interactive = true;
        repl_state.push_message(
            "You didn't request interactive mode, but we've enabled it,".into(),
            STYLE_WARN,
        );
        repl_state.push_message(
            "because you didn't specify a program to load via the CLI".into(),
            STYLE_WARN,
        );
        repl_state.push_message(
            "But, lucky you, do so now with the `load <path>` command!".into(),
            STYLE_WARN,
        );
    }

    vm_state.set_pc(parameters.entrypoint);
    //repl_state.push_message(format!("Set program counter to start at '0x{:x}", parameters.entrypoint), STYLE_INFO);

    if repl_state.interactive {
        loop_interactive(&mut vm_state, &mut repl_state)?;
    } else {
        let elapsed = loop_non_interactive(&mut vm_state, &mut repl_state)?;
        let mhz = repl_state.ticks_executed as f64 / elapsed.as_secs() as f64 / 1_000_000.0;
        println!(
            "Executed {} ticks in {}ms ({} MHz)",
            repl_state.ticks_executed,
            elapsed.as_millis(),
            mhz
        )
    }

    Ok(())
}
