pub mod ast;
pub mod ident;
pub mod lexer;
pub mod utils;

use std::collections::HashMap;

use bitflags::Flags;
use chumsky::error::{Error as ChumskyError, Rich};
use chumsky::extra::Err as ChumskyErr;
use chumsky::input::SpannedInput;
use chumsky::label::LabelError;
use chumsky::prelude::{any, choice, end, just, nested_delimiters, recursive, via_parser, Recursive, SimpleSpan};
use chumsky::recursive::Direct;
use chumsky::util::MaybeRef;
use chumsky::{select, IterParser, Parser};
use ordinal::Ordinal;
use smallvec::SmallVec;

use crate::parse::ast::{
    Assignment, BinaryOp, ControlFlow, Decl, Expr, FnArg, FnDecl, LetDecl, Lit, Modifiers, ObjectPropName, Program,
    Stmt, UnaryOp,
};
use crate::parse::ident::Ident;
use crate::parse::lexer::token::{Ctrl, Keyword, Token};
use crate::parse::utils::SmallVecContainer;

type ParserInput<'s> = SpannedInput<Token, SimpleSpan, &'s [(Token, SimpleSpan)]>;
type ParserExtra<'s> = ChumskyErr<Rich<'s, Token, SimpleSpan, String>>;

macro Parser($s:lifetime, $output:ty $(, $($extra:lifetime),*$(,)?)?) {
    impl ::chumsky::Parser<
        $s,
        ::chumsky::input::SpannedInput<
            crate::parse::lexer::token::Token,
            ::chumsky::span::SimpleSpan,
            &$s [(crate::parse::lexer::token::Token, ::chumsky::span::SimpleSpan)],
        >,
        $output,
        ::chumsky::extra::Err<
            chumsky::error::Rich<
                $s,
                crate::parse::lexer::token::Token,
                ::chumsky::span::SimpleSpan,
                ::std::string::String,
            >
        >
    > + ::core::clone::Clone $($(+ $extra)*)?
}

pub type Spanned<T> = (T, SimpleSpan);

