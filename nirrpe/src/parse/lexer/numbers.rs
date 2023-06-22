use std::mem::size_of;
use std::num::ParseIntError;

use chumsky::prelude::{choice, just, one_of, via_parser, Parser};
use chumsky::text;

use crate::parse::lexer::Lexer;
use crate::parse::token::Token;
use crate::parse::utils::{just_str, ParserTryUnwrapped};

pub fn lexer<'s>() -> Lexer!['s, Token] {
    let hex_header = just('0').ignore_then(one_of("xX"));
    let hex_int = hex_header
        .clone()
        .ignore_then(num_with_separators::<16>())
        .try_unwrapped()
        .map(Token::Int)
        .recover_with(via_parser(hex_header.ignore_then(num_with_separators_ignored::<16>())));

    let bin_header = just('0').ignore_then(one_of("bB"));
    let bin_int = bin_header
        .clone()
        .ignore_then(num_with_separators::<2>())
        .try_unwrapped()
        .map(Token::Int)
        .recover_with(via_parser(bin_header.ignore_then(num_with_separators_ignored::<2>())));

    let float_part = text::digits(10)
        .ignored()
        .separated_by(just('_').ignored().repeated())
        .allow_leading();
    let float_exp = one_of("eE")
        .ignore_then(one_of("+-").or_not().ignored())
        .ignore_then(float_part.clone());
    let float = choice([
        just_str("inf"),
        just_str("Inf"),
        just_str("infinity"),
        just_str("Infinity"),
        just_str("NaN"),
        just_str("nan"),
    ])
    .slice()
    .or(float_part
        .clone()
        .or_not()
        .then(just('.'))
        .then(float_part.clone())
        .then(float_exp.clone().or_not())
        .slice())
    .or(float_part
        .clone()
        .then(just('.'))
        .then(float_exp.clone().or_not())
        .slice())
    .or(float_part.ignore_then(float_exp).slice())
    .map(|x: &str| x.replace('_', "").parse::<f64>())
    .try_unwrapped()
    .map(Token::Float);

    let int = num_with_separators::<10>().try_unwrapped().map(Token::Int);
    hex_int.or(bin_int).or(float).or(int)
}

fn num_with_separators<'s, const RADIX: u32>() -> Lexer!['s, Result<u64, ParseIntError>] {
    partial_num::<RADIX>()
        .foldl(
            just('_').repeated().ignore_then(partial_num::<RADIX>()).repeated(),
            |left, right| {
                let left = left?;
                let right = right?;
                let left_size = num_radix_bits_in::<RADIX>(left);
                let right_size = num_radix_bits_in::<RADIX>(right);
                const MAX_SIZE: u32 = (size_of::<u64>() * 8) as u32;
                if left_size + right_size > MAX_SIZE {
                    // forcibly generate a ParseIntError
                    Err("420".parse::<u8>().unwrap_err())
                } else {
                    Ok((left << right_size) | right)
                }
            },
        )
        .padded_by(just('_').repeated())
}

fn num_with_separators_ignored<'s, const RADIX: u32>() -> Lexer!['s, Token] {
    text::digits(RADIX)
        .ignored()
        .separated_by(just('_').ignored().repeated())
        .allow_leading()
        .allow_trailing()
        .map(|_| Token::Err)
}

fn partial_num<'s, const RADIX: u32>() -> Lexer!['s, Result<u64, ParseIntError>] {
    text::digits(RADIX).slice().map(move |x| u64::from_str_radix(x, RADIX))
}

fn num_radix_bits_in<const RADIX: u32>(x: u64) -> u32 {
    if x == 0 {
        0
    } else {
        (64 - x.leading_zeros()).next_multiple_of(RADIX.trailing_zeros())
    }
}
