extern crate pest;
#[macro_use]
extern crate pest_derive;

use std::error::Error;

use pest::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct Lc3Parser;

pub fn main() -> Result<(), Box<dyn Error>> {
    // let asd = FooParser::parse(Rule::field, "asd");
    // println!("{:?}", asd);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pest::iterators::Pair;

    fn format_pair(pair: Pair<Rule>, indent_level: usize, is_newline: bool) -> String {
        let indent = if is_newline {
            "  ".repeat(indent_level)
        } else {
            "".to_string()
        };

        let children: Vec<_> = pair.clone().into_inner().collect();
        let len = children.len();
        let children: Vec<_> = children
            .into_iter()
            .map(|pair| {
                format_pair(
                    pair,
                    if len > 1 {
                        indent_level + 1
                    } else {
                        indent_level
                    },
                    len > 1,
                )
            })
            .collect();

        let dash = if is_newline { "- " } else { "" };

        match len {
            0 => format!(
                "{}{}{:?}: {:?}",
                indent,
                dash,
                pair.as_rule(),
                pair.as_span().as_str()
            ),
            1 => format!("{}{}{:?} > {}", indent, dash, pair.as_rule(), children[0]),
            _ => format!(
                "{}{}{:?}\n{}",
                indent,
                dash,
                pair.as_rule(),
                children.join("\n")
            ),
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
                $ex.trim(),
                format_pair(res.unwrap().next().unwrap(), 0, false)
            );
        };
    }

    #[test]
    fn test_lex_immediate() {
        assert_rule!(Rule::immediate_operand, "#12");
        assert_rule!(Rule::immediate_operand, "#-24");
        assert_not_rule!(Rule::immediate_operand, "#-24.0");
    }

    #[test]
    fn test_lex_instruction() {
        assert_rule!(Rule::instruction, "ADD R0, R0, #1");
        assert_rule!(Rule::instruction, "ADD #1, #1");
        assert_rule!(Rule::instruction, "LDI R0,OS_MCR");
    }

    #[test]
    fn test_lex_line() {
        assert_rule!(Rule::line, "SOME_LABEL ; nice label\n");
        assert_rule!(Rule::line, "SOME_LABEL ADD R0, R0, #1 ; nice label\n");

        assert_rule_match_ast!(
            Rule::line,
            "SOME_LABEL LDI R0, #1 ;;;; nice label\n",
            r###"
line
  - label: "SOME_LABEL"
  - instruction
    - opcode: "LDI"
    - operand: "R0"
    - operand: "#1"
  - comment: ";;;; nice label"
"###
        );
    }

    // #[test]
    // fn test_lex_register() {
    //     assert_rule!(Rule::register, "R1");
    //     assert_not_rule!(Rule::register, "RR");
    // }
}
