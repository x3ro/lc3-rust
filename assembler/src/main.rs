#![feature(range_contains)]

extern crate combine;
extern crate num_traits;
extern crate num_derive;

use num_traits::FromPrimitive;

use combine::{many1, Parser, sep_by,char,skip_many,satisfy,skip_many1};
use combine::char::{space,hex_digit,digit,upper};

use combine::error::ParseError;
use combine::stream::StreamErrorFor;
use combine::error::StreamError;
use combine::{choice, optional, Stream};
use combine::stream::state::State;



#[macro_use]
mod tokens;
use tokens::*;

fn space_no_line_ending<I>() -> impl Parser<Input = I, Output = char>
    where
        I: Stream<Item = char>,
        I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    let f: fn(char) -> bool = |x| char::is_whitespace(x) && x != '\n' && x != '\r';
    satisfy(f).expected("whitespace (except line endings)")
}

#[test]
fn parse_space_no_line_endings() {
    let result = skip_many(space_no_line_ending()).easy_parse("     \r\n");
    assert_eq!(Ok(((), "\r\n")), result)
}


fn dot_command<I>(cmd: &'static str) -> impl Parser<Input = I, Output = &'static str>
    where
        I: Stream<Item = char>,
        I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (
        char::char('.'),
        char::string_cmp(cmd, |l, r| l.eq_ignore_ascii_case(&r))
    )
        .map(|(_, parsed_cmd)| {
            parsed_cmd
        })
}

fn dot_origin<I>() -> impl Parser<Input = I, Output = i64>
    where
        I: Stream<Item = char>,
        I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (
        dot_command("ORIG"),
        skip_many1(space()),
        immediate()
    )
        .and_then(|(_,_,value)| {
            let max = std::u16::MAX as i64;
            match value {
                Operand::Immediate { value } if (0 ..= max).contains(&value) => Ok(value),
                value => Err(format!("Selected origin '{:?}' is negative or too large", value))
            }.map_err(|e|StreamErrorFor::<I>::message_message(e))
//            if value > u16::MAX { Ok(value) } else { Err("") }
        })
//        .map(|(_, _,value)| {
//            // It's okay to unwrap here, since the parser guarantees
//            // the string to only contain hex digits
//            u32::from_str_radix(&value, 16).unwrap()
//        })
}

#[test]
fn parse_dot_origin() {
    assert_eq!(Ok((0x1234, "")), dot_origin().easy_parse(".ORIG x1234"));
    assert_eq!(Ok((0x30, "")), dot_origin().easy_parse(".ORIG x30"));
    assert_eq!(true, dot_origin().easy_parse(".ORIG xFFFF1").is_err());
    assert_eq!(true, dot_origin().easy_parse(".ORIG #-1").is_err());
}

fn register<I>() -> impl Parser<Input = I, Output = Operand>
    where
        I: Stream<Item = char>,
        I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (
        char::char('R'),
        digit()
    )
        .map(|(_,register)| {
            let register_number = u32::from_str_radix(&register.to_string(), 10).unwrap();
            Operand::Register { r: Registers::from_u32(register_number).unwrap() }
        })
}

#[test]
fn parse_register() {
    assert_eq!(Ok((Operand::Register { r: Registers::R5 }, "")), register().easy_parse("R5"));
    assert_eq!(true, register().easy_parse("RX").is_err());
}



fn prefixed_hex<I>() -> impl Parser<Input = I, Output = i64>
    where
        I: Stream<Item = char>,
        I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (
        char::char('x'),
        many1::<String,_>(hex_digit())
    )
        .and_then(|(_, s)| {
            u16::from_str_radix(&s, 16)
                .map(|x| x as i64)
                .map_err(|e| StreamErrorFor::<I>::message_message(e))
        })
}

#[test]
fn parse_prefixed_hex() {
    assert_eq!(Ok((65298, "")), prefixed_hex().easy_parse("xFF12"));
    assert_eq!(Ok((66, "")), prefixed_hex().easy_parse("x42"));
    // Parse error if the integer is too large
    assert_eq!(true, prefixed_hex().easy_parse(State::new("xFFFF1")).is_err());
}

