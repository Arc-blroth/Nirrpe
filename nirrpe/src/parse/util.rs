use pest::error::{Error, ErrorVariant};
use pest::iterators::Pair;
use pest::{RuleType, Span};

use crate::parse::Rule;

pub(crate) trait GetSingleInner<'i, R> {
    fn into_single_inner(self) -> Pair<'i, R>;
}

impl<'i, R: RuleType> GetSingleInner<'i, R> for Pair<'i, R> {
    fn into_single_inner(self) -> Pair<'i, R> {
        self.into_inner().next().expect("Expected single inner rule!")
    }
}

pub fn parse_error<S: ToString>(message: S, span: Span) -> Error<Rule> {
    Error::new_from_span(
        ErrorVariant::CustomError {
            message: message.to_string(),
        },
        span,
    )
}
