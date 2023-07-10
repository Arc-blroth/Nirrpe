pub mod ast;
pub mod ident;
pub mod lexer;
pub mod token;
pub mod utils;

use bitflags::Flags;
use chumsky::error::Rich;
use chumsky::extra::Err as ChumskyErr;
use chumsky::input::SpannedInput;
use chumsky::label::LabelError;
use chumsky::prelude::{choice, end, just, nested_delimiters, recursive, via_parser, Recursive, SimpleSpan};
use chumsky::recursive::Direct;
use chumsky::{select, IterParser, Parser};
use ordinal::Ordinal;
use smallvec::SmallVec;
use token::Token;

use crate::parse::ast::{BinaryOp, Decl, Expr, FnArg, FnDecl, Lit, Modifiers, Program, Stmt};
use crate::parse::ident::Ident;
use crate::parse::token::{Ctrl, Keyword};
use crate::parse::utils::SmallVecContainer;

type ParserInput<'s> = SpannedInput<Token, SimpleSpan, &'s [(Token, SimpleSpan)]>;
type ParserExtra<'s> = ChumskyErr<Rich<'s, Token, SimpleSpan, String>>;

macro Parser($s:lifetime, $output:ty $(, $($extra:lifetime),*$(,)?)?) {
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
                ::chumsky::span::SimpleSpan,
                ::std::string::String,
            >
        >
    > + ::core::clone::Clone $($(+ $extra)*)?
}

pub type Spanned<T> = (T, SimpleSpan);

pub fn parser<'s>() -> Parser!['s, Program] {
    recursive(|stmts| {
        decl(stmts)
            .map(Stmt::Decl)
            .or(expr().map(Stmt::Expr))
            .labelled("statement".into())
            .separated_by(just(Token::Ctrl(Ctrl::Semicolon)).repeated().ignored())
            .allow_leading()
            .allow_trailing()
            .collect()
    })
    .map(|x| Program { stmts: x })
    .then_ignore(end())
}

pub fn decl<'b, 's: 'b>(
    stmts: Recursive<Direct<'s, 'b, ParserInput<'s>, Vec<Stmt>, ParserExtra<'s>>>,
) -> Parser!['s, Decl, 'b] {
    let ident = ident();
    let r#fn = {
        let modifiers = choice([
            just::<'s, _, ParserInput<'s>, ParserExtra<'s>>(Token::Keyword(Keyword::Extern)).to(Modifiers::EXTERN),
            just(Token::Keyword(Keyword::Impure)).to(Modifiers::IMPURE),
            just(Token::Keyword(Keyword::Priv)).to(Modifiers::PRIV),
            just(Token::Keyword(Keyword::Pub)).to(Modifiers::PUB),
            just(Token::Keyword(Keyword::Pure)).to(Modifiers::PURE),
        ])
        .map_with_span(|modifiers, span| (modifiers, span))
        .repeated()
        .collect::<SmallVecContainer<[(Modifiers, SimpleSpan); Modifiers::FLAGS.len()]>>()
        .validate(|modifiers, _, emitter| {
            if modifiers.len() > 1 {
                let mut duplicates = SmallVec::<[SimpleSpan; 3]>::new();
                for flag in Modifiers::FLAGS {
                    for modifier in &*modifiers {
                        if flag.value() == &modifier.0 {
                            duplicates.push(modifier.1);
                        }
                    }
                    if duplicates.len() > 1 {
                        let modifier_name = flag.name().to_ascii_lowercase();
                        let mut err = Rich::custom(
                            duplicates[1],
                            format!(
                                "duplicate {} modifier{} found",
                                modifier_name,
                                if duplicates.len() > 2 { "s" } else { "" },
                            ),
                        );
                        <Rich<_, _, String> as LabelError<'s, ParserInput<'s>, _>>::in_context(
                            &mut err,
                            format!("first {} modifier found here", modifier_name),
                            duplicates[0],
                        );
                        if duplicates.len() > 2 {
                            for duplicate in duplicates.iter().enumerate().skip(2) {
                                <Rich<_, _, String> as LabelError<'s, ParserInput<'s>, _>>::in_context(
                                    &mut err,
                                    format!(
                                        "{} additional {} modifier found here",
                                        Ordinal(duplicate.0 + 1),
                                        modifier_name,
                                    ),
                                    *duplicate.1,
                                );
                            }
                        }
                        emitter.emit(err);
                    }
                    duplicates.clear();
                }
            }
            modifiers.iter().fold(Modifiers::empty(), |m, x| m | x.0)
        })
        .labelled("function modifiers".into());

        let args = ident
            .clone()
            .then_ignore(just(Token::Ctrl(Ctrl::Colon)))
            .then(ident.clone())
            .map(|(name, ty)| FnArg {
                name,
                ty,
                variadic: false,
            })
            .labelled("function argument".into())
            .separated_by(just(Token::Ctrl(Ctrl::Comma)))
            .allow_trailing()
            .collect()
            .delimited_by(just(Token::Ctrl(Ctrl::LeftParen)), just(Token::Ctrl(Ctrl::RightParen)))
            .labelled("function arguments".into());

        modifiers
            .then_ignore(just(Token::Keyword(Keyword::Fn)))
            .then(ident.clone().labelled("function name".into()))
            .then(args)
            .then(
                just(Token::Ctrl(Ctrl::Colon))
                    .ignore_then(ident)
                    .labelled("function return type".into())
                    .or_not(),
            )
            .then(
                stmts
                    .delimited_by(just(Token::Ctrl(Ctrl::LeftBrace)), just(Token::Ctrl(Ctrl::RightBrace)))
                    .recover_with(via_parser(
                        nested_recovery::<{ Ctrl::LeftBracket }, { Ctrl::RightBracket }>().map(|x| vec![Stmt::Expr(x)]),
                    ))
                    .or(just(Token::Ctrl(Ctrl::Eq))
                        .ignore_then(expr())
                        .map(|x| vec![Stmt::Expr(x)]))
                    .labelled("function body".into())
                    .or_not(),
            )
            .map(|((((modifiers, name), args), return_ty), body)| {
                Decl::FnDecl(FnDecl {
                    modifiers,
                    name,
                    args,
                    return_ty,
                    body,
                })
            })
            .labelled("function".into())
    };
    r#fn
}

