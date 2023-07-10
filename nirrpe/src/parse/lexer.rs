mod numbers;
mod text;
pub mod token;

use chumsky::prelude::{any, choice, end, one_of, skip_then_retry_until, IterParser, Parser};

use crate::parse::ast::BinaryOp;
use crate::parse::ident::Ident;
use crate::parse::lexer::token::{Ctrl, Keyword, Token};
use crate::parse::utils::just_str;
use crate::parse::Spanned;

pub(super) macro Lexer($s:lifetime, $output:ty) {
    impl ::chumsky::Parser<$s, &$s str, $output, ::chumsky::extra::Err<chumsky::error::Rich<$s, char, ::chumsky::span::SimpleSpan>>> + ::core::clone::Clone
}

pub fn lexer<'s>() -> Lexer!['s, Vec<Spanned<Token>>] {
    let bool = just_str("true")
        .to(Token::Bool(true))
        .or(just_str("false").to(Token::Bool(false)))
        .labelled("bool");

    let op = just_str("**")
        .to(BinaryOp::Pow)
        .or(just_str("<<").to(BinaryOp::Shl))
        .or(just_str(">>").to(BinaryOp::Shr))
        .or(just_str("==").to(BinaryOp::Eq))
        .or(just_str("!=").to(BinaryOp::Neq))
        .or(just_str("<=").to(BinaryOp::Lte))
        .or(just_str(">=").to(BinaryOp::Gte))
        .or(choice([just_str("&&"), just_str("and")]).to(BinaryOp::And))
        .or(choice([just_str("||"), just_str("or")]).to(BinaryOp::Or))
        .or(one_of("+-*/%&|^⟲⟳<>").map(BinaryOp::from_char).unwrapped())
        .map(Token::Op)
        .labelled("op");

    let ctrl = one_of("()[]{}:;,.=").map(Ctrl::from_char).unwrapped().map(Token::Ctrl);

    let keyword = just_str(Keyword::Extern.keyword())
        .to(Keyword::Extern)
        .or(just_str(Keyword::Fn.keyword()).to(Keyword::Fn))
        .or(just_str(Keyword::Impure.keyword()).to(Keyword::Impure))
        .or(just_str(Keyword::Priv.keyword()).to(Keyword::Priv))
        .or(just_str(Keyword::Pub.keyword()).to(Keyword::Pub))
        .or(just_str(Keyword::Pure.keyword()).to(Keyword::Pure))
        .map(Token::Keyword)
        .labelled("keyword");

    let ident = chumsky::text::unicode::ident()
        .map(Ident::new)
        .map(Token::Ident)
        .labelled("identifier");

    ctrl.or(op)
        .or(bool)
        .or(keyword)
        .or(ident)
        .or(numbers::lexer())
        .or(text::lexer())
        .map_with_span(|a, b| (a, b))
        .padded()
        .recover_with(skip_then_retry_until(any().ignored(), end()))
        .repeated()
        .collect()
}
