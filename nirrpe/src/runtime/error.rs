use pest::error::Error as PestError;

use crate::parse::Rule;

#[derive(Debug)]
pub enum NirrpeError {
    ParseError(PestError<Rule>),
    RuntimeError(String),
}

impl From<PestError<Rule>> for NirrpeError {
    fn from(error: PestError<Rule>) -> Self {
        NirrpeError::ParseError(error)
    }
}

pub type NirrpeResult<T = ()> = Result<T, NirrpeError>;
