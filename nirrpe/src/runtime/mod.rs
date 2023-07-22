pub mod utils;
pub mod value;

use std::cell::RefCell;
use std::collections::HashMap;

use crate::parse::ast::{Assignment, BinaryOp, ControlFlow, Decl, Expr, Modifiers, Program, Stmt, UnaryOp};
use crate::parse::ident::Ident;
use crate::runtime::value::Value;

pub struct NirrpeRuntime<'r> {
    global: Scope<'r>,
}

impl<'r> NirrpeRuntime<'r> {
    pub fn new() -> Self {
        Self {
            global: Scope::global(),
        }
    }

    pub fn execute(&mut self, program: Program) {
        program.execute(&mut self.global);
    }
}

impl<'r> Default for NirrpeRuntime<'r> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug)]
pub enum RuntimeControlFlow {
    Continue,
    Break(Value),
    Return(Value),
    Panic(Value),
}

pub macro runtime_panic {
    ($msg:literal) => {
        return ::core::result::Result::Err(
            $crate::runtime::RuntimeControlFlow::Panic(
                $crate::runtime::value::Value::Str($msg.to_string())
            )
        )
    },
    ($msg:literal, $($args:tt)*) => {
        return ::core::result::Result::Err(
            $crate::runtime::RuntimeControlFlow::Panic(
                $crate::runtime::value::Value::Str(::std::format!($msg, $($args)*))
            )
        )
    },
}

pub struct Scope<'p> {
    parent: Option<&'p Scope<'p>>,
    variables: RefCell<HashMap<Ident, Value>>,
}

impl<'p> Scope<'p> {
    pub fn new(parent: &'p Scope<'p>) -> Self {
        Self {
            parent: Some(parent),
            variables: RefCell::new(HashMap::new()),
        }
    }

    pub fn global() -> Self {
        Self {
            parent: None,
            variables: RefCell::new(HashMap::new()),
        }
    }

    pub fn has_local_value(&self, name: &Ident) -> bool {
        self.variables.borrow().contains_key(name)
    }

    pub fn has_value(&self, name: &Ident) -> bool {
        match self.variables.borrow().contains_key(name) {
            true => true,
            false => self.parent.map_or(false, |p| p.has_value(name)),
        }
    }

    pub fn get_value(&self, name: &Ident) -> Option<Value> {
        match self.variables.borrow().get(name) {
            Some(x) => Some(x.clone()),
            None => self.parent.and_then(|p| p.get_value(name)),
        }
    }

    pub fn replace_value(&self, name: &Ident, value: Value) -> bool {
        match self.variables.borrow_mut().get_mut(name) {
            Some(x) => {
                *x = value;
                true
            }
            None => self.parent.map_or(false, move |p| p.replace_value(name, value)),
        }
    }
}

impl Program {
    pub fn execute(&self, scope: &mut Scope) {
        if let Err(err) = execute_stmts(&self.stmts, scope) {
            match err {
                RuntimeControlFlow::Panic(x) => {
                    eprintln!("Program panicked at '{}'", x);
                }
                err => {
                    eprintln!(
                        "Illegal top-level {}",
                        match err {
                            RuntimeControlFlow::Continue => "continue",
                            RuntimeControlFlow::Break(_) => "break",
                            RuntimeControlFlow::Return(_) => "return",
                            _ => unreachable!(),
                        }
                    );
                }
            }
        }
    }
}

fn execute_stmts(stmts: &Vec<Stmt>, scope: &mut Scope) -> Result<Value, RuntimeControlFlow> {
    let mut last_ret = Value::unit();
    for stmt in stmts {
        last_ret = stmt.execute(scope)?;
    }
    Ok(last_ret)
}

