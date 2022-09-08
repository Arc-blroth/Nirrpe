pub mod error;

use lasso::Rodeo;

use crate::parse::ast::{Expr, Program, Stmt};
use crate::parse::lit::Lit;
use crate::runtime::error::{NirrpeError, NirrpeResult};

#[derive(Default)]
pub struct NirrpeRuntime {
    pub rodeo: Rodeo,
}

impl NirrpeRuntime {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn execute_str<S: AsRef<str>>(&mut self, program_str: S) -> NirrpeResult {
        let program = Program::parse(&mut self.rodeo, program_str)?;
        self.execute(program)
    }

    pub fn execute(&mut self, program: Program) -> NirrpeResult {
        for stmt in program.stmts {
            #[allow(clippy::single_match)]
            match stmt {
                Stmt::Expr(expr) => self.handle_expr(expr)?,
            }
        }
        Ok(())
    }

    fn handle_expr(&mut self, expr: Expr) -> NirrpeResult {
        match expr {
            Expr::MethodCall { name, args } => match name.resolve(&self.rodeo) {
                "print" => {
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
                _ => Err(NirrpeError::RuntimeError(
                    "methods don't actually exist yet lmao".to_string(),
                )),
            },
            Expr::Lit(_) => Ok(()),
        }
    }
}
