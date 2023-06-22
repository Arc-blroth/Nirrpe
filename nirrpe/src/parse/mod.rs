pub mod ast;
pub mod ident;
pub mod lexer;
pub mod token;
pub mod utils;

use chumsky::prelude::{choice, end, just, recursive, SimpleSpan};
use chumsky::{select, Parser};
use token::Token;

use crate::parse::ast::{BinaryOp, Expr, Lit};
use crate::parse::token::Ctrl;

macro Parser($s:lifetime, $output:ty) {
    impl ::chumsky::Parser<
        $s,
        ::chumsky::input::SpannedInput<
            crate::parse::token::Token,
            ::chumsky::span::SimpleSpan,
            &$s [(crate::parse::token::Token, ::chumsky::span::SimpleSpan)],
        >,
        $output,
        ::chumsky::extra::Err<
            chumsky::error::Rich<
                $s,
                crate::parse::token::Token,
                ::chumsky::span::SimpleSpan
            >
        >
    > + ::core::clone::Clone
}

pub type Spanned<T> = (T, SimpleSpan);

pub fn parser<'s>() -> Parser!['s, Expr] {
    recursive(|expr| {
        let atom = select! {
            Token::Int(x) => Expr::Lit(Lit::Int(x)),
        }
        .labelled("value")
        .or(expr.delimited_by(just(Token::Ctrl(Ctrl::LeftParen)), just(Token::Ctrl(Ctrl::RightParen))));

        let pow = binary_ops!(atom, [BinaryOp::Pow]);

        let unary = pow;

        let more_binary_ops = binary_ops!(
            unary,
            [BinaryOp::Mul, BinaryOp::Div, BinaryOp::Rem],
            [BinaryOp::Add, BinaryOp::Sub],
            [BinaryOp::Shl, BinaryOp::Shr],
            [BinaryOp::Rol, BinaryOp::Ror],
            [BinaryOp::BitAnd],
            [BinaryOp::Xor],
            [BinaryOp::BitOr],
            [
                BinaryOp::Eq,
                BinaryOp::Neq,
                BinaryOp::Lt,
                BinaryOp::Lte,
                BinaryOp::Gt,
                BinaryOp::Gte
            ],
            [BinaryOp::And],
            [BinaryOp::Or],
        );

        more_binary_ops
    })
    .then_ignore(end())
}

fn binary_op<const O: BinaryOp>(left: Box<Expr>, right: Box<Expr>) -> Expr {
    Expr::BinaryOp { op: O, left, right }
}

macro binary_ops($atom:expr, [$($op:expr),+$(,)?]$(,)?$(,$($rest:tt)+)?) {{
    let out = $atom.clone().foldl(
        choice([
            $(
                just(Token::Op($op)).to(binary_op::<{ $op }> as fn(_, _) -> _)
            ),+
        ])
        .then($atom)
        .repeated(),
        |left, (op, right)| op(Box::new(left), Box::new(right)),
    )
    // TODO(arc) I literally don't have enough memory to compile the parser on my computer
    //           without short-circuiting the type system here, but allocations cringe ;(
    .boxed();
    $(
        let out = binary_ops!(out, $($rest)+);
    )?
    out
}}
