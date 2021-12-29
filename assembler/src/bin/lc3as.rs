use std::{env, fs};
use std::collections::HashMap;
use std::fs::OpenOptions;

use anyhow::{bail, Result};
use pest::iterators::Pairs;
use pest::Parser;

use std::io::Write;


use lc3as::parser::{parse, Lc3Parser, Rule, ErrorWithPosition, PositionContext};
use lc3as::*;
use lc3as::emitter::Emittable;

pub fn to_emittable(node: &Box<AstNode>) -> anyhow::Result<Emittable> {
    match node.as_ref() {
        AstNode::Instruction { opcode, operands } => {
            Emittable::from(opcode.clone(), operands.clone())
        },
        x => unreachable!("{:?}", x)
    }
}

pub fn get_label_maybe(label: &Option<Box<AstNode>>) -> Option<String> {
    if label.is_none() {
        return None
    }

    let unboxed = label.as_ref().unwrap().as_ref();
    match unboxed {
        AstNode::Label(name) => Some(name.clone()),
        x => unreachable!("{:?}", x),
    }
}

pub fn emit_section(origin: u16, content: Vec<AstNode>) -> Result<Vec<u16>, ErrorWithPosition> {
    let mut labels = HashMap::new();
    let mut emittables = vec![];

    // Pass 1
    for line in &content {
        match line {
            AstNode::Line {
                label,
                instruction: Some(x),
                position, ..
            } => {
                let label = get_label_maybe(label);
                let emittable = to_emittable(x).position(position.clone())?;
                emittables.push((label, emittable));
            }

            x => unreachable!("{:?}", x)
        }
    }

    // Pass 2: Collect the labels and their respective offsets
    let mut offset = origin;
    for (maybe_label, e) in &emittables {
        if let Some(label) = maybe_label {
            labels.insert(label.clone(), offset);
        }
        offset += e.size() as u16;
    }

    // Pass 3
    let mut offset = origin;
    let mut data = vec![origin];
    for (_, e) in emittables {
        data.append(&mut e.emit(offset, &labels));
        offset += e.size() as u16;
    }

    Ok(data)
}

pub fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let input_file = args.get(1).unwrap();
    let output_file = args.get(2).unwrap();

    // We leak the input assembly here, because Pests `Position` type contains references to
    // the content. Since we augment our own error type (ErrorWithPosition) with Pests position,
    // and we use the `?` operator in this function, the error potentially returned through use
    // of `?` lives longer than content. If it weren't static, the compiler would forbid us from
    // returning this error.
    let contents: &'static String = Box::leak(Box::new(fs::read_to_string(input_file)?));

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

            let data = emit_section(origin, content)?;
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
