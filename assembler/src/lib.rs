pub mod parser;
pub mod emittable;
mod emitter;
mod errors;

#[macro_use]
extern crate pest_derive;

use wasm_bindgen::prelude::*;


use anyhow::{anyhow, bail};
use pest::iterators::Pair;

use num_traits::FromPrimitive;
use pest::Position;
use crate::emitter::{Assembly, emit_section};


use crate::parser::{parse, Rule};

#[derive(Debug, PartialEq, Copy, Clone, num_derive::FromPrimitive)]
pub enum Register {
    R0 = 0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
}

impl Register {
    pub fn from_str(str: &str) -> anyhow::Result<Self> {
        let s = str.to_lowercase();
        let s = s.trim_start_matches("r");
        let n = u8::from_str_radix(s, 10)?;
        Register::from_u8(n)
            .ok_or(anyhow!("Unknown register '{}'", str))
    }
}

#[derive(Debug, Clone)]
pub struct Modifiers {
    negative: bool,
    zero: bool,
    positive: bool,
}

impl Modifiers {
    fn from_str(s: &str) -> anyhow::Result<Self> {
        let invalid = s.contains(|c| c != 'n' && c != 'z' && c != 'p');
        if invalid {
            bail!("Invalid modifiers")
        }

        let negative = s.contains("n");
        let zero = s.contains("z");
        let positive = s.contains("p");

        if !negative && !zero && !positive {
            // If there aren't any modifiers, the branching is unconditional,
            // which means we branch in all of the possible cases.
            Ok(Modifiers {
                negative: true,
                zero: true,
                positive: true,
            })
        } else {
            Ok(Modifiers {
                negative,
                zero,
                positive,
            })
        }
    }
}

#[derive(Debug, Clone)]
pub enum Opcode {
    Add,
    And,
    Br { modifiers: Modifiers },
    Jmp,
    Jsr,
    Jsrr,
    Ld,
    Ldi,
    Ldr,
    Lea,
    Nop,
    Not,
    Ret,
    Rti,
    St,
    Sti,
    Str,
    Trap,

    // Traps with opcode-like aliases
    Getc,
    Out,
    Puts,
    In,
    Putsp,
    Halt,

    // Pseudo-opcodes
    Fill,
    Stringz,
    Blkw,
}

impl Opcode {
    pub fn from(value: &str) -> anyhow::Result<Self> {
        let value = value.to_lowercase();

        // The BR opcode is the only one that supports
        // modifiers (n, z and p), so we handle it as
        // a special case here.
        if value.starts_with("br") {
            let modifiers_str = value.trim_start_matches("br");
            return Ok(Opcode::Br {
                modifiers: Modifiers::from_str(modifiers_str)?,
            });
        }

        match value.as_ref() {
            "add" => Ok(Opcode::Add),
            "and" => Ok(Opcode::And),
            "jmp" => Ok(Opcode::Jmp),
            "jsr" => Ok(Opcode::Jsr),
            "jsrr" => Ok(Opcode::Jsrr),
            "ld" => Ok(Opcode::Ld),
            "ldi" => Ok(Opcode::Ldi),
            "ldr" => Ok(Opcode::Ldr),
            "lea" => Ok(Opcode::Lea),
            "nop" => Ok(Opcode::Nop),
            "not" => Ok(Opcode::Not),
            "ret" => Ok(Opcode::Ret),
            "rti" => Ok(Opcode::Rti),
            "st" => Ok(Opcode::St),
            "sti" => Ok(Opcode::Sti),
            "str" => Ok(Opcode::Str),

            "trap" => Ok(Opcode::Trap),
            "getc" => Ok(Opcode::Getc),
            "out" => Ok(Opcode::Out),
            "puts" => Ok(Opcode::Puts),
            "in" => Ok(Opcode::In),
            "putsp" => Ok(Opcode::Putsp),
            "halt" => Ok(Opcode::Halt),

            ".fill" => Ok(Opcode::Fill),
            ".stringz" => Ok(Opcode::Stringz),
            ".blkw" => Ok(Opcode::Blkw),

            _ => Err(anyhow!("Unknown opcode '{}'", value)),
        }
    }
}

#[derive(Debug, Clone)]
pub enum AstNode<'a> {
    Line {
        label: Option<Box<AstNode<'a>>>,
        instruction: Option<Box<AstNode<'a>>>,
        comment: Option<String>,
        position: Position<'a>
    },
    Instruction {
        opcode: Opcode,
        operands: Vec<AstNode<'a>>,
    },
    SectionScope {
        origin: u16,
        content: Vec<AstNode<'a>>,
    },
    FileScope {
        content: Vec<AstNode<'a>>,
    },
    Label(String),
    StringLiteral(String),
    RegisterOperand(Register),
    ImmediateOperand(u16),
}


pub fn assemble(source: &str) -> anyhow::Result<Assembly> {
    let mut ast = parse(source)?;
    // TODO: This assertion could be reflected in the grammar
    assert_eq!(ast.len(), 1, "More than one ORIGIN per file doesn't make sense");

    let res = match ast.remove(0) {
        AstNode::SectionScope { origin, content } => {
            emit_section(origin, content)
        }
        x => unreachable!("Assembler bug: Unexpected top-level AST node. {:?}", x)
    };
    
    res.map_err(|e| {
        anyhow!("{}", e)
    })
}

#[wasm_bindgen]
pub fn assemble_js(source: &str) -> Result<Vec<u16>, JsValue> {
    let res = assemble(source);
    // res.map(|data| data.into_iter().map(|x| format!("{:x}", x).into()).collect())
    res.map(|assembly| assembly.data().clone())
        .map_err(|err| err.to_string().into())
}


// Taken from https://github.com/pest-parser/site/blob/221c5b1dd84e15752680cc129fa6138196f2a24e/src/main.rs#L70
pub fn format_pair(pair: Pair<Rule>, indent_level: usize, is_newline: bool) -> String {
    let indent = if is_newline {
        "  ".repeat(indent_level)
    } else {
        "".to_string()
    };

    let children: Vec<_> = pair.clone().into_inner().collect();
    let len = children.len();
    let children: Vec<_> = children
        .into_iter()
        .map(|pair| {
            format_pair(
                pair,
                if len > 1 {
                    indent_level + 1
                } else {
                    indent_level
                },
                len > 1,
            )
        })
        .collect();

    let dash = if is_newline { "- " } else { "" };

    match len {
        0 => format!(
            "{}{}{:?}: {:?}",
            indent,
            dash,
            pair.as_rule(),
            pair.as_span().as_str()
        ),
        1 => format!("{}{}{:?} > {}", indent, dash, pair.as_rule(), children[0]),
        _ => format!(
            "{}{}{:?}\n{}",
            indent,
            dash,
            pair.as_rule(),
            children.join("\n")
        ),
    }
}
