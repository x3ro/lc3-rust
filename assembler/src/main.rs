extern crate combine;
extern crate num_traits;
extern crate num_derive;

#[macro_use]
mod tokens;
mod parser;
mod emitter;

use tokens::*;
use parser::lc3_file;
use combine::Parser;

use combine::stream::state::State;

use std::collections::HashMap;
use emitter::Emittable;

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
        buffer.extend(&emittable.emit(&state));
    }

    buffer
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
//    LD R2, SOME_Y
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

    let expected: Vec<u16> = vec![0x3000, 0x1027, 0x1267, 0x1442, 0xf025];
    assert_eq!(expected, actual);
}
