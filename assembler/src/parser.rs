use anyhow::{Result, Context, anyhow};
use pest::error::{Error, ErrorVariant};
use pest::iterators::{Pair, Pairs};
use pest::Parser;

use crate::{AstNode, Opcode, Register};

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct Lc3Parser;

pub fn parse(source: &str) -> Result<Vec<AstNode>> {
    let mut ast = vec![];
    let mut pairs: Pairs<Rule> = Lc3Parser::parse(Rule::file, &source)?;

    let file = pairs.next().unwrap();
    assert_eq!(file.as_rule(), Rule::file);

    for pair in file.into_inner() {
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

fn parse_hex(value: &str) -> Result<u16> {
    u16::from_str_radix(value.trim_start_matches("x"), 16).context("")
}

fn build_ast_from_section(pair: Pair<Rule>) -> Result<AstNode> {
    assert_eq!(pair.as_rule(), Rule::section);

    let mut origin: u16 = 0;
    let mut content = vec![];

    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::section_start => {
                let origin_str = pair.into_inner().next().unwrap().as_str();
                origin = parse_hex(origin_str)?;
            }
            Rule::line => content.push(build_ast_from_line(pair)?),
            Rule::section_end => { /* Ignore */ }
            rule => unreachable!("{:?} should not occur here", rule),
        }
    }

    Ok(AstNode::SectionScope { origin, content })
}

fn build_ast_from_line(pair: Pair<Rule>) -> Result<AstNode> {
    assert_eq!(pair.as_rule(), Rule::line);

    let mut label = None;
    let mut comment = None;
    let mut instruction = None;

    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::instruction => {
                let span = pair.as_span();
                let result = build_ast_from_instruction(pair);

                let node = result.map_err(|err| {
                    let msg: Error<()> = Error::new_from_span(
                        ErrorVariant::CustomError {
                            message: err.to_string()
                        },
                        span
                    );
                    anyhow!("{}", msg)
                })?;

                instruction = Some(Box::new(node));
            }

            Rule::label => {
                let node = Box::new(AstNode::Label(pair.as_str().into()));
                label = Some(node);
            }

            Rule::comment => {
                let value = pair.as_str().trim_start_matches(";").trim();
                comment = Some(value.into());
            }

            x => unreachable!("{:?}", x)
        }
    }

    Ok(AstNode::Line {
        label,
        comment,
        instruction,
    })
}

fn build_ast_from_instruction(pair: Pair<Rule>) -> Result<AstNode> {
    assert_eq!(pair.as_rule(), Rule::instruction);

    let mut opcode = None;
    let mut operands = vec![];

    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::opcode | Rule::pseudo_opcode | Rule::trap_alias => {
                let res = Opcode::from(pair.as_str())?;
                opcode = Some(res)
            },

            Rule::register_operand => {
                let node = AstNode::RegisterOperand(Register::from_str(pair.as_str())?);
                operands.push(node);
            }

            Rule::decimal_operand |
            Rule::hex_operand => {
                let s = pair.as_str();
                let value = match &s[..1] {
                    "#" =>  i16::from_str_radix(&s[1..], 10)? as u16,
                    "x" =>  u16::from_str_radix(&s[1..], 16)?,
                    _ => unreachable!("The parser should make sure we can't get anything else here"),
                };
                operands.push(AstNode::ImmediateOperand(value))

            }

            Rule::string => {
                let value = pair.into_inner().next().unwrap();
                let node = AstNode::StringLiteral(value.as_str().into());
                operands.push(node);
            }

            Rule::label => {
                let node = AstNode::Label(pair.as_str().into());
                operands.push(node);
            }

            x => unreachable!("{:?}", x)
        }
    }

    Ok(AstNode::Instruction {
        opcode: opcode.unwrap(),
        operands,
    })
}