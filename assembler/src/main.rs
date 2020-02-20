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
use emitter::lol;

#[derive(Debug)]
pub struct Lc3State {
    pub offset: u16,
    pub emittables: Vec<Box<Emittable>>,
    pub labels: HashMap<String, u16>,
}

impl Lc3State {
    pub fn relative_offset(&self, from_offset: u16, to_label: &String) -> u16 {
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

pub fn into_emittable(state: &mut Lc3State, line: Line) -> &mut Lc3State {
    match line {
        Line { label, instruction: Some(i), .. } => {
            let e = lol(state.offset, i);

            if let Some(name) = label {
                state.labels.insert(name, state.offset);
            }

            state.offset += e.size();
            state.emittables.push(e);
        },
        _ => ()
    };
    state
}

pub fn foobar(ast: Lc3File) -> Vec<u16> {
//    let base_addr = ast.origin;

    let mut buffer:Vec<u16> = vec![];

    let mut state = Lc3State { offset: ast.origin, emittables: vec![], labels: HashMap::new() };
    ast.lines
        .into_iter()
        .fold(&mut state, |state, line| into_emittable(state, line));


//    println!("{:#?}", state.labels);

    for emittable in &state.emittables {
        buffer.extend(&emittable.emit(&state));
    }

    buffer
}

#[test]
pub fn test_foobar() {


    let input = r#"
.ORIG x3000
LD R1, SOME_X
LD R2, SOME_Y
;ADD R0, R0, R1 ; = 0 + 16 = 16
LD R2, SOME_Y
;HALT
LD R2, SOME_Y
;ADD R0, R0, R2 ; = 16 - 16 = 0
LD R2, SOME_Y
;HALT
LD R2, SOME_Y
;ADD R0, R0, R2 ;  = 0 - 16 = -16
LD R2, SOME_Y
;HALT
LD R2, SOME_Y
SOME_X    .FILL x10   ;  16
SOME_Y    .FILL xFFF0 ; -16
.END

"#;

    let r = lc3_file().easy_parse(State::new(input));
    if r.is_err() {
        println!("{:#?}", r);
    }

    let ast = r.unwrap().0;
//    println!("{:?}", &ast);



    let asd = foobar(ast);
    asd.iter().for_each(|x| println!("{:X} {:X}", (x >> 8) & 0xFF as u16, x & 0xFF));

    assert_eq!("foo","bar");
}
