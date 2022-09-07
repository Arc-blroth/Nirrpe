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
            Expr::MethodCall { name, args } => {
                match name.resolve(&self.rodeo) {
                    "print" => {
                        if args.len() == 1 && let Expr::Lit(lit) = &args[0] {
                            match lit {
                                Lit::String(x) => println!("{x}"),
                                Lit::Char(x) => println!("{x}"),
                                Lit::Int(x) => println!("{x}"),
                                Lit::Float(x) => println!("{x}"),
                            }
                            Ok(())
                        } else {
                            Err(NirrpeError::RuntimeError("invalid arguments".to_string()))
                        }
                    }
                    _ => Err(NirrpeError::RuntimeError("methods don't actually exist yet lmao".to_string())),
                }
            }
            Expr::Lit(_) => {
                Ok(())
            }
        }
    }
}
