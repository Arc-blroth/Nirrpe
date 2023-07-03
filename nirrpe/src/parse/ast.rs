use std::marker::ConstParamTy;

use bitflags::bitflags;
use enum_assoc::Assoc;

use crate::parse::ident::Ident;

bitflags! {
    #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
    pub struct Modifiers: u32 {
        const EXTERN = 1 << 0;
        const PUB    = 1 << 1;
        const PRIV   = 1 << 2;
        const PURE   = 1 << 3;
        const IMPURE = 1 << 4;
    }
}

#[derive(Clone, Debug)]
pub struct Program {
    pub stmts: Vec<Stmt>,
}

#[derive(Clone, Debug)]
pub enum Stmt {
    Decl(Decl),
    Expr(Expr),
}

#[derive(Clone, Debug)]
pub enum Decl {
    FnDecl(FnDecl),
}

#[derive(Clone, Debug)]
pub struct FnDecl {
    pub modifiers: Modifiers,
    pub name: Ident,
    pub args: Vec<FnArg>,
    pub return_ty: Option<Ident>,
    pub body: Option<Vec<Stmt>>,
}

#[derive(Clone, Debug)]
pub struct FnArg {
    pub name: Ident,
    pub ty: Ident,
    pub variadic: bool,
}

#[derive(Clone, Debug)]
pub enum Expr {
    Lit(Lit),
    Var {
        name: Ident,
    },
    BinaryOp {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Call {
        target: Box<Expr>,
        args: Vec<Expr>,
    },
    Error,
}

#[derive(Clone, Debug)]
pub enum Lit {
    Unit,
    Bool(bool),
    Char(char),
    Int(u64),
    Float(f64),
    Str(String),
}

#[derive(Assoc, Copy, Clone, Eq, PartialEq, Debug, ConstParamTy)]
#[func(pub const fn from_char(x: char) -> Option<Self>)]
pub enum BinaryOp {
    // arithmetic
    #[assoc(from_char = '+')]
    Add,
    #[assoc(from_char = '-')]
    Sub,
    #[assoc(from_char = '*')]
    Mul,
    #[assoc(from_char = '/')]
    Div,
    Pow,
    #[assoc(from_char = '%')]
    Rem,
    // bitwise
    #[assoc(from_char = '&')]
    BitAnd,
    #[assoc(from_char = '|')]
    BitOr,
    #[assoc(from_char = '^')]
    Xor,
    Shl,
    Shr,
    #[assoc(from_char = '⟲')]
    Rol,
    #[assoc(from_char = '⟳')]
    Ror,
    // logical
    And,
    Or,
    // comparison
    Eq,
    Neq,
    #[assoc(from_char = '<')]
    Lt,
    Lte,
    #[assoc(from_char = '>')]
    Gt,
    Gte,
}
