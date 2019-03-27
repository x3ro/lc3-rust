#![feature(range_contains)]

extern crate combine;
extern crate num_traits;
extern crate num_derive;

#[macro_use]
mod tokens;
mod parser;

use tokens::*;
use parser::lc3_file;
use combine::Parser;

use combine::stream::state::State;

use std::collections::HashMap;
use std::fmt::Debug;


//#[derive(Debug)]
//pub struct Emittable {
//    offset: u16,
//    instruction: Option<Instruction>,
//}


fn lol(offset: u16, instruction: Instruction) -> Box<Emittable> {
    match instruction {
        Instruction { opcode: Opcode::Ld, .. } => Box::new(Load { offset, instruction }),
        Instruction { opcode: Opcode::Fill, .. } => Box::new(Fill { offset, instruction }),
        _ => panic!("Unknown {:?}", instruction)
    }
}



pub trait Emittable : Debug {
    fn size(&self) -> u16 {
        16 // TODO: Not all emittables are 2 bytes
    }

    fn emit(&self, labels: &HashMap<String, u16>) -> Vec<u16>;
}



#[derive(Debug)]
pub struct Load { offset: u16, instruction: Instruction }
impl Load {
    pub fn calculate_relative_offset(&self, labels: &HashMap<String, u16>, label: &String) -> u16 {
        // -1 Because offset is counted from the next instruction
        // /16 because the relative offset is given in bytes, not bits
        ((labels.get(label).unwrap() - self.offset)/16 - 1) as u16
    }
}


impl Emittable for Load {
    fn emit(&self, labels: &HashMap<String, u16>) -> Vec<u16> {
        let opcode:u16 = 0b0010;

        let (register, offset) = match self.instruction.operands.as_slice() {
            [Operand::Register {r}, Operand::Label {name}] => (r, self.calculate_relative_offset(labels, name)),
            _ => panic!("Unsupported {:?}", self.instruction)
        };

        vec![(opcode << 12) | ((register.to_owned() as u16) << 9) | offset]
    }
}

#[derive(Debug)]
pub struct Fill { offset: u16, instruction: Instruction }
impl Emittable for Fill {
    fn emit(&self, _: &HashMap<String, u16>) -> Vec<u16> {
        self.instruction.operands
            .iter()
            .map(|x| match x {
                Operand::Immediate { value } => *value as u16,
                _ => panic!("Only immediate operands are allowed for fill in {:?}", self.instruction)
            })
            .collect()

    }
}



#[derive(Debug)]
pub struct Lc3State {
    offset: u16,
    emittables: Vec<Box<Emittable>>,
    labels: HashMap<String, u16>,
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

    for emittable in state.emittables {
        buffer.extend(&emittable.emit(&state.labels));
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
