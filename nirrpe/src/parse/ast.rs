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
    Assignment(Assignment),
    ControlFlow(ControlFlow),
    Error,
}

#[derive(Clone, Debug)]
pub enum Decl {
    LetDecl(LetDecl),
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
pub struct LetDecl {
    pub name: Ident,
    pub value: Expr,
}

#[derive(Clone, Debug)]
pub enum Expr {
    Lit(Lit),
    Var {
        name: Ident,
    },
    UnaryOp {
        ops: Vec<UnaryOp>,
        input: Box<Expr>,
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
    Block {
        body: Vec<Stmt>,
    },
    If {
        condition: Box<Expr>,
        body: Vec<Stmt>,
        r#else: Option<Box<Expr>>,
    },
    Loop {
        body: Vec<Stmt>,
    },
    While {
        condition: Box<Expr>,
        body: Vec<Stmt>,
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

#[derive(Assoc, Copy, Clone, Eq, PartialEq, Debug)]
#[func(pub const fn from_char(x: char) -> Option<Self>)]
pub enum UnaryOp {
    #[assoc(from_char = '+')]
    Plus,
    #[assoc(from_char = '-')]
    Minus,
    #[assoc(from_char = '!')]
    Not,
    #[assoc(from_char = '~')]
    BitNot,
}

#[derive(Assoc, Copy, Clone, Eq, PartialEq, Debug, ConstParamTy)]
#[func(pub const fn from_char(x: char) -> Option<Self>)]
#[func(pub const fn allows_assignment(&self) -> bool { false })]
pub enum BinaryOp {
    // arithmetic
    #[assoc(from_char = '+')]
    #[assoc(allows_assignment = true)]
    Add,
    #[assoc(from_char = '-')]
    #[assoc(allows_assignment = true)]
    Sub,
    #[assoc(from_char = '*')]
    #[assoc(allows_assignment = true)]
    Mul,
    #[assoc(from_char = '/')]
    #[assoc(allows_assignment = true)]
    Div,
    #[assoc(allows_assignment = true)]
    Pow,
    #[assoc(from_char = '%')]
    #[assoc(allows_assignment = true)]
    Rem,
    // bitwise
    #[assoc(from_char = '&')]
    #[assoc(allows_assignment = true)]
    BitAnd,
    #[assoc(from_char = '|')]
    #[assoc(allows_assignment = true)]
    BitOr,
    #[assoc(from_char = '^')]
    #[assoc(allows_assignment = true)]
    Xor,
    #[assoc(allows_assignment = true)]
    Shl,
    #[assoc(allows_assignment = true)]
    Shr,
    #[assoc(from_char = '⟲')]
    #[assoc(allows_assignment = true)]
    Rol,
    #[assoc(from_char = '⟳')]
    #[assoc(allows_assignment = true)]
    Ror,
    // logical
    #[assoc(allows_assignment = true)]
    And,
    #[assoc(allows_assignment = true)]
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

#[derive(Clone, Debug)]
pub struct Assignment {
    pub name: Ident,
    pub value: Expr,
    pub op: Option<BinaryOp>,
}

#[derive(Clone, Debug)]
pub enum ControlFlow {
    Continue,
    Break(Option<Expr>),
    Return(Option<Expr>),
}
