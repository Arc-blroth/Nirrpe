use enum_assoc::Assoc;

use crate::parse::ast::BinaryOp;

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
}

#[derive(Assoc, Clone, Debug, PartialEq)]
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
    #[assoc(from_char = ';')]
    Semicolon,
    #[assoc(from_char = ',')]
    Comma,
    #[assoc(from_char = '.')]
    Period,
}
