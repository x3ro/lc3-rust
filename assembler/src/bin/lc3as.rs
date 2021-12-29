use std::{env, fs};
use std::fs::OpenOptions;

use anyhow::Result;
use pest::iterators::Pairs;
use pest::Parser;

use std::io::Write;


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

pub fn emit_section(origin: u16, content: Vec<AstNode>) -> Vec<u16> {
    let mut emittables = vec![];

    for line in &content {
        match line {
            AstNode::Line { instruction: Some(x), .. } => {
                //let y = *x.clone();
                emittables.push(to_emittable(x));
            }

            x => unreachable!("{:?}", x)
        }
    }






    println!("{:#?}", &emittables);


    let mut data = vec![origin];
    println!("{:?}", data);
    for e in emittables {
        data.append(&mut e.emit());
    }

    data
}

pub fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let input_file = args.get(1).unwrap();
    let output_file = args.get(2).unwrap();
    let contents = fs::read_to_string(input_file)?;

    let pairs: Pairs<Rule> = Lc3Parser::parse(Rule::file, &contents)?;
    for pair in pairs {
        assert!(pair.as_rule() == Rule::file);
        println!("{}", format_pair(pair, 0, false));
    }

    let mut ast = parse(&contents)?;
    // TODO: This assertion could be reflected in the grammar
    assert_eq!(ast.len(), 1, "More than one ORIGIN per file doesn't make sense");

    match ast.remove(0) {
        AstNode::SectionScope { origin, content } => {
            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .open(output_file)
                .unwrap();

            let data = emit_section(origin, content);
            for word in data {
                let low = (word & 0xff) as u8;
                let high = (word >> 8 & 0xff) as u8;
                file.write(&[high, low]);
            }
        }

        x => unreachable!("{:?}", x)
    }

    Ok(())
}
