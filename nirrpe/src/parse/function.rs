use lasso::Rodeo;
use pest::iterators::Pair;

use crate::parse::ast::{Expr, FnArg, FnDecl, Stmt};
use crate::parse::ident::Ident;
use crate::parse::modifier::Modifiers;
use crate::parse::{Parsable, ParseResult, Rule};

impl Parsable for FnDecl {
    fn parse(rodeo: &mut Rodeo, pair: Pair<Rule>) -> ParseResult<Self> {
        assert_eq!(pair.as_rule(), Rule::fnDecl);
        let mut pairs = pair.into_inner();
        let modifiers = Modifiers::parse_greedy(rodeo, &mut pairs)?;
        let name = Ident::parse(rodeo, pairs.next().unwrap())?;
        let mut args = Vec::new();
        while pairs.peek().map(|x| x.as_rule()) == Some(Rule::fnArg) {
            args.push(FnArg::parse(rodeo, pairs.next().unwrap())?);
        }
        if pairs.peek().map(|x| x.as_rule()) == Some(Rule::fnVariadicArgEllipse) {
            let mut variadic_arg = FnArg::parse(rodeo, pairs.nth(1).unwrap())?;
            variadic_arg.variadic = true;
            args.push(variadic_arg);
        }
        let body = match pairs.next() {
            Some(pair) if pair.as_rule() == Rule::expr => Some(vec![Stmt::Expr(Expr::parse(rodeo, pair)?)]),
            Some(pair) if pair.as_rule() == Rule::fnBody => {
                let mut stmts = Vec::new();
                let stmt_pairs = pair.into_inner();
                for stmt_pair in stmt_pairs {
                    stmts.push(Stmt::parse(rodeo, stmt_pair)?);
                }
                Some(stmts)
            }
            None => None,
            _ => unreachable!(),
        };
        Ok(FnDecl {
            modifiers,
            name,
            args,
            body,
        })
    }
}

impl Parsable for FnArg {
    fn parse(rodeo: &mut Rodeo, pair: Pair<Rule>) -> ParseResult<Self> {
        assert_eq!(pair.as_rule(), Rule::fnArg);
        let mut pairs = pair.into_inner();
        let name = Ident::parse(rodeo, pairs.next().unwrap())?;
        let ty = Ident::parse(rodeo, pairs.next().unwrap())?;
        Ok(FnArg {
            name,
            ty,
            variadic: false,
        })
    }
}
