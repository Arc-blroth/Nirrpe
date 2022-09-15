use crate::parse::ident::Ident;
use crate::parse::lit::Lit;
use crate::parse::modifier::Modifiers;

#[derive(Debug)]
pub struct Program {
    pub stmts: Vec<Stmt>,
}

#[derive(Debug)]
pub enum Stmt {
    Decl(Decl),
    Expr(Expr),
}

#[derive(Debug)]
pub enum Decl {
    FnDecl(FnDecl),
}

#[derive(Debug)]
pub struct FnDecl {
    pub modifiers: Modifiers,
    pub name: Ident,
    pub args: Vec<FnArg>,
    pub body: Option<Vec<Stmt>>,
}

#[derive(Debug)]
pub struct FnArg {
    pub name: Ident,
    pub ty: Ident,
    pub variadic: bool,
}

#[derive(Debug)]
pub enum Expr {
    MethodCall { name: Ident, args: Vec<Expr> },
    Lit(Lit),
}
