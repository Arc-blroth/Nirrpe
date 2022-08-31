use crate::parse::ident::Ident;

#[derive(Debug)]
pub struct Program {
    pub stmts: Vec<Stmt>,
}

#[derive(Debug)]
pub enum Stmt {
    Expr { expr: Expr },
}

#[derive(Debug)]
pub enum Expr {
    MethodCall { name: Ident, args: Vec<Expr> },
}
