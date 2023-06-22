use crate::parse::ast::{BinaryOp, Expr, Lit};

impl Expr {
    pub fn execute(&self) -> u64 {
        match self {
            Expr::MethodCall { .. } => unimplemented!("you can't actually execute this yet"),
            Expr::BinaryOp { op, left, right } => {
                let left = left.execute();
                let right = right.execute();
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
            Expr::Lit(lit) => {
                match lit {
                    Lit::Int(x) => *x,
                    _ => unimplemented!(),
                }
            }
        }
    }
}