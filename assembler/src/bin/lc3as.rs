use std::{env, fs};

use std::fs::OpenOptions;

use anyhow::{Result};



use std::io::Write;
use pest::Position;

use lc3as::*;




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

    //let pairs: Pairs<Rule> = Lc3Parser::parse(Rule::file, &contents)?;
    // for pair in pairs {
    //     assert!(pair.as_rule() == Rule::file);
    //     println!("{}", format_pair(pair, 0, false));
    // }

    let assembly = assemble(&contents)?;

    // for (offset, loc) in assembly.source_map().iter() {
    //     let pos = Position::new(&contents, *loc).unwrap();
    //     println!("0x{:x} -> {}", offset, pos.line_of());
    // }

    let bytecode = assembly.data();

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(output_file)
        .unwrap();

    for word in bytecode {
        let low = (word & 0xff) as u8;
        let high = (word >> 8 & 0xff) as u8;
        file.write(&[high, low])?;
    }

    Ok(())
}
