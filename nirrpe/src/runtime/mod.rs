use std::collections::HashMap;
use std::sync::Arc;

use crate::parse::ast::{BinaryOp, Decl, Expr, FnDecl, Lit, Modifiers, Program, Stmt};
use crate::parse::ident::Ident;

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

pub struct Scope<'p> {
    parent: Option<&'p Scope<'p>>,
    variables: HashMap<Ident, u64>,
    functions: HashMap<Ident, Arc<FnDecl>>,
}

impl<'p> Scope<'p> {
    pub fn new(parent: &'p Scope<'p>) -> Self {
        Self {
            parent: Some(parent),
            variables: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    pub fn global() -> Self {
        Self {
            parent: None,
            variables: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    pub fn get_value(&self, name: &Ident) -> Option<u64> {
        match self.variables.get(name) {
            Some(x) => Some(*x),
            None => self.parent.and_then(|p| p.get_value(name)),
        }
    }

    pub fn get_function(&self, name: &Ident) -> Option<Arc<FnDecl>> {
        match self.functions.get(name) {
            Some(x) => Some(x.clone()),
            None => self.parent.and_then(|p| p.get_function(name)),
        }
    }
}

impl Program {
    pub fn execute(&self, scope: &mut Scope) {
        execute_stmts(&self.stmts, scope);
    }
}

fn execute_stmts(stmts: &Vec<Stmt>, scope: &mut Scope) -> Option<u64> {
    let mut last_ret = None;
    for stmt in stmts {
        match stmt.execute(scope) {
            Some(x) => last_ret = x,
            None => break,
        }
    }
    last_ret
}

impl Stmt {
    pub fn execute(&self, scope: &mut Scope) -> Option<Option<u64>> {
        match self {
            Stmt::Decl(Decl::FnDecl(function)) => {
                if scope.functions.contains_key(&function.name) {
                    panic!("function {:?} already defined", function.name);
                } else {
                    scope
                        .functions
                        .insert(function.name.clone(), Arc::new(function.clone()));
                }
                Some(None)
            }
            Stmt::Expr(expr) => Some(Some(expr.execute(scope))),
        }
    }
}

impl Expr {
    pub fn execute(&self, scope: &mut Scope) -> u64 {
        match self {
            Expr::Lit(lit) => match lit {
                Lit::Int(x) => *x,
                _ => unimplemented!(),
            },
            Expr::Var { name } => match scope.get_value(name) {
                Some(x) => x,
                None => panic!("variable {:?} isn't defined", name),
            },
            Expr::BinaryOp { op, left, right } => {
                let left = left.execute(scope);
                let right = right.execute(scope);
                match op {
                    BinaryOp::Add => left + right,
                    BinaryOp::Sub => left - right,
                    BinaryOp::Mul => left * right,
                    BinaryOp::Div => left / right,
                    BinaryOp::Pow => left.pow(right as u32),
                    BinaryOp::Rem => left % right,
                    BinaryOp::BitAnd => left & right,
                    BinaryOp::BitOr => left | right,
                    BinaryOp::Xor => left ^ right,
                    BinaryOp::Shl => left << right,
                    BinaryOp::Shr => left >> right,
                    BinaryOp::Rol => left.rotate_left(right as u32),
                    BinaryOp::Ror => left.rotate_right(right as u32),
                    BinaryOp::And => unimplemented!(),
                    BinaryOp::Or => unimplemented!(),
                    BinaryOp::Eq => unimplemented!(),
                    BinaryOp::Neq => unimplemented!(),
                    BinaryOp::Lt => unimplemented!(),
                    BinaryOp::Lte => unimplemented!(),
                    BinaryOp::Gt => unimplemented!(),
                    BinaryOp::Gte => unimplemented!(),
                }
            }
            Expr::Call { target, args } => {
                let fun_name = match &**target {
                    Expr::Var { name } => name,
                    _ => unimplemented!("you can't actually execute this yet"),
                };
                match scope.get_function(fun_name) {
                    Some(decl) => {
                        assert_eq!(
                            decl.args.len(),
                            args.len(),
                            "wrong number of arguments to function {:?}",
                            fun_name
                        );

                        let mut evaluated_args = HashMap::new();
                        for arg in args.iter().enumerate() {
                            evaluated_args.insert(decl.args[arg.0].name.clone(), arg.1.execute(scope));
                        }

                        if let Some(stmts) = &decl.body {
                            let mut new_scope = Scope::new(scope);
                            evaluated_args.into_iter().for_each(|(k, v)| {
                                new_scope.variables.insert(k, v);
                            });
                            execute_stmts(stmts, &mut new_scope).unwrap_or(0)
                        } else if decl.modifiers.contains(Modifiers::EXTERN) {
                            execute_builtin_function(fun_name, evaluated_args)
                        } else {
                            panic!("function {:?} doesn't have a body", fun_name);
                        }
                    }
                    None => panic!("function {:?} isn't defined", fun_name),
                }
            }
            _ => todo!(),
        }
    }
}

fn execute_builtin_function(name: &Ident, args: HashMap<Ident, u64>) -> u64 {
    match name.id.as_str() {
        "print" => {
            println!("{}", args.iter().next().unwrap().1);
            0
        }
        _ => panic!("unknown builtin function {:?}", name),
    }
}