fn prefixed_signed<I>() -> impl Parser<Input = I, Output = i64>
    where
        I: Stream<Item = char>,
        I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (
        char::char('#'),
        optional(char::char('-')),
        many1::<String,_>(digit())
    )
        .and_then(|(_,sign,value)| {
            let multiplier = if sign == Some('-') { -1 } else { 1 };
            i16::from_str_radix(&value, 10)
                .map(|e| (e * multiplier) as i64)
                .map_err(|e| StreamErrorFor::<I>::message_message(e))
        })
}

#[test]
fn parse_prefixed_signed() {
    assert_eq!(Ok((32767, "")), prefixed_signed().easy_parse("#32767"));
    assert_eq!(Ok((-32767, "")), prefixed_signed().easy_parse("#-32767"));
    assert_eq!(Ok((0, "")), prefixed_signed().easy_parse("#0"));
    // Parse error if the integer is too large (signed, thus 2^15-1)
    assert_eq!(true, prefixed_signed().easy_parse(State::new("#32768")).is_err());
}


fn immediate<I>() -> impl Parser<Input = I, Output = Operand>
    where
        I: Stream<Item = char>,
        I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    choice((
        prefixed_hex(),
        prefixed_signed()
    ))
        .map(|op| Operand::Immediate { value: op } )
}

#[test]
fn parse_immediate() {
    assert_eq!(Ok((Operand::Immediate { value: -42}, "")), immediate().easy_parse("#-42"));
    assert_eq!(Ok((Operand::Immediate { value: 123}, "")), immediate().easy_parse("#123"));
    assert_eq!(Ok((Operand::Immediate { value: 255}, "")), immediate().easy_parse("xFF"));
}



fn operand<I>() -> impl Parser<Input = I, Output = Operand>
    where
        I: Stream<Item = char>,
        I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (
        skip_many(space_no_line_ending()),
        choice((
            register(),
            immediate()
        ))
    )
        .map(|(_,op)| {
            op
        })
}

fn operands<I>() -> impl Parser<Input = I, Output = Vec<Operand>>
    where
        I: Stream<Item = char>,
        I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (sep_by(
        operand(),
        char::char(',')
    ))
}

#[test]
fn parse_operands() {
    let mut expected = Ok((vec![Operand::register(0),Operand::register(1), Operand::immediate(-42)], ""));
    assert_eq!(expected, operands().easy_parse("R0, R1, #-42"));

    expected = Ok((vec![Operand::register(5),Operand::register(5), Operand::immediate(123)], ""));
    assert_eq!(expected, operands().easy_parse("R5, R5, #123"))
}





fn opcode<I>() -> impl Parser<Input = I, Output = Opcode>
    where
        I: Stream<Item = char>,
        I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    many1::<String,_>(upper())
        .and_then(|s|
            Opcode::from_string(s.to_owned())
                .map_err(|x| StreamErrorFor::<I>::unexpected_message(x))
        )
}


fn instruction<I>() -> impl Parser<Input = I, Output = Instruction>
    where
        I: Stream<Item = char>,
        I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (
        opcode(),
        skip_many1(space_no_line_ending()),
        operands()
    )
        .map(|(opcode,_,operands)| {
            Instruction { opcode, operands }
        })
}

#[test]
fn parse_instruction() {
    assert_eq!(
        instruction().easy_parse(State::new("ADD R0, R0, #7")).unwrap().0,
        Opcode::Add.instruction(vec![Operand::register(0), Operand::register(0), Operand::immediate(7)])
    );
}

fn identifier<I>() -> impl Parser<Input = I, Output = String>
    where
        I: Stream<Item = char>,
        I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    many1(upper())
}

#[test]
fn parse_identifier() {
    assert_eq!(identifier().easy_parse("FLUBBEL"), Ok((String::from("FLUBBEL"), "")));
    assert_eq!(identifier().easy_parse("fLUBBEL").is_err(), true)
}

// OTHER_VALUE .FILL x1200
// HELLO_STR .STRINGZ "If I don't add this the assembler segfaults"