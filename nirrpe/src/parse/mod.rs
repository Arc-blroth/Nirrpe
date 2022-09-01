pub mod ast;
pub mod ident;
pub mod lit;
pub mod util;

use lasso::Rodeo;
use pest::error::Error;
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;

use crate::parse::ast::{Expr, Program, Stmt};
use crate::parse::ident::Ident;
use crate::parse::lit::Lit;
use crate::parse::util::GetSingleInner;

#[derive(Parser)]
#[grammar = "parse/nirrpe.pest"]
pub struct NirrpeParser;

pub type ParseResult<T> = Result<T, Error<Rule>>;

// Generic trait for parsing AST nodes.
pub trait Parseable: Sized {
    /// Parses the given rule into this AST node.
    ///
    /// # Panics
    /// If the given rule does not match the type of this node.
    fn parse(rodeo: &mut Rodeo, pair: Pair<Rule>) -> ParseResult<Self>;
}

impl Program {
    pub fn parse<S: AsRef<str>>(program: S) -> Result<(Rodeo, Program), Error<Rule>> {
        let mut rodeo = Rodeo::new();
        let mut pairs = NirrpeParser::parse(Rule::program, program.as_ref())?;
        let program = <Self as Parseable>::parse(&mut rodeo, pairs.next().unwrap())?;
        Ok((rodeo, program))
    }
}

impl Parseable for Program {
    fn parse(rodeo: &mut Rodeo, pair: Pair<Rule>) -> ParseResult<Self> {
        assert_eq!(pair.as_rule(), Rule::program);
        let mut stmts = Vec::new();
        for pair in pair.into_inner() {
            if pair.as_rule() != Rule::EOI {
                stmts.push(Stmt::parse(rodeo, pair)?);
            }
        }
        Ok(Self { stmts })
    }
}

impl Parseable for Stmt {
    fn parse(rodeo: &mut Rodeo, pair: Pair<Rule>) -> ParseResult<Self> {
        assert_eq!(pair.as_rule(), Rule::stmt);
        let inner = pair.into_single_inner();
        match inner.as_rule() {
            Rule::expr => Ok(Stmt::Expr(Expr::parse(rodeo, inner)?)),
            _ => unreachable!(),
        }
    }
}

impl Parseable for Expr {
    fn parse(rodeo: &mut Rodeo, pair: Pair<Rule>) -> ParseResult<Self> {
        assert_eq!(pair.as_rule(), Rule::expr);
        let inner = pair.into_single_inner();
        match inner.as_rule() {
            Rule::methodCall => {
                let mut inner = inner.into_inner();
                let name = Ident::parse(rodeo, inner.next().unwrap())?;
                let mut args = Vec::new();
                for arg in inner {
                    args.push(Expr::parse(rodeo, arg)?);
                }
                Ok(Expr::MethodCall { name, args })
            }
            Rule::lit => Ok(Expr::Lit(Lit::parse(rodeo, inner)?)),
            _ => unreachable!(),
        }
    }
}
