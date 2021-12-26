use std::{env, fs};

use anyhow::{bail, Context, Result};
use pest::iterators::{Pair, Pairs};
use pest::{Parser, Span};

use lc3as::AstNode::{ImmediateOperand, SectionScope};
use lc3as::*;
use lc3as::parser::{Lc3Parser, parse, Rule};

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

    println!("{:#?}", parse(&contents)?);
    Ok(())
}
