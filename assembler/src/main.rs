#![feature(range_contains)]

extern crate combine;
extern crate num_traits;
extern crate num_derive;

use num_traits::FromPrimitive;
use std::convert::TryFrom;

use combine::{many1, Parser, sep_by,char,skip_many,satisfy,skip_many1,attempt,many};
use combine::char::{space,hex_digit,digit,upper,newline};
use combine::parser::range::take_while;

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



fn all_but_line_endings<I>() -> impl Parser<Input = I, Output = char>
    where
        I: Stream<Item = char>,
        I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    let f: fn(char) -> bool = |c| c != '\n' && c != '\r';
    satisfy(f).expected("any character (except line endings)")
}

#[test]
fn test_all_but_line_endings() {
    let result = skip_many(all_but_line_endings()).easy_parse("abcABC123;üX\nnew line");
    assert_eq!(Ok(((), "\nnew line")), result)
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
        })
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

fn operand_label<I>() -> impl Parser<Input = I, Output = Operand>
    where
        I: Stream<Item = char>,
        I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    label().map(|name| Operand::Label { name })
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
            immediate(),
            operand_label(),
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
            Opcode::try_from(&s)
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
        skip_many(space_no_line_ending()),
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



fn label<I>() -> impl Parser<Input = I, Output = String>
    where
        I: Stream<Item = char>,
        I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    many1(choice((upper(), char::char('_'))))
        .and_then(|label: String|
            match Opcode::try_from(&label) {
                Err(_) => Ok(label),
                Ok(_) => Err(format!("Labels must not have the same name as opcodes. Here: '{}'", label))
            }.map_err(|e| StreamErrorFor::<I>::message_message(e))
        )

}



// For convenience, to make writing "line()" a bit easier
fn maybe_label<I>() -> impl Parser<Input = I, Output = Option<String>>
    where
        I: Stream<Item = char>,
        I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    optional(attempt((
        label(),
        skip_many(space_no_line_ending()),
    )))
        .map(|x| match x {
            Some((label, _)) => Some(label),
            _ => None,
        })
}

#[test]
fn parse_label() {
    assert_eq!(label().easy_parse("FLUBBEL"), Ok((String::from("FLUBBEL"), "")));
    assert_eq!(label().easy_parse("fLUBBEL").is_err(), true);

    assert_eq!(maybe_label().easy_parse("FLUBBEL"), Ok((Some(String::from("FLUBBEL")), "")));
    assert_eq!(maybe_label().easy_parse("fLUBBEL"), Ok((None, "fLUBBEL")));

    // Labels cannot be the same as opcodes
    assert_eq!(label().easy_parse("ADD").is_err(), true);
}



fn maybe_comment<I>() -> impl Parser<Input = I, Output = Option<String>>
    where
        I: Stream<Item = char>,
        I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    optional((
        skip_many(space_no_line_ending()),
        char::char(';'),
        many(all_but_line_endings()),
    ))
        .map(|opt| match opt {
            Some((_,_,asd)) => Some(asd),
            None => None,
        })
}

#[test]
fn parse_maybe_comment() {
    let (result, remainder) = maybe_comment().easy_parse(" ; as;das;das\nnew line").unwrap();
    assert_eq!(result, Some(String::from(" as;das;das")));
    assert_eq!(remainder, "\nnew line");
}



fn some_line<I>() -> impl Parser<Input = I, Output = Line>
    where
        I: Stream<Item = char>,
        I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (
        maybe_label(),
        optional(instruction()),
        maybe_comment(),
    )
        .map(|(label,instruction,comment)| Line { label, instruction, comment })
}

#[test]
fn parse_line_full() {
    let (result, remainder) = some_line().easy_parse("FLUBBEL ADD R0, R1, #12 ; foobar").unwrap();

    assert_eq!(result.comment, Some(String::from(" foobar")));
    assert_eq!(result.label, Some(String::from("FLUBBEL")));
    assert_eq!(result.instruction, Some(Instruction {
        opcode: Opcode::Add,
        operands: vec![Operand::register(0), Operand::register(1), Operand::immediate(12)]})
    );
    assert_eq!(remainder, "");
}

#[test]
fn parse_line_only_label() {
    let (result, remainder) = some_line().easy_parse("FLUBBEL").unwrap();

    assert_eq!(result.comment, None);
    assert_eq!(result.label, Some(String::from("FLUBBEL")));
    assert_eq!(result.instruction, None);
    assert_eq!(remainder, "");
}

#[test]
fn parse_line_only_comment() {
    let (result, remainder) = some_line().easy_parse("; foobar").unwrap();

    assert_eq!(result.comment, Some(String::from(" foobar")));
    assert_eq!(result.label, None);
    assert_eq!(result.instruction, None);
    assert_eq!(remainder, "");
}



fn line<I, P, O>(p: P) -> impl Parser<Input = I, Output = O>
    where
        I: Stream<Item = char>,
        I::Error: ParseError<I::Item, I::Range, I::Position>,
        P: Parser<Input = I, Output = O>
{
    (
        skip_many(space_no_line_ending()),
        p,
        newline()
    )
        .map(|(_,output,_)| output)
}



fn lc3_file<I>() -> impl Parser<Input = I, Output = Lc3File>
    where
        I: Stream<Item = char>,
        I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (
        skip_many(space()),
        line(dot_origin()),
        many::<Vec<Line>,_>(line(some_line())),
        line(dot_command("END")),
        skip_many(space()),
    )
        .map(|(_,origin,lines,_,_)| Lc3File { origin, lines })
}

#[test]
fn parse_lc3_file() {
    let input = r#"
.ORIG x3000
LD R1, SOME_X
LD R2, SOME_Y
ADD R0, R0, R1 ; = 0 + 16 = 16
HALT
ADD R0, R0, R2 ; = 16 - 16 = 0
HALT
ADD R0, R0, R2 ;  = 0 - 16 = -16
HALT
SOME_X    .FILL x10   ;  16
SOME_Y    .FILL xFFF0 ; -16
HELLO_STR .STRINGZ "If I don't add this the assembler segfaults"
.END

"#;

    let r = lc3_file().easy_parse(State::new(input));
    println!("{:?}", r);


    assert_eq!("foo", "bar");

}

// TODO: more opcodes and dot commands
// OTHER_VALUE .FILL x1200
// HELLO_STR .STRINGZ "If I don't add this the assembler segfaults"