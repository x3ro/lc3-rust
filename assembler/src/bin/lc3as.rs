use std::{env, fs};

use anyhow::Result;
use pest::iterators::Pairs;
use pest::Parser;



use lc3as::parser::{parse, Lc3Parser, Rule};
use lc3as::*;
use lc3as::emitter::Emittable;

pub fn to_emittable(node: &Box<AstNode>) -> Emittable {
    // TODO: Is there a way to pattern-match on this node without
    //       cloning the node? It seems like pattern matching on a
    //       boxed value isn't possible (?), but maybe I've missed
    //       something.
    let node = node.clone();
    match *node {
        AstNode::Instruction { opcode, operands } => {
            Emittable::from(opcode, operands)
        },
        x => unreachable!("{:?}", x)
    }
}

pub fn emit_section(origin: u16, content: Vec<AstNode>) {

    for line in &content {
        match line {
            AstNode::Line { instruction: Some(x), .. } => {
                //let y = *x.clone();
                println!("{:?}", to_emittable(x));
            }

            x => unreachable!("{:?}", x)
        }
    }
}

pub fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let file = args.get(1).unwrap();
    println!("{}", file);
    let contents = fs::read_to_string(file)?;
    println!("{}", contents);

    let pairs: Pairs<Rule> = Lc3Parser::parse(Rule::file, &contents)?;
    for pair in pairs {
        //println!("{:?}", pair.as_rule());
        assert!(pair.as_rule() == Rule::file);
        println!("{}", format_pair(pair, 0, false));
    }

    let mut ast = parse(&contents)?;
    // TODO: This assertion could be reflected in the grammar
    assert_eq!(ast.len(), 1, "More than one ORIGIN per file doesn't make sense");

    match ast.remove(0) {
        AstNode::SectionScope { origin, content } => {
            emit_section(origin, content);
        }

        x => unreachable!("{:?}", x)
    }

    Ok(())
}
