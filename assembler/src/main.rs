extern crate combine;
extern crate num_traits;
extern crate num_derive;

#[macro_use]
mod tokens;
mod parser;
mod emitter;
mod pretty_parser_error;

use tokens::*;
use parser::lc3_file;
use combine::Parser;


use combine::stream::state::State;

use std::collections::HashMap;
use emitter::Emittable;
use std::error::Error;
use pretty_parser_error::format_parser_error;

type Offset = u16;

#[derive(Debug)]
pub struct Lc3State {
    pub offset: Offset,
    pub emittables: Vec<Emittable>,
    pub labels: HashMap<String, Offset>,
}

impl Lc3State {
    pub fn relative_offset(&self, from_offset: Offset, to_label: &String) -> u16 {
        match self.labels.get(to_label) {
            None => panic!("Label '{}' referenced but never defined", to_label),
            Some(v) => {
                // -1 Because offset is counted from the next instruction
                // /16 because the relative offset is given in bytes, not bits
                ((v - from_offset)/16 - 1) as u16
            }
        }
    }
}

pub fn into_emittable(state: &mut Lc3State, line: Line) {
    if let Line { label, instruction: Some(instruction), .. } = line {
        let e = Emittable::from(instruction, state.offset);

        if let Some(name) = label {
            state.labels.insert(name, state.offset);
        }

        state.offset += e.size();
        state.emittables.push(e);
    }
}

pub fn assemble(ast: Lc3File) -> Vec<u16> {
    let mut buffer:Vec<u16> = vec![];
    let mut errors: Vec<(&Emittable, String)> = vec![];

    let mut state = Lc3State {
        offset: ast.origin,
        emittables: vec![],
        labels: HashMap::new()
    };

    // In this first pass, we record all labels and the offset of each instruction (i.e.
    // the value of the program counter when this instruction is executed).
    // in the to-be-assembled file, which is needed to calculate program counter based
    // offset parameters inside the file.
    ast.lines
        .into_iter()
        .for_each(|line| into_emittable(&mut state, line));

    // The origin (i.e. where the code should be loaded in memory) goes first
    buffer.push(ast.origin);

    // The second pass emits the actual byte code
    for emittable in &state.emittables {
        let res= emittable.emit(&state);
        if res.is_ok() {
            buffer.extend(res.unwrap());
        } else {
            errors.push((emittable, res.unwrap_err()));
        }
    }

    if errors.len() > 0 {
        for error in errors {
            panic!("Emitting error: {}", error.1);
        }
    }

    buffer
}

pub fn fulleverything(contents: &Box<String>) -> Result<Vec<u8>, Box<dyn Error>> {
    let r = lc3_file()
        .easy_parse(State::new(contents.as_str()))
        .map_err(|err| format_parser_error(contents.as_str(), err))?;

    let ast = r.0;
    let actual : Vec<u8> = assemble(ast)
        .iter()
        .flat_map(|x| vec![(x >> 8) as u8, (x & 0xff) as u8] )
        .collect();

    Ok(actual)
}

//pub fn main() {
//    let res = real_main();
//    if res.is_err() {
//        println!("{:?}", res.unwrap_err());
//    }
//}

pub fn main() -> Result<(), Box<dyn Error>> {
    use std::env;
    use std::fs::File;
    use std::io::prelude::*;

    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("Usage: lc3as <input file> <output file>");
        return Ok(())
        //return Err(Error::new(ErrorKind::Other, "Please provide a source file as the only parameter"));
    }
    let asm_input = args.get(1).unwrap();
    let obj_output = args.get(2).unwrap();

    let mut infile = File::open(asm_input)?;
    let mut contents = Box::new(String::new());
    infile.read_to_string(&mut contents)?;

    let bytecode = fulleverything(&contents)?;

    let mut outfile = File::create(obj_output)?;
    outfile.write_all(bytecode.as_slice())?;

    Ok(())
}


#[test]
pub fn test_basic_bytecode_emitting() {
//    let input = r#"
//.ORIG x3000
//    LD R1, SOME_X
//    LD R2, SOME_Y
//    ;ADD R0, R0, R1 ; = 0 + 16 = 16
//    LD R2, SOME_Y
//    ;HALT
//    LD R2, SOME_Y1
//    ;ADD R0, R0, R2 ; = 16 - 16 = 0
//    LD R2, SOME_Y
//    ;HALT
//    LD R2, SOME_Y
//    ;ADD R0, R0, R2 ;  = 0 - 16 = -16
//    LD R2, SOME_Y
//    ;HALT
//    LD R2, SOME_Y
//    SOME_X    .FILL x10   ;  16
//    SOME_Y    .FILL xFFF0 ; -16
//.END
//"#;

    let other = r#"
.ORIG x3000
    ADD R0, R0, #7
    ADD R1, R1, #7
    ADD R2, R1, R2
    HALT
.END
"#;

    let r = lc3_file().easy_parse(State::new(other));
    if r.is_err() {
        println!("{:#?}", r);
    }

    let ast = r.unwrap().0;
    let actual = assemble(ast);

    // Expected bytecode generated with lc3as of lc3tools package
    let expected: Vec<u16> = vec![0x3000, 0x1027, 0x1267, 0x1442, 0xf025];
    assert_eq!(expected, actual);
}