pub fn expr<'s>() -> Parser!['s, Expr] {
    recursive(|expr| {
        let inline_expr = recursive(|inline_expr| {
            let value = select! {
                Token::Bool(x) => Expr::Lit(Lit::Bool(x)),
                Token::Char(x) => Expr::Lit(Lit::Char(x)),
                Token::Int(x) => Expr::Lit(Lit::Int(x)),
                Token::Float(x) => Expr::Lit(Lit::Float(x)),
                Token::Str(x) => Expr::Lit(Lit::Str(x)),
            }
            .labelled("value".into());

            let var = ident().map(|name| Expr::Var { name }).labelled("variable".into());

            let exprs = expr
                .clone()
                .separated_by(just(Token::Ctrl(Ctrl::Comma)))
                .allow_trailing()
                .collect::<Vec<_>>();

            let atom = value
                .or(var)
                .or(inline_expr.delimited_by(just(Token::Ctrl(Ctrl::LeftParen)), just(Token::Ctrl(Ctrl::RightParen))));

            let call = atom.foldl(
                exprs
                    .delimited_by(just(Token::Ctrl(Ctrl::LeftParen)), just(Token::Ctrl(Ctrl::RightParen)))
                    .repeated(),
                |target, args| Expr::Call {
                    target: Box::new(target),
                    args,
                },
            );

            let pow = binary_ops!(call, [BinaryOp::Pow]);

            let unary = pow;

            binary_ops!(
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
            )
        });

        let block = inline_expr
            .clone()
            .delimited_by(just(Token::Ctrl(Ctrl::LeftBrace)), just(Token::Ctrl(Ctrl::RightBrace)))
            .recover_with(via_parser(nested_recovery::<
                { Ctrl::LeftBracket },
                { Ctrl::RightBracket },
            >()));
        block.or(inline_expr)
    })
}

fn ident<'s>() -> Parser!['s, Ident] {
    select! {
        Token::Ident(x) => x,
    }
    .labelled("identifier".into())
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

fn nested_recovery<'s, const LEFT: Ctrl, const RIGHT: Ctrl>() -> Parser!['s, Expr] {
    const fn other_closers<const LEFT: Ctrl, const RIGHT: Ctrl>() -> [(Token, Token); 2] {
        if matches!(LEFT, Ctrl::LeftParen) && matches!(RIGHT, Ctrl::RightParen) {
            [
                (Token::Ctrl(Ctrl::LeftBracket), Token::Ctrl(Ctrl::RightBracket)),
                (Token::Ctrl(Ctrl::LeftBrace), Token::Ctrl(Ctrl::RightBrace)),
            ]
        } else if matches!(LEFT, Ctrl::LeftBracket) && matches!(RIGHT, Ctrl::RightBracket) {
            [
                (Token::Ctrl(Ctrl::LeftParen), Token::Ctrl(Ctrl::RightParen)),
                (Token::Ctrl(Ctrl::LeftBrace), Token::Ctrl(Ctrl::RightBrace)),
            ]
        } else if matches!(LEFT, Ctrl::LeftBrace) && matches!(RIGHT, Ctrl::RightBrace) {
            [
                (Token::Ctrl(Ctrl::LeftParen), Token::Ctrl(Ctrl::RightParen)),
                (Token::Ctrl(Ctrl::LeftBracket), Token::Ctrl(Ctrl::RightBracket)),
            ]
        } else {
            #[allow(unconditional_panic)]
            let _ = ["invalid arguments"][1];
            other_closers::<{ Ctrl::LeftParen }, { Ctrl::RightParen }>()
        }
    }

    nested_delimiters(
        Token::Ctrl(LEFT),
        Token::Ctrl(RIGHT),
        other_closers::<LEFT, RIGHT>(),
        |_| Expr::Error,
    )
}
