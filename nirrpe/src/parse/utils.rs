use std::ops::RangeInclusive;

use chumsky::combinator::{Repeated, TryMap};
use chumsky::error::Rich;
use chumsky::extra::ParserExtra;
use chumsky::input::ValueInput;
use chumsky::prelude::{any, Input};
use chumsky::text::{digits, Char};
use chumsky::util::MaybeRef;
use chumsky::Parser;

use crate::parse::lexer::Lexer;

pub trait ParserTryUnwrapped<'a, I: Input<'a>, O, M, E: ParserExtra<'a, I>> {
    #[must_use]
    fn try_unwrapped(self) -> TryMap<Self, Result<O, M>, &'a dyn Fn(Result<O, M>, I::Span) -> Result<O, E::Error>>
    where
        Self: Sized;
}

impl<'a, P, I, O, M, E, S> ParserTryUnwrapped<'a, I, O, M, E> for P
where
    P: Parser<'a, I, Result<O, M>, E>,
    I: Input<'a, Span = S>,
    M: ToString,
    E: ParserExtra<'a, I, Error = Rich<'a, I::Token, S>>,
{
    fn try_unwrapped(self) -> TryMap<Self, Result<O, M>, &'a dyn Fn(Result<O, M>, S) -> Result<O, E::Error>>
    where
        Self: Sized,
    {
        self.try_map(&|o, span| o.map_err(|e| Rich::custom(span, e)))
    }
}

#[must_use]
pub fn n_digits<'a, C, I, E>(
    radix: u32,
    n: RangeInclusive<usize>,
) -> Repeated<impl Parser<'a, I, C, E> + Copy + Clone, C, I, E>
where
    C: Char,
    I: ValueInput<'a> + Input<'a, Token = C>,
    E: ParserExtra<'a, I>,
{
    digits(radix).at_least(*n.start()).at_most(*n.end())
}

#[must_use]
pub fn just_str<'s>(s: &'static str) -> Lexer!['s, &'s str] {
    any().repeated().exactly(s.len()).slice().try_map(move |x: &str, span| {
        if x == s {
            Ok(x)
        } else {
            Err(chumsky::error::Error::<&'s str>::expected_found(
                s.chars().map(|x| Some(MaybeRef::Val(x))).collect::<Vec<_>>(),
                None,
                span,
            ))
        }
    })
}
