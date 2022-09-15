use bitflags::bitflags;
use lasso::Rodeo;
use pest::iterators::{Pair, Pairs};

use crate::parse::util::parse_error;
use crate::parse::{Parsable, ParseResult, Rule};

bitflags! {
    pub struct Modifiers: u32 {
        const EXTERN = 1 << 0;
        const PUB    = 1 << 1;
        const PRIV   = 1 << 2;
        const PURE   = 1 << 3;
        const IMPURE = 1 << 4;
    }
}

impl Parsable for Modifiers {
    fn parse(_rodeo: &mut Rodeo, pair: Pair<Rule>) -> ParseResult<Self> {
        assert_eq!(pair.as_rule(), Rule::modifier);
        Ok(match pair.as_str() {
            "extern" => Modifiers::EXTERN,
            "pub" => Modifiers::PUB,
            "priv" => Modifiers::PRIV,
            "pure" => Modifiers::PURE,
            "impure" => Modifiers::IMPURE,
            _ => unreachable!(),
        })
    }
}

impl Modifiers {
    pub fn parse_greedy(rodeo: &mut Rodeo, pairs: &mut Pairs<Rule>) -> ParseResult<Self> {
        let mut modifiers = Modifiers::empty();
        while let Some(pair) = pairs.peek() {
            if pair.as_rule() == Rule::modifier {
                let _ = pairs.next().unwrap();
                let span = pair.as_span();
                let modifier = Modifiers::parse(rodeo, pair)?;
                if modifiers.contains(modifier) {
                    return Err(parse_error("found duplicate modifier", span));
                }
                modifiers |= modifier;
                if modifiers.contains(Modifiers::PUB | Modifiers::PRIV) {
                    return Err(parse_error("cannot have multiple visibility modifiers", span));
                }
                if modifiers.contains(Modifiers::PURE | Modifiers::IMPURE) {
                    return Err(parse_error("cannot have both pure and impure modifiers", span));
                }
            } else {
                return Ok(modifiers);
            }
        }
        Ok(modifiers)
    }
}
