use std::error::Error;
use combine::stream::state::SourcePosition;

struct PrettyParserError {
    msg: Box<String>
}

impl Error for PrettyParserError {

}

impl std::fmt::Display for PrettyParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Oh no")
    }
}

impl std::fmt::Debug for PrettyParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

pub fn format_parser_error(contents: &str, err: combine::easy::Errors<char, &str, SourcePosition>) ->  Box<dyn Error> {
    use combine::easy::Error::*;
    use combine::easy::Info::*;

    let mut msg = Box::new(String::new());
    msg.push_str(format!("Encountered an issue while parsing file around line {} column {}:\n\n",
                         err.position.line,
                         err.position.column).as_str());

    if let Some(line) = contents.lines().nth((err.position.line - 1) as usize) {
        const INDENT_SIZE: usize = 8;
        let indent = " ".repeat(INDENT_SIZE);
        msg.push_str(format!(
            "{}{}\n{}---^\n\n",
            indent,
            line,
            " ".repeat(INDENT_SIZE + (err.position.column as usize) - 4)
        ).as_str())
    }

    for err in err.errors {
        let prefix = match err {
            Expected(info) => ("    * Expected: ", Some(info)),
            Unexpected(info) => ("    * Unexpected: ", Some(info)),
            Message(info) => ("    * ", Some(info)),
            _ => ("    * ", None),
        };

        let suffix: String = if let (_, Some(info)) = prefix {
            match info {
                Token(c) => format!("Token '{}'", c),
                Range(s) => s.into(),
                Owned(owned) => owned.clone(),
                Borrowed(s) => s.into()
            }
        } else {
            String::new()
        };

        msg.push_str(format!("{}{}\n", prefix.0, suffix).as_str());
    }
    Box::new(PrettyParserError { msg })
}