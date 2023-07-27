use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;
use std::rc::Rc;

use crate::parse::ast::{FnDecl, Lit};
use crate::parse::ident::Ident;
use crate::runtime::utils::DelegateDebugToDisplay;
use crate::runtime::{runtime_panic, RuntimeControlFlow};

#[derive(Clone, Debug)]
pub enum Value {
    Bool(bool),
    I8(i8),
    U8(u8),
    I16(i16),
    U16(u16),
    I32(i32),
    U32(u32),
    I64(i64),
    U64(u64),
    F32(f32),
    F64(f64),
    Char(char),
    Str(String),
    Object(Rc<RefCell<Object>>),
    Array(Vec<Value>),
    Tuple(Vec<Value>),
    Function(FnDecl),
}

impl Value {
    pub fn unit() -> Self {
        Self::Tuple(Vec::new())
    }

    /// Gets a property on an Object.
    pub fn try_get_property(&self, prop: &Ident) -> Result<Value, RuntimeControlFlow> {
        match self {
            Value::Object(object) => match object.borrow().values.get(&prop.id) {
                Some(x) => Ok(x.clone()),
                None => runtime_panic!("property {:?} not found in object", prop),
            },
            _ => runtime_panic!("only objects can have properties"),
        }
    }
}

impl From<&Lit> for Value {
    fn from(lit: &Lit) -> Self {
        match lit {
            Lit::Unit => Self::unit(),
            Lit::Bool(x) => Self::Bool(*x),
            Lit::Char(x) => Self::Char(*x),
            Lit::Int(x) => Self::U64(*x),
            Lit::Float(x) => Self::F64(*x),
            Lit::Str(x) => Self::Str(x.clone()),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Bool(x) => Display::fmt(x, f),
            Value::I8(x) => Display::fmt(x, f),
            Value::U8(x) => Display::fmt(x, f),
            Value::I16(x) => Display::fmt(x, f),
            Value::U16(x) => Display::fmt(x, f),
            Value::I32(x) => Display::fmt(x, f),
            Value::U32(x) => Display::fmt(x, f),
            Value::I64(x) => Display::fmt(x, f),
            Value::U64(x) => Display::fmt(x, f),
            Value::F32(x) => Display::fmt(x, f),
            Value::F64(x) => Display::fmt(x, f),
            Value::Char(x) => Display::fmt(x, f),
            Value::Str(x) => Display::fmt(x, f),
            Value::Object(x) => Display::fmt(x.borrow().deref(), f),
            Value::Array(x) => f.debug_list().entries(x.iter().map(DelegateDebugToDisplay)).finish(),
            Value::Tuple(x) => {
                let mut tuple = f.debug_tuple("");
                for element in x {
                    tuple.field(&DelegateDebugToDisplay(element));
                }
                tuple.finish()
            }
            Value::Function(x) => write!(f, "[function {:?}]", x.name),
        }
    }
}

#[derive(Debug)]
pub struct Object {
    pub values: HashMap<String, Value>,
}

impl Display for Object {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("[object Object]")
    }
}
