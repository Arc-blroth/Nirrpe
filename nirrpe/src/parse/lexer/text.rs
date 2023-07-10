use chumsky::prelude::{any, just, none_of, via_parser};
use chumsky::{IterParser, Parser};

use crate::parse::lexer::token::Token;
use crate::parse::lexer::Lexer;
use crate::parse::utils::{just_str, n_digits, recover_delimited_by, ParserTryUnwrapped};

const REPLACEMENT: char = '\u{fffd}';

pub fn lexer<'s>() -> Lexer!['s, Token] {
    let char_non_escape = none_of(r#""\"#);

    let char_x_escape = unicode_fixed_width_escape('x', 2);
    let char_u_escape = unicode_fixed_width_escape('u', 4);
    let char_u_long_escape = just_str("\\u").ignore_then(
        n_digits(16, 1..=6)
            .slice()
            .map(|x| u32::from_str_radix(x, 16))
            .unwrapped()
            .map(char::try_from)
            .try_unwrapped()
            .delimited_by(just('{'), just('}'))
            .recover_with(via_parser(recover_delimited_by('{', '}').to(REPLACEMENT))),
    );

    let char_single_escape = just('\\')
        .ignore_then(any())
        .map(|c: char| {
            Ok(match c {
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
                _ => return Err("unrecognized single character escape"),
            })
        })
        .try_unwrapped();

    let char_not_control_meta_escape = char_non_escape
        .or(char_x_escape)
        .or(char_u_long_escape)
        .or(char_u_escape)
        .or(char_single_escape);

    let char_control_escape_start = just('\\')
        .ignore_then(just('c').ignored().or(just_str("C-").ignored()))
        .ignored();
    let meta_control_escape_start = just('\\').ignore_then(just_str("M-")).ignored();
    let char_control_escape = char_control_escape_start
        .clone()
        .ignore_then(char_not_control_meta_escape.clone())
        .map(|x| u8::try_from(x).map_err(|_| "Invalid control character escape"))
        .try_unwrapped()
        // SAFETY: all 5-bit chars are valid
        .map(|x| unsafe { char::from_u32_unchecked((x & 0x1f) as u32) })
        .recover_with(via_parser(
            char_control_escape_start
                .clone()
                .ignore_then(char_not_control_meta_escape.clone().or_not().ignored())
                .to(REPLACEMENT),
        ));
    let char_meta_escape = meta_control_escape_start
        .clone()
        .ignore_then(char_not_control_meta_escape.clone())
        .map(|x| u8::try_from(x).map_err(|_| "Invalid meta character escape"))
        .try_unwrapped()
        // SAFETY: all 1-byte chars are valid
        .map(|x| unsafe { char::from_u32_unchecked((x | 0x80) as u32) })
        .recover_with(via_parser(
            meta_control_escape_start
                .clone()
                .ignore_then(char_not_control_meta_escape.clone().or_not().ignored())
                .to(REPLACEMENT),
        ));
    let char_meta_control_escape_start = char_control_escape_start
        .clone()
        .ignore_then(meta_control_escape_start.clone())
        .or(meta_control_escape_start.ignore_then(char_control_escape_start));
    let char_meta_control_escape = char_meta_control_escape_start
        .clone()
        .ignore_then(char_not_control_meta_escape.clone())
        .map(|x| u8::try_from(x).map_err(|_| "Invalid meta control character escape"))
        .try_unwrapped()
        // SAFETY: all 1-byte chars are valid
        .map(|x| unsafe { char::from_u32_unchecked(((x & 0x1f) | 0x80) as u32) })
        .recover_with(via_parser(
            char_meta_control_escape_start
                .ignore_then(char_not_control_meta_escape.clone().or_not().ignored())
                .to(REPLACEMENT),
        ));

    let char = char_not_control_meta_escape
        .or(char_meta_control_escape)
        .or(char_control_escape)
        .or(char_meta_escape);

    let string_lit = char
        .clone()
        .repeated()
        .collect()
        .padded_by(just('"'))
        .map(Token::Str)
        .recover_with(via_parser(recover_delimited_by('"', '"').to(Token::Err)))
        .labelled("string literal");
    #[rustfmt::skip]
    let char_lit = char
        .padded_by(just('\''))
        .map(Token::Char)
        .recover_with(via_parser(recover_delimited_by('\'', '\'').to(Token::Char(REPLACEMENT))))
        .labelled("char literal");

    string_lit.or(char_lit)
}

fn unicode_fixed_width_escape<'s>(escape: char, width: usize) -> Lexer!['s, char] {
    let prefix = just('\\').ignore_then(just(escape)).ignored();
    prefix
        .then(n_digits(16, 1..=width))
        .slice()
        .map(move |x: &str| {
            (x.len() - 2 == width)
                .then_some(&x[2..])
                .ok_or_else(|| format!("truncated \\{}{} escape", escape, "X".repeat(width)))
        })
        .try_unwrapped()
        .map(|x| u32::from_str_radix(x, 16))
        .unwrapped()
        .map(char::try_from)
        .try_unwrapped()
        .recover_with(via_parser(prefix.then(n_digits(16, 1..=width)).to(REPLACEMENT)))
}
