use std::fmt;

use anyhow::{anyhow, Context};
use pest::error::{Error, ErrorVariant};
use pest::iterators::{Pair, Pairs};
use pest::{Parser, Position};

use crate::{AstNode, Opcode, Register};

#[derive(Debug, Clone)]
pub struct ErrorWithPosition<'a> {
    msg: String,
    pos: Position<'a>,
}

impl std::error::Error for ErrorWithPosition<'_> {}

impl<'a> fmt::Display for ErrorWithPosition<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err: Error<()> = Error::new_from_pos(
            ErrorVariant::CustomError {
                message: self.msg.clone(),
            },
            self.pos.clone(),
        );

        write!(f, "{}", err)
    }
}

pub trait PositionContext<'a, T, E> {
    /// Wrap the error value with additional context.
    fn position(self, pos: Position<'a>) -> Result<T, ErrorWithPosition<'a>>;
}

impl<'a, T, E: std::fmt::Display> PositionContext<'a, T, E> for Result<T, E> {
    fn position(self, pos: Position<'a>) -> Result<T, ErrorWithPosition<'a>> {
        match self {
            Ok(x) => Ok(x),
            Err(err) => Err(ErrorWithPosition {
                msg: format!("{}", err),
                pos,
            }),
        }
    }
}

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct Lc3Parser;

pub fn parse(source: &str) -> anyhow::Result<Vec<AstNode>> {
    let mut pairs: Pairs<Rule> = Lc3Parser::parse(Rule::file, &source)?;

    let file = pairs.next().unwrap();
    assert_eq!(file.as_rule(), Rule::file);

    let yay = traverse(file).map_err(|e| {
        anyhow!("{}", e)
    })?;
    Ok(yay)
}

fn traverse(file: Pair<Rule>) -> Result<Vec<AstNode>, ErrorWithPosition> {
    let mut ast = vec![];

    for pair in file.into_inner() {
        let pos = pair.as_span().start_pos().clone();

        match pair.as_rule() {
            Rule::comment => { /* We ignore top-level comments */ }
            Rule::section => {
                ast.push(build_ast_from_section(pair)?);
            }
            Rule::EOI => { /* Ignore */ }
            rule => unreachable!("{:?} should not occur here", rule),
        }
    }

    Ok(ast)
}

fn parse_immediate_hex(src: &str) -> anyhow::Result<u16> {
    u16::from_str_radix(src, 16)
        .context(format!("'{}' is not a valid hexadecimal number", src))
}

fn parse_immediate_decimal(src: &str) -> anyhow::Result<u16> {
    i16::from_str_radix(src, 10).map(|x| x as u16)
        .context(format!("'{}' is not a valid decimal number", src))
}

fn parse_immediate(src: &str) -> anyhow::Result<u16> {
    match &src[..1] {
        "x" => parse_immediate_hex(&src[1..]),
        "#" => parse_immediate_decimal(&src[1..]),
        x => unreachable!("Invalid immediate value prefix '{}'. This is a bug, since the grammar should prevent this.", x),
    }
}

fn build_ast_from_section(pair: Pair<Rule>) -> Result<AstNode, ErrorWithPosition> {
    assert_eq!(pair.as_rule(), Rule::section);

    let mut origin: u16 = 0;
    let mut content = vec![];

    for pair in pair.into_inner() {
        let pos = pair.as_span().start_pos().clone();

        match pair.as_rule() {
            Rule::section_start => {
                let origin_str = pair.into_inner().next().unwrap().as_str();
                origin = parse_immediate(origin_str).position(pos)?;
            }
            Rule::line => content.push(build_ast_from_line(pair)?),
            Rule::section_end => { /* Ignore */ }
            rule => unreachable!("{:?} should not occur here", rule),
        }
    }

    Ok(AstNode::SectionScope { origin, content })
}

fn build_ast_from_line(pair: Pair<Rule>) -> Result<AstNode, ErrorWithPosition> {
    assert_eq!(pair.as_rule(), Rule::line);

    let mut label = None;
    let mut comment = None;
    let mut instruction = None;

    for pair in pair.into_inner() {
        let pos = pair.as_span().start_pos().clone();

        match pair.as_rule() {
            Rule::instruction => {
                let result = build_ast_from_instruction(pair)?;

                instruction = Some(Box::new(result));
            }

            Rule::label => {
                let node = Box::new(AstNode::Label(pair.as_str().into()));
                label = Some(node);
            }

            Rule::comment => {
                let value = pair.as_str().trim_start_matches(";").trim();
                comment = Some(value.into());
            }

            x => unreachable!("{:?}", x),
        }
    }

    Ok(AstNode::Line {
        label,
        comment,
        instruction,
    })
}

fn build_ast_from_instruction(pair: Pair<Rule>) -> Result<AstNode, ErrorWithPosition> {
    assert_eq!(pair.as_rule(), Rule::instruction);

    let mut opcode = None;
    let mut operands = vec![];

    for pair in pair.into_inner() {
        let pos = pair.as_span().start_pos().clone();

        match pair.as_rule() {
            Rule::opcode | Rule::pseudo_opcode | Rule::trap_alias => {
                let res = Opcode::from(pair.as_str()).position(pos)?;
                opcode = Some(res)
            }

            Rule::register_operand => {
                let register = Register::from_str(pair.as_str()).position(pos)?;
                let node = AstNode::RegisterOperand(register);
                operands.push(node);
            }

            Rule::decimal_operand | Rule::hex_operand => {
                let s = pair.as_str();
                let value = parse_immediate(s).position(pos)?;
                let node = AstNode::ImmediateOperand(value);
                operands.push(node);
            }

            Rule::string => {
                let value = pair.into_inner().next().expect("Presence should be guaranteed by the grammar");
                let node = AstNode::StringLiteral(value.as_str().into());
                operands.push(node);
            }

            Rule::label => {
                let node = AstNode::Label(pair.as_str().into());
                operands.push(node);
            }

            x => unreachable!("{:?}", x),
        }
    }

    Ok(AstNode::Instruction {
        opcode: opcode.expect("Presence should be guaranteed by the grammar"),
        operands,
    })
}
