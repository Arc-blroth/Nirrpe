use crate::parse::ident::Ident;
use crate::parse::lit::Lit;

#[derive(Debug)]
pub struct Program {
    pub stmts: Vec<Stmt>,
}

#[derive(Debug)]
pub enum Stmt {
    Expr(Expr),
}

#[derive(Debug)]
pub enum Expr {
    MethodCall { name: Ident, args: Vec<Expr> },
    Lit(Lit),
}
