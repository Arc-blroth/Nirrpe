pub mod error;

use std::collections::hash_map::OccupiedError;
use std::collections::HashMap;
use std::rc::Rc;

use lasso::Rodeo;

use crate::parse::ast::{Decl, Expr, FnDecl, Program, Stmt};
use crate::parse::ident::Ident;
use crate::parse::lit::Lit;
use crate::parse::modifier::Modifiers;
use crate::runtime::error::{NirrpeError, NirrpeResult};

pub type ExternFnImpl = dyn Fn(&mut NirrpeRuntime, Vec<Expr>) -> NirrpeResult;

#[derive(Default)]
pub struct NirrpeRuntime {
    pub rodeo: Rodeo,
    pub functions: HashMap<Ident, FnDecl>,
    extern_functions: HashMap<Ident, Rc<ExternFnImpl>>,
}

impl NirrpeRuntime {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_with_std_extern_functions() -> Self {
        let mut runtime = Self::new();
        runtime
            .extern_functions
            .insert(Ident::new(&mut runtime.rodeo, "print"), Rc::new(Self::print));
        runtime
    }

    pub fn execute_str<S: AsRef<str>>(&mut self, program_str: S) -> NirrpeResult {
        let program = Program::parse(&mut self.rodeo, program_str)?;
        self.execute(program)
    }

    pub fn execute(&mut self, program: Program) -> NirrpeResult {
        self.handle_stmts(program.stmts)
    }

    fn handle_stmts(&mut self, stmts: Vec<Stmt>) -> NirrpeResult {
        for stmt in stmts {
            match stmt {
                Stmt::Decl(decl) => self.handle_decl(decl)?,
                Stmt::Expr(expr) => self.handle_expr(expr)?,
            }
        }
        Ok(())
    }

    fn handle_decl(&mut self, decl: Decl) -> NirrpeResult {
        match decl {
            Decl::FnDecl(fn_decl) => {
                if fn_decl.modifiers.contains(Modifiers::EXTERN) && !self.extern_functions.contains_key(&fn_decl.name) {
                    return Err(NirrpeError::RuntimeError(format!(
                        "unknown extern function `{}`",
                        fn_decl.name.resolve(&self.rodeo)
                    )));
                }
                self.functions.try_insert(fn_decl.name, fn_decl).map(|_| ()).map_err(
                    |OccupiedError { value, .. }| {
                        NirrpeError::RuntimeError(format!(
                            "redefinition of function `{}`",
                            value.name.resolve(&self.rodeo)
                        ))
                    },
                )
            }
        }
    }

    fn handle_expr(&mut self, expr: Expr) -> NirrpeResult {
        match expr {
            Expr::MethodCall { name, args } => match self.functions.get(&name) {
                Some(function) => {
                    if function.modifiers.contains(Modifiers::EXTERN) {
                        (self.extern_functions[&function.name].clone())(self, args)
                    } else {
                        Err(NirrpeError::RuntimeError(
                            "you can't actually execute this yet".to_string(),
                        ))
                    }
                }
                None => Err(NirrpeError::RuntimeError(format!(
                    "undefined function `{}`",
                    name.resolve(&self.rodeo)
                ))),
            },
            Expr::Lit(_) => Ok(()),
        }
    }

    fn print(&mut self, args: Vec<Expr>) -> NirrpeResult {
        for arg in &args {
            if !matches!(arg, Expr::Lit(_)) {
                Err(NirrpeError::RuntimeError("invalid arguments".to_string()))?
            }
        }
        let mut strings = Vec::with_capacity(args.len());
        for arg in args {
            let Expr::Lit(lit) = arg else { unreachable!() };
            match lit {
                Lit::String(x) => strings.push(x),
                Lit::Char(x) => strings.push(x.to_string()),
                Lit::Int(x) => strings.push(x.to_string()),
                Lit::Float(x) => strings.push(x.to_string()),
            }
        }
        println!("{}", strings.join(" "));
        Ok(())
    }
}
