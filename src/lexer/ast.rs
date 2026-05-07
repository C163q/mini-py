use std::sync::Arc;

use num_bigint::BigInt;

use crate::{Interpreter, types::int::PyInt};

#[derive(Debug, Clone)]
pub struct Expr {
    pub value: UnaryExpr,
}

impl Expr {
    pub fn new_unary(value: UnaryExpr) -> Self {
        Self { value }
    }

    pub fn new_primary(value: PrimaryExpr) -> Self {
        Self {
            value: UnaryExpr::new_primary(value),
        }
    }

    pub fn new_number(num: Number) -> Self {
        Self {
            value: UnaryExpr::new_number(num),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Pos,
    Neg,
    // TODO
}

#[derive(Debug, Clone)]
pub enum UnaryExpr {
    Expr { op: UnaryOp, expr: Box<UnaryExpr> },
    Primary(PrimaryExpr),
}

impl UnaryExpr {
    pub fn new_primary(expr: PrimaryExpr) -> Self {
        Self::Primary(expr)
    }

    pub fn new_unary(op: UnaryOp, expr: UnaryExpr) -> Self {
        Self::Expr {
            op,
            expr: Box::new(expr),
        }
    }

    pub fn new_number(num: Number) -> Self {
        Self::Primary(PrimaryExpr::new_number(num))
    }
}

#[derive(Debug, Clone)]
pub enum PrimaryExpr {
    Expr(Box<Expr>),
    Number(Number),
}

impl PrimaryExpr {
    pub fn new_expr(expr: Expr) -> Self {
        Self::Expr(Box::new(expr))
    }

    pub fn new_number(num: Number) -> Self {
        Self::Number(num)
    }
}

#[derive(Debug, Clone)]
pub enum Number {
    Int(PyInt), // TODO: support float
}

impl Number {
    pub fn new_int(interpreter: Arc<Interpreter>, num: BigInt) -> Self {
        Self::Int(PyInt::new(interpreter, num))
    }
}
