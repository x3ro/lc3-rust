//#[cfg(all(test, not(target_arch = "wasm32")))]

use std::fmt;

use lc3as::*;
use pest::iterators::Pair;
use pest::Parser;

use pretty_assertions::assert_eq;

const OS_ASM: &str = include_str!("../../virtual-machine/testcases/complex/os.asm");

#[derive(PartialEq, Eq)]
#[doc(hidden)]
pub struct PrettyString<'a>(pub &'a str);

/// Make diff to display string as multi-line string
impl<'a> fmt::Debug for PrettyString<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.0)
    }
}

// Macros taken from https://github.com/sunng87/handlebars-rust/blob/d8d9c6e25f49905fcfa1ec0c1afb32d95495cdc7/src/grammar.rs#L33
macro_rules! assert_rule {
    ($rule:expr, $in:expr) => {
        assert_eq!(
            Lc3Parser::parse($rule, $in)
                .unwrap()
                .last()
                .unwrap()
                .as_span()
                .end(),
            $in.len()
        );
    };
}

macro_rules! assert_not_rule {
    ($rule:expr, $in:expr) => {
        assert!(
            Lc3Parser::parse($rule, $in).is_err()
                || Lc3Parser::parse($rule, $in)
                    .unwrap()
                    .last()
                    .unwrap()
                    .as_span()
                    .end()
                    != $in.len()
        );
    };
}

macro_rules! assert_rule_match_ast {
    ($rule:expr, $in:expr, $ex:expr) => {
        let res = Lc3Parser::parse($rule, $in);
        assert!(res.is_ok());
        assert_eq!(
            PrettyString(&format_pair(res.unwrap().next().unwrap(), 0, false)),
            PrettyString($ex.trim()),
        );
    };
}

#[test]
#[wasm_bindgen_test]
fn test_lex_immediate() {
    assert_rule!(Rule::immediate_operand, "#12");
    assert_rule!(Rule::immediate_operand, "#-24");
    assert_not_rule!(Rule::immediate_operand, "#-24.0");
}

#[test]
#[wasm_bindgen_test]
fn test_lex_instruction() {
    assert_rule!(Rule::instruction, "ADD R0, R0, #1");
    assert_rule!(Rule::instruction, "ADD #1, #1");
    assert_rule!(Rule::instruction, "LDI R0,OS_MCR");
    assert_rule!(Rule::instruction, "BRnzp SOME_LABEL");
}

#[test]
#[wasm_bindgen_test]
fn test_lex_line() {
    assert_rule!(Rule::line, "SOME_LABEL ; nice label\n");
    assert_rule!(Rule::line, "SOME_LABEL ADD R0, R0, #1 ; nice label\n");

    assert_rule_match_ast!(
        Rule::line,
        "SOME_LABEL LDI R0, #1, x123 ;;;; nice label\n",
        r###"
line
  - label: "SOME_LABEL"
  - instruction
    - opcode: "LDI"
    - register_operand: "R0"
    - decimal_operand: "#1"
    - hex_operand: "x123"
  - comment: ";;;; nice label"
"###
    );
}

fn test_foo() {
    assert_rule_match_ast!(
        Rule::line,
        "HALT",
        r###"
line
  - instruction
    - trap_alias: "HALT"
"###
    );
}

#[test]
#[wasm_bindgen_test]
fn test_lex_section() {
    assert_rule!(Rule::section, ".ORIG x1234\n.END");
    assert_rule!(Rule::section, ".ORIG x1234\nADD R0, R0, #1\n.END");
}

#[test]
#[wasm_bindgen_test]
fn test_lex_file() {
    // A section can be preceded by comments, or comments can come after
    assert_rule!(
        Rule::file,
        "    ; some stuff\n;foo\n.ORIG x1234\nADD R0, R0, #1\n.END"
    );
    assert_rule!(Rule::file, ".ORIG x1234\nADD R0, R0, #1\n.END\n;wat?!");

    let input = r###"
; asd
.ORIG x1234
FOO
.END
.ORIG #3000
    ADD R0, R0, x1
    TRAP GETC
.END
; foo111
"###;

    let expected = r###"
 file
  - comment: "; asd"
  - section
    - section_start > hex_operand: "x1234"
    - line > label: "FOO"
    - section_end: ".END"
  - section
    - section_start > decimal_operand: "#3000"
    - line > instruction
      - opcode: "ADD"
      - register_operand: "R0"
      - register_operand: "R0"
      - hex_operand: "x1"
    - line > instruction
      - opcode: "TRAP"
      - label: "GETC"
    - section_end: ".END"
  - comment: "; foo111"
  - EOI: ""
"###;
    assert_rule_match_ast!(Rule::file, input, expected);
}

extern crate wasm_bindgen_test;
use wasm_bindgen_test::*;

#[test]
#[wasm_bindgen_test]
fn test_lex_lc3_os() {
    assert_rule!(Rule::file, OS_ASM);
}