pub fn parser<'s>() -> Parser!['s, Program] {
    recursive(|stmts| {
        let expr = expr(stmts.clone());
        let ident = ident();

        let r#let = just(Token::Keyword(Keyword::Let))
            .ignore_then(ident.clone())
            .then_ignore(just(Token::Ctrl(Ctrl::Eq)))
            .then(expr.clone())
            .map(|(name, value)| Stmt::Decl(Decl::LetDecl(LetDecl { name, value })))
            .labelled("let declaration".into());
        let assignment = ident
            .separated_by(just(Token::Ctrl(Ctrl::Period)))
            .collect::<Vec<_>>()
            .then(
                any()
                    .try_map(|x, span| match x {
                        Token::Op(binop) => Ok(binop),
                        x => Err(ChumskyError::<ParserInput<'s>>::expected_found(
                            [],
                            Some(MaybeRef::Val(x)),
                            span,
                        )),
                    })
                    .or_not()
                    .then_ignore(just(Token::Ctrl(Ctrl::Eq)))
                    .validate(|x, span, emitter| {
                        if let Some(op) = x && !op.allows_assignment() {
                            emitter.emit(Rich::custom(
                                span,
                                format!("{:?} is not a valid assignment operator", op),
                            ))
                        }
                        x
                    }),
            )
            .then(expr.clone())
            .map(|((path, op), value)| Stmt::Assignment(Assignment { path, value, op }))
            .labelled("variable assignment".into());

        let r#continue = just(Token::Keyword(Keyword::Continue))
            .to(Stmt::ControlFlow(ControlFlow::Continue))
            .labelled("continue statement".into());
        let r#break = just(Token::Keyword(Keyword::Break))
            .ignore_then(expr.clone().or_not())
            .map(|x| Stmt::ControlFlow(ControlFlow::Break(x)))
            .labelled("break statement".into());
        let r#return = just(Token::Keyword(Keyword::Return))
            .ignore_then(expr.clone().or_not())
            .map(|x| Stmt::ControlFlow(ControlFlow::Return(x)))
            .labelled("return statement".into());

        decl(stmts)
            .map(Stmt::Decl)
            .or(r#let)
            .or(assignment)
            .or(expr.map(Stmt::Expr))
            .or(r#continue)
            .or(r#break)
            .or(r#return)
            .labelled("statement".into())
            .separated_by(just(Token::Ctrl(Ctrl::Semicolon)).repeated().ignored())
            .allow_leading()
            .allow_trailing()
            .collect()
    })
    .map(|x| Program { stmts: x })
    .then_ignore(end())
}

pub fn decl<'s>(stmts: Recursive<Direct<'s, 's, ParserInput<'s>, Vec<Stmt>, ParserExtra<'s>>>) -> Parser!['s, Decl] {
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
                    .clone()
                    .delimited_by(just(Token::Ctrl(Ctrl::LeftBrace)), just(Token::Ctrl(Ctrl::RightBrace)))
                    .recover_with(via_parser(
                        nested_recovery::<{ Ctrl::LeftBrace }, { Ctrl::RightBrace }>().map(|x| vec![Stmt::Expr(x)]),
                    ))
                    .or(just(Token::Ctrl(Ctrl::Eq))
                        .ignore_then(expr(stmts))
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

pub fn expr<'s>(stmts: Recursive<Direct<'s, 's, ParserInput<'s>, Vec<Stmt>, ParserExtra<'s>>>) -> Parser!['s, Expr] {
    recursive(|expr| {
        let inline_expr = recursive(|inline_expr| {
            let value = select! {
                Token::Bool(x) => Expr::Lit(Lit::Bool(x)),
                Token::Char(x) => Expr::Lit(Lit::Char(x)),
                Token::Int(x) => Expr::Lit(Lit::Int(x)),
                Token::Float(x) => Expr::Lit(Lit::Float(x)),
                Token::Str(x) => Expr::Lit(Lit::Str(x)),
            }
            .labelled("literal".into());

            let ident = ident();

            let object = ident
                .clone()
                .map_with_span(|x, span| (ObjectPropName::Ident(x), span))
                .or(expr
                    .clone()
                    .delimited_by(
                        just(Token::Ctrl(Ctrl::LeftBracket)),
                        just(Token::Ctrl(Ctrl::RightBracket)),
                    )
                    .recover_with(via_parser(nested_recovery::<
                        { Ctrl::LeftBracket },
                        { Ctrl::RightBracket },
                    >()))
                    .map_with_span(|x, span| (ObjectPropName::Expr(x), span)))
                .then_ignore(just(Token::Ctrl(Ctrl::Colon)))
                .then(expr.clone())
                .separated_by(just(Token::Ctrl(Ctrl::Comma)))
                .allow_trailing()
                .collect::<Vec<_>>()
                .validate(|x, _, emitter| {
                    let mut props = Vec::new();
                    let mut duplicates = HashMap::new();
                    for ((name, span), expr) in x {
                        if let ObjectPropName::Ident(name_ident) = &name {
                            duplicates
                                .entry(name_ident.clone())
                                .or_insert_with(SmallVec::<[SimpleSpan; 3]>::new)
                                .push(span);
                        }
                        props.push((name, expr));
                    }
                    for (name, spans) in duplicates {
                        if spans.len() > 1 {
                            let mut err =
                                Rich::custom(spans[1], format!("property `{}' specified more than once", name.id,));
                            <Rich<_, _, String> as LabelError<'s, ParserInput<'s>, _>>::in_context(
                                &mut err,
                                "first usage found here".into(),
                                spans[0],
                            );
                            if spans.len() > 2 {
                                for span in spans.iter().enumerate().skip(2) {
                                    <Rich<_, _, String> as LabelError<'s, ParserInput<'s>, _>>::in_context(
                                        &mut err,
                                        format!("{} additional usage found here", Ordinal(span.0 + 1),),
                                        *span.1,
                                    );
                                }
                            }
                            emitter.emit(err);
                        }
                    }
                    Expr::Object { props }
                })
                .delimited_by(just(Token::Ctrl(Ctrl::LeftBrace)), just(Token::Ctrl(Ctrl::RightBrace)))
                // don't use recovery here in case this is a block expression
                .labelled("object".into());

            let var = ident.clone().map(|name| Expr::Var { name }).labelled("variable".into());

            let exprs = expr
                .clone()
                .separated_by(just(Token::Ctrl(Ctrl::Comma)))
                .allow_trailing()
                .collect::<Vec<_>>();

            let atom = value
                .or(object)
                .or(var)
                .or(inline_expr.delimited_by(just(Token::Ctrl(Ctrl::LeftParen)), just(Token::Ctrl(Ctrl::RightParen))));

            let dot = atom.foldl(
                just(Token::Ctrl(Ctrl::Period)).ignore_then(ident.clone()).repeated(),
                |left, right| Expr::Dot {
                    left: Box::new(left),
                    right,
                },
            );

            let call = dot.foldl(
                exprs
                    .delimited_by(just(Token::Ctrl(Ctrl::LeftParen)), just(Token::Ctrl(Ctrl::RightParen)))
                    .repeated(),
                |target, args| Expr::Call {
                    target: Box::new(target),
                    args,
                },
            );

            let pow = binary_ops!(call, [BinaryOp::Pow]);

            let unary = just(Token::Op(BinaryOp::Add))
                .to(UnaryOp::Plus)
                .or(just(Token::Op(BinaryOp::Sub)).to(UnaryOp::Minus))
                .or(just(Token::UnaryOp(UnaryOp::Not)).to(UnaryOp::Not))
                .or(just(Token::UnaryOp(UnaryOp::BitNot)).to(UnaryOp::BitNot))
                .repeated()
                .collect::<Vec<_>>()
                .then(pow)
                .map(|(ops, pow)| {
                    if !ops.is_empty() {
                        Expr::UnaryOp {
                            ops,
                            input: Box::new(pow),
                        }
                    } else {
                        pow
                    }
                });

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

        let block = stmts.delimited_by(just(Token::Ctrl(Ctrl::LeftBrace)), just(Token::Ctrl(Ctrl::RightBrace)));

        let expr_block = block
            .clone()
            .map(|body| Expr::Block { body })
            .recover_with(via_parser(
                nested_recovery::<{ Ctrl::LeftBrace }, { Ctrl::RightBrace }>(),
            ))
            .labelled("block expression".into());

        let stmts_block = block.clone().recover_with(via_parser(
            nested_recovery::<{ Ctrl::LeftBrace }, { Ctrl::RightBrace }>().to(vec![Stmt::Error]),
        ));

        let if_block = just(Token::Keyword(Keyword::If))
            .ignore_then(expr.clone())
            .then(stmts_block.clone())
            .then(just(Token::Keyword(Keyword::Else)).ignore_then(expr.clone()).or_not())
            .map(|((condition, body), r#else)| Expr::If {
                condition: Box::new(condition),
                body,
                r#else: r#else.map(Box::new),
            })
            .labelled("if block".into());

        let loop_block = just(Token::Keyword(Keyword::Loop))
            .ignore_then(stmts_block.clone())
            .map(|body| Expr::Loop { body })
            .labelled("loop block".into());

        let while_block = just(Token::Keyword(Keyword::While))
            .ignore_then(expr.clone())
            .then(stmts_block.clone())
            .map(|(condition, body)| Expr::While {
                condition: Box::new(condition),
                body,
            })
            .labelled("while block".into());

        if_block.or(loop_block).or(while_block).or(inline_expr).or(expr_block)
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
            panic!("invalid arguments");
        }
    }

    nested_delimiters(
        Token::Ctrl(LEFT),
        Token::Ctrl(RIGHT),
        other_closers::<LEFT, RIGHT>(),
        |_| Expr::Error,
    )
}
