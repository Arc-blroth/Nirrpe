use std::ops::RangeInclusive;

use chumsky::combinator::{Repeated, TryMap};
use chumsky::container::Container;
use chumsky::error::Rich;
use chumsky::extra::ParserExtra;
use chumsky::input::ValueInput;
use chumsky::prelude::{any, just, none_of, Input};
use chumsky::text::{digits, Char};
use chumsky::util::MaybeRef;
use chumsky::Parser;
use smallvec::{Array, SmallVec};

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
    O: Clone,
    M: ToString,
    E: ParserExtra<'a, I, Error = Rich<'a, I::Token, S>>,
{
    fn try_unwrapped(self) -> TryMap<Self, Result<O, M>, &'a dyn Fn(Result<O, M>, S) -> Result<O, E::Error>>
    where
        Self: Sized,
    {
        self.try_map_override(&|o, span| o.map_err(|e| Rich::custom(span, e)))
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

#[must_use]
pub fn recover_delimited_by<'a, C, I, E>(start: C, end: C) -> impl Parser<'a, I, (), E> + Clone
where
    C: Char,
    I: ValueInput<'a> + Input<'a, Token = C>,
    E: ParserExtra<'a, I>,
{
    just(start)
        .ignore_then(none_of(end).repeated().ignore_then(just(end)))
        .ignored()
}

#[derive(derive_more::Deref)]
pub struct SmallVecContainer<A: Array>(SmallVec<A>);

impl<A: Array> Default for SmallVecContainer<A> {
    fn default() -> Self {
        Self(SmallVec::default())
    }
}

impl<A: Array> Container<A::Item> for SmallVecContainer<A> {
    fn push(&mut self, item: A::Item) {
        self.0.push(item);
    }
}
