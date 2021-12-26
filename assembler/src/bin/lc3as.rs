use std::{env, fs};

use anyhow::Result;
use pest::iterators::Pairs;
use pest::Parser;

use lc3as::parser::{parse, Lc3Parser, Rule};
use lc3as::*;

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
