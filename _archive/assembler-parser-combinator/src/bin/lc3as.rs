use std::error::Error;

pub fn main() -> Result<(), Box<dyn Error>> {
    use std::env;
    use std::fs::File;
    use std::io::prelude::*;

    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("Usage: lc3as <input file> <output file>");
        return Ok(());
    }
    let asm_input = args.get(1).unwrap();
    let obj_output = args.get(2).unwrap();

    let mut infile = File::open(asm_input)?;
    let mut contents = Box::new(String::new());
    infile.read_to_string(&mut contents)?;

    let bytecode = lc3_assembler::fulleverything(&contents)?;

    let mut outfile = File::create(obj_output)?;
    outfile.write_all(bytecode.as_slice())?;

    Ok(())
}