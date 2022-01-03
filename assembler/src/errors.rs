use std::fmt;
use pest::error::{Error, ErrorVariant};
use pest::Position;

#[derive(Debug, Clone)]
pub struct ErrorWithPosition<'a> {
    msg: String,
    pos: Position<'a>,
}

impl std::error::Error for ErrorWithPosition<'_> {}

impl<'a> fmt::Display for ErrorWithPosition<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err: Error<()> = Error::new_from_pos(
            ErrorVariant::CustomError {
                message: self.msg.clone(),
            },
            self.pos.clone(),
        );

        write!(f, "{}", err)
    }
}

pub trait PositionContext<'a, T, E> {
    /// Wrap the error value with additional context.
    fn position(self, pos: Position<'a>) -> Result<T, ErrorWithPosition<'a>>;
}

impl<'a, T, E: std::fmt::Display> PositionContext<'a, T, E> for Result<T, E> {
    fn position(self, pos: Position<'a>) -> Result<T, ErrorWithPosition<'a>> {
        match self {
            Ok(x) => Ok(x),
            Err(err) => Err(ErrorWithPosition {
                msg: format!("{}", err),
                pos,
            }),
        }
    }
}

// impl<'a> Into<anyhow::Error> for ErrorWithPosition<'a> {
//     fn into(self) -> anyhow::Error {
//         todo!()
//     }
// }

// impl<'a> From<ErrorWithPosition> for anyhow::Error {
//     fn from(_: ErrorWithPosition) -> Self {
//         todo!()
//     }
// }