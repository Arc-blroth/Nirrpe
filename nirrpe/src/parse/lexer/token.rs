use std::marker::ConstParamTy;

use enum_assoc::Assoc;

use crate::parse::ast::BinaryOp;
use crate::parse::ident::Ident;

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Err,
    Bool(bool),
    Char(char),
    Int(u64),
    Float(f64),
    Str(String),
    Op(BinaryOp),
    Ctrl(Ctrl),
    Keyword(Keyword),
    Ident(Ident),
}

#[derive(Assoc, Clone, Debug, PartialEq, Eq, ConstParamTy)]
#[func(pub const fn from_char(x: char) -> Option<Self>)]
pub enum Ctrl {
    #[assoc(from_char = '(')]
    LeftParen,
    #[assoc(from_char = ')')]
    RightParen,
    #[assoc(from_char = '[')]
    LeftBracket,
    #[assoc(from_char = ']')]
    RightBracket,
    #[assoc(from_char = '{')]
    LeftBrace,
    #[assoc(from_char = '}')]
    RightBrace,
    #[assoc(from_char = ':')]
    Colon,
    #[assoc(from_char = ';')]
    Semicolon,
    #[assoc(from_char = ',')]
    Comma,
    #[assoc(from_char = '.')]
    Period,
    #[assoc(from_char = '=')]
    Eq,
}

#[derive(Assoc, Clone, Debug, PartialEq)]
#[func(pub const fn keyword(&self) -> &'static str)]
pub enum Keyword {
    #[assoc(keyword = "break")]
    Break,
    #[assoc(keyword = "continue")]
    Continue,
    #[assoc(keyword = "else")]
    Else,
    #[assoc(keyword = "extern")]
    Extern,
    #[assoc(keyword = "fn")]
    Fn,
    #[assoc(keyword = "for")]
    For,
    #[assoc(keyword = "if")]
    If,
    #[assoc(keyword = "in")]
    In,
    #[assoc(keyword = "impure")]
    Impure,
    #[assoc(keyword = "loop")]
    Loop,
    #[assoc(keyword = "priv")]
    Priv,
    #[assoc(keyword = "pub")]
    Pub,
    #[assoc(keyword = "pure")]
    Pure,
    #[assoc(keyword = "return")]
    Return,
    #[assoc(keyword = "while")]
    While,
}
