#![feature(slice_patterns)]

#[macro_use]
extern crate nom;



#[derive(Debug,PartialEq)]
pub struct Color {
  pub red:   u8,
  pub green: u8,
  pub blue:  u8,
}

fn from_hex(input: &str) -> Result<u8, std::num::ParseIntError> {
  u8::from_str_radix(input, 16)
}

fn is_hex_digit(c: char) -> bool {
  c.is_digit(16)
}

fn is_digit(c: char) -> bool {
    println!("wat: {}", c);
    c.is_digit(10)
}

fn is_space(c: char) -> bool { nom::is_space(c as u8) }

named!(hex_primary<&str, u8>,
  map_res!(take_while_m_n!(2, 2, is_hex_digit), from_hex)
);

named!(hex_color<&str, Color>,
  do_parse!(
           tag!("#")   >>
    red:   hex_primary >>
    green: hex_primary >>
    blue:  hex_primary >>
    (Color { red, green, blue })
  )
);


named!(positive_year<&str, &str>,
    take_while_m_n!(4, 4, is_digit)
);

named!(dotyear<&str, (&str, &str)>,
    do_parse!(
        left: positive_year >>
        char!('.') >>
        right: positive_year >>
        ((left, right))
    )
);

#[test]
fn parse_color() {
  assert_eq!(hex_color("#2F14DF"), Ok(("", Color {
    red: 47,
    green: 20,
    blue: 223,
  })));
}

//#[test]
//fn testlol() {
//    println!("{:?}", dotyear("1234.1234"));
//    assert_eq!("foo", "bar")
//}

//#[derive(Debug,PartialEq)]
//pub struct Origin<'a> {
//    pub val:   &'a str,
//}
//
#[derive(Debug,PartialEq)]
enum Merp {
    Origin = 1024,
}

impl Merp {
    fn into_nom(self) -> nom::ErrorKind<u32> {
        nom::ErrorKind::Custom(self as u32)
    }
}

named!(the_hex<CompleteByteSlice, CompleteByteSlice>,
    add_return_error!(Merp::Origin.into_nom(),
    do_parse!(
        opt!(char!('0')) >>
        char!('x') >>
        d: take_while1!(nom::is_digit) >>
        (d)
    ))
);

use nom::types::*;
use nom::{Err,ErrorKind,error_to_list,prepare_errors};
use nom::AsBytes;





#[test]
fn parse_origin() {
    let expected = CompleteByteSlice::from(&b"3000"[..]);
    let remainder = CompleteByteSlice::from(&b""[..]);

    let input = CompleteByteSlice::from(&b"3000"[..]);


    let res = the_hex(input);
    let foo = res.err().unwrap();

    if let Err::Error(context) = foo {
        let v : std::vec::Vec<(nom::types::CompleteByteSlice<'_>, nom::ErrorKind)> = error_to_list(&context);
//        let asd : Vec<nom::ErrorKind> = v.into_iter().map(|x| { x.1 }).collect();
//
//        println!("{:?} {:?}", asd, Merp::Origin.into_nom());

//        let ohnoes=  match &v[..] {
//            [(_, ErrorKind::Custom(42)), (_, ErrorKind::Char)] => "",
//            _            => "unrecognized error"
//        };


//        v.con


        let msg = if v.iter().any(|x| x.1 == Merp::Origin.into_nom()) {
          "ORIGIN ERROR LOL"
        } else {
            "dont know man"
        };

//        let msg = if &asd[..] == [Merp::Origin.into_nom(), nom::ErrorKind::Char] {
//            "oh shit"
//        } else {
//            "dont know man"
//        };
        println!("THE ERROR: {}", msg);

    }


//    match foo.clone() {
//        nom::Err::Error(x) => {
//            let v:Vec<ErrorKind> = error_to_list(&x);
//                println!("{:?}", v)
//        },
//        _ => println!("wtf")
//    }

//
//    println!("{:?}", error_to_list(res));

//    assert_eq!(error_to_list(res), Ok((remainder, expected)));
//    println!("{:?}", the_hex(b"3000"));
    assert_eq!("asd", "lol")
}


//    do_parse!(
//        tag!(".ORIG") >>
//        take_while!(is_space) >>
//        val: the_hex >>
//        Orig { val }
//    )
//    do_parse!(
//        tag!(".ORIG") >>
//        Orig { 'x' }
//    )
//named!(orig<&str, Origin>,
//    do_parse!(
//        tag!(".ORIG") >>
//        take_while!(is_space) >>
//        val: the_hex >>
//        (Origin { val })
//    )
//);





//#[test]
//fn foo() {
//    let foo = ".ORIG x3000";
//    let asd = r#"
//.ORIG x3000
//ADD R0, R0, #7
//HALT
//ADD R0, R0, #-7
//HALT
//ADD R0, R0, #-1
//HALT
//HELLO_STR .STRINGZ "If I don't add this the assembler segfaults"
//.END
//    "#;
//    named!(f, dbg!( watwat ) );
//    println!("{:?}", the_hex(b"3000"));
//    assert_eq!("foo", "bar")
//}