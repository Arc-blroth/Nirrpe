use std::str::FromStr;

use lasso::Rodeo;
use pest::error::{Error, ErrorVariant};
use pest::iterators::Pair;

use crate::parse::util::GetSingleInner;
use crate::parse::{Parsable, ParseResult, Rule};

#[derive(Debug)]
pub enum Lit {
    String(String),
    Char(char),
    Int(u64),
    Float(f64),
}

impl Parsable for Lit {
    fn parse(_rodeo: &mut Rodeo, pair: Pair<Rule>) -> ParseResult<Self> {
        assert_eq!(pair.as_rule(), Rule::lit);
        let inner = pair.into_single_inner();
        match inner.as_rule() {
            Rule::stringLit => Ok(Lit::String(parse_string_lit(inner)?)),
            Rule::charLit => Ok(Lit::Char(parse_char_lit(inner)?)),
            Rule::numLit => parse_num_lit(inner),
            _ => unreachable!(),
        }
    }
}

fn parse_string_lit(pair: Pair<Rule>) -> ParseResult<String> {
    assert_eq!(pair.as_rule(), Rule::stringLit);
    let pairs = pair.into_inner();
    let mut chars = Vec::new();
    for inner_pair in pairs {
        chars.push(parse_char_lit(inner_pair)?);
    }
    Ok(String::from_iter(chars))
}

fn parse_char_lit(pair: Pair<Rule>) -> ParseResult<char> {
    assert_eq!(pair.as_rule(), Rule::stringLitChar);
    let inner = pair.into_single_inner();
    let inner_str = inner.as_str();
    let mut chars = inner_str.chars();
    match inner.as_rule() {
        Rule::stringLitCharNonEscape => Ok(chars.next().unwrap()),
        Rule::stringLitCharSingleEscape => {
            let char = match chars.nth(1).unwrap() {
                '0' => '\0',
                'a' => '\u{0007}',
                'b' => '\u{0008}',
                'e' => '\u{001b}',
                'f' => '\u{000c}',
                'n' => '\n',
                'r' => '\r',
                's' => ' ',
                't' => '\t',
                'v' => '\u{000b}',
                '\'' => '\'',
                '\"' => '\"',
                '\\' => '\\',
                _ => unreachable!("unrecognized single character escape"),
            };
            Ok(char)
        }
        Rule::stringLitCharUnicodeEscape => u32::from_str_radix(
            match chars.nth(2).unwrap() {
                '{' => &inner_str[3..inner_str.len() - 1],
                _ => &inner_str[2..],
            },
            16,
        )
        .ok()
        .and_then(char::from_u32)
        .ok_or_else(|| {
            Error::new_from_span(
                ErrorVariant::CustomError {
                    message: "invalid unicode character escape".to_string(),
                },
                inner.as_span(),
            )
        }),
        Rule::stringLitCharControlEscape => {
            let rhs_pair = inner.into_single_inner();
            let rhs_span = rhs_pair.as_span();
            let rhs: u8 = parse_char_lit(rhs_pair)?.try_into().map_err(|_| {
                Error::new_from_span(
                    ErrorVariant::CustomError {
                        message: "invalid control character escape".to_string(),
                    },
                    rhs_span,
                )
            })?;
            // SAFETY: all 5-bit chars are valid
            Ok(unsafe { char::from_u32_unchecked((rhs & 0b00011111) as u32) })
        }
        Rule::stringLitCharMetaEscape => {
            let is_reversed_meta_control = !inner_str.starts_with(r#"\M-"#);
            let rhs_pair = inner.into_single_inner();
            let rhs_span = rhs_pair.as_span();
            let rhs: u8 = parse_char_lit(rhs_pair)?.try_into().map_err(|_| {
                Error::new_from_span(
                    ErrorVariant::CustomError {
                        message: "invalid meta character escape".to_string(),
                    },
                    rhs_span,
                )
            })?;
            let rhs = if is_reversed_meta_control {
                rhs & 0b00011111
            } else {
                rhs
            };
            // SAFETY: all 1-byte chars are valid
            Ok(unsafe { char::from_u32_unchecked((rhs | 0b10000000) as u32) })
        }
        _ => unreachable!(),
    }
}

fn parse_num_lit(pair: Pair<Rule>) -> ParseResult<Lit> {
    assert_eq!(pair.as_rule(), Rule::numLit);
    let inner = pair.into_single_inner();
    let inner_without_sep = inner.as_str().replace('_', "");
    match inner.as_rule() {
        Rule::intLit => Ok(Lit::Int(u64::from_str(&inner_without_sep).unwrap())),
        Rule::binIntLit => Ok(Lit::Int(u64::from_str_radix(&inner_without_sep[2..], 2).unwrap())),
        Rule::hexIntLit => Ok(Lit::Int(u64::from_str_radix(&inner_without_sep[2..], 16).unwrap())),
        Rule::floatLit => Ok(Lit::Float(f64::from_str(&inner_without_sep).unwrap())),
        _ => unreachable!(),
    }
}