impl Stmt {
    pub fn execute(&self, scope: &mut Scope) -> Result<Value, RuntimeControlFlow> {
        match self {
            Stmt::Decl(decl) => match decl {
                Decl::LetDecl(r#let) => {
                    if scope.has_local_value(&r#let.name) {
                        runtime_panic!("variable {:?} already defined", r#let.name);
                    } else {
                        let value = r#let.value.execute(scope)?;
                        scope.variables.borrow_mut().insert(r#let.name.clone(), value);
                    }
                    Ok(Value::unit())
                }
                Decl::FnDecl(function) => {
                    if scope.has_local_value(&function.name) {
                        runtime_panic!("function {:?} already defined", function.name);
                    } else {
                        scope
                            .variables
                            .borrow_mut()
                            .insert(function.name.clone(), Value::Function(function.clone()));
                    }
                    Ok(Value::unit())
                }
            },
            Stmt::Expr(expr) => expr.execute(scope),
            Stmt::Assignment(Assignment { name, value, op }) => {
                if !scope.has_value(name) {
                    runtime_panic!("variable {:?} is undefined", name);
                }
                let mut result = value.execute(scope)?;
                if let Some(op) = op {
                    assert!(
                        op.allows_assignment(),
                        "invalid AST: given operator {:?} is not assignable!",
                        op
                    );
                    result = execute_builtin_binop(
                        *op,
                        scope
                            .get_value(name)
                            .unwrap_or_else(|| panic!("variable {:?} is undefined but was defined earlier?", name)),
                        result,
                    )?;
                }
                if !scope.replace_value(name, result) {
                    panic!("variable {:?} is undefined but was defined earlier?", name);
                }
                Ok(Value::unit())
            }
            Stmt::ControlFlow(flow) => match flow {
                ControlFlow::Continue => Err(RuntimeControlFlow::Continue),
                ControlFlow::Break(maybe_expr) => {
                    let value = match maybe_expr {
                        Some(expr) => expr.execute(scope)?,
                        None => Value::unit(),
                    };
                    Err(RuntimeControlFlow::Break(value))
                }
                ControlFlow::Return(maybe_expr) => {
                    let value = match maybe_expr {
                        Some(expr) => expr.execute(scope)?,
                        None => Value::unit(),
                    };
                    Err(RuntimeControlFlow::Return(value))
                }
            },
            Stmt::Error => runtime_panic!("Cannot execute AST with errors!"),
        }
    }
}

impl Expr {
    pub fn execute(&self, scope: &mut Scope) -> Result<Value, RuntimeControlFlow> {
        match self {
            Expr::Lit(lit) => Ok(lit.into()),
            Expr::Var { name } => match scope.get_value(name) {
                Some(x) => Ok(x),
                None => runtime_panic!("variable {:?} isn't defined", name),
            },
            Expr::UnaryOp { ops, input } => {
                let mut input = input.execute(scope)?;
                for op in ops.iter().rev() {
                    input = execute_builtin_unary_op(*op, input)?;
                }
                Ok(input)
            }
            Expr::BinaryOp { op, left, right } => {
                let left = left.execute(scope)?;
                let right = right.execute(scope)?;
                execute_builtin_binop(*op, left, right)
            }
            Expr::Call { target, args } => {
                let decl = match target.execute(scope)? {
                    Value::Function(decl) => decl,
                    _ => runtime_panic!("tried to call a non-function"),
                };

                let fun_name = &decl.name;
                if decl.args.len() != args.len() {
                    runtime_panic!("wrong number of arguments to function {:?}", fun_name);
                }

                let mut evaluated_args = HashMap::new();
                for arg in args.iter().enumerate() {
                    evaluated_args.insert(decl.args[arg.0].name.clone(), arg.1.execute(scope)?);
                }

                if let Some(stmts) = &decl.body {
                    let mut new_scope = Scope::new(scope);
                    evaluated_args.into_iter().for_each(|(k, v)| {
                        new_scope.variables.borrow_mut().insert(k, v);
                    });
                    match execute_stmts(stmts, &mut new_scope) {
                        Ok(x) | Err(RuntimeControlFlow::Return(x)) => Ok(x),
                        Err(RuntimeControlFlow::Continue) => runtime_panic!("Illegal continue outside loop"),
                        Err(RuntimeControlFlow::Break(_)) => runtime_panic!("Illegal break outside block or loop"),
                        x @ Err(RuntimeControlFlow::Panic(_)) => x,
                    }
                } else if decl.modifiers.contains(Modifiers::EXTERN) {
                    execute_builtin_function(fun_name, evaluated_args)
                } else {
                    runtime_panic!("function {:?} doesn't have a body", fun_name);
                }
            }
            Expr::Block { body } => {
                let mut new_scope = Scope::new(scope);
                match execute_stmts(body, &mut new_scope) {
                    Ok(x) | Err(RuntimeControlFlow::Break(x)) => Ok(x),
                    x => x,
                }
            }
            Expr::If {
                condition,
                body,
                r#else,
            } => {
                if match condition.execute(scope)? {
                    Value::Bool(x) => x,
                    _ => runtime_panic!("expected bool type for condition"),
                } {
                    let mut new_scope = Scope::new(scope);
                    execute_stmts(body, &mut new_scope)
                } else if let Some(r#else) = r#else {
                    let mut new_scope = Scope::new(scope);
                    r#else.execute(&mut new_scope)
                } else {
                    Ok(Value::unit())
                }
            }
            Expr::Loop { body } => loop {
                let mut new_scope = Scope::new(scope);
                match execute_stmts(body, &mut new_scope) {
                    Err(RuntimeControlFlow::Break(x)) => break Ok(x),
                    Err(RuntimeControlFlow::Continue) => {}
                    Err(x) => break Err(x),
                    _ => {}
                }
            },
            Expr::While { condition, body } => {
                while match condition.execute(scope)? {
                    Value::Bool(x) => x,
                    _ => runtime_panic!("expected bool type for condition"),
                } {
                    let mut new_scope = Scope::new(scope);
                    match execute_stmts(body, &mut new_scope) {
                        Err(RuntimeControlFlow::Break(x)) => return Ok(x),
                        Err(RuntimeControlFlow::Continue) => {}
                        Err(x) => return Err(x),
                        _ => {}
                    }
                }
                Ok(Value::unit())
            }
            _ => todo!(),
        }
    }
}

fn execute_builtin_binop(op: BinaryOp, left: Value, right: Value) -> Result<Value, RuntimeControlFlow> {
    if let Value::U64(left) = left && let Value::U64(right) = right {
        Ok(match op {
            BinaryOp::Add => Value::U64(left + right),
            BinaryOp::Sub => Value::U64(left - right),
            BinaryOp::Mul => Value::U64(left * right),
            BinaryOp::Div => Value::U64(left / right),
            BinaryOp::Pow => Value::U64(left.pow(right as u32)),
            BinaryOp::Rem => Value::U64(left % right),
            BinaryOp::BitAnd => Value::U64(left & right),
            BinaryOp::BitOr => Value::U64(left | right),
            BinaryOp::Xor => Value::U64(left ^ right),
            BinaryOp::Shl => Value::U64(left << right),
            BinaryOp::Shr => Value::U64(left >> right),
            BinaryOp::Rol => Value::U64(left.rotate_left(right as u32)),
            BinaryOp::Ror => Value::U64(left.rotate_right(right as u32)),
            BinaryOp::Eq => Value::Bool(left == right),
            BinaryOp::Neq => Value::Bool(left != right),
            BinaryOp::Lt => Value::Bool(left < right),
            BinaryOp::Lte => Value::Bool(left <= right),
            BinaryOp::Gt => Value::Bool(left > right),
            BinaryOp::Gte => Value::Bool(left >= right),
            _ => runtime_panic!("u64s can't do that"),
        })
    } else if let Value::Bool(left) = left && let Value::Bool(right) = right {
        Ok(match op {
            BinaryOp::And => Value::Bool(left && right),
            BinaryOp::Or => Value::Bool(left || right),
            BinaryOp::Eq => Value::Bool(left == right),
            BinaryOp::Neq => Value::Bool(left != right),
            _ => runtime_panic!("bools can't do that"),
        })
    } else {
        todo!()
    }
}

fn execute_builtin_unary_op(op: UnaryOp, input: Value) -> Result<Value, RuntimeControlFlow> {
    if let Value::U64(input) = input {
        Ok(match op {
            UnaryOp::Plus => Value::U64(input),
            UnaryOp::Minus => Value::I64(-(input as i64)),
            UnaryOp::BitNot => Value::U64(!input),
            _ => runtime_panic!("u64s can't do that"),
        })
    } else if let Value::Bool(input) = input {
        Ok(match op {
            UnaryOp::Not => Value::Bool(!input),
            _ => runtime_panic!("bools can't do that"),
        })
    } else {
        todo!()
    }
}

fn execute_builtin_function(name: &Ident, args: HashMap<Ident, Value>) -> Result<Value, RuntimeControlFlow> {
    match name.id.as_str() {
        "panic" => Err(RuntimeControlFlow::Panic(
            args.into_iter()
                .next()
                .map(|x| x.1)
                .unwrap_or(Value::Str("explicit panic".to_string())),
        )),
        "print" => {
            print!("{}", args.iter().next().unwrap().1);
            Ok(Value::unit())
        }
        "println" => {
            println!("{}", args.iter().next().unwrap().1);
            Ok(Value::unit())
        }
        _ => runtime_panic!("unknown builtin function {:?}", name),
    }
}
