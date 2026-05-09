use std::sync::Arc;

use num_bigint::BigInt;

use crate::{Interpreter, types::int::PyInt};

#[derive(Debug, Clone)]
pub struct Expr {
    pub value: AddExpr,
}

impl Expr {
    pub fn new_add(value: AddExpr) -> Self {
        Self { value }
    }

    pub fn new_mul(value: MulExpr) -> Self {
        Self {
            value: AddExpr::new_mul(value),
        }
    }

    pub fn new_unary(value: UnaryExpr) -> Self {
        Self {
            value: AddExpr::new_unary(value),
        }
    }

    pub fn new_primary(value: PrimaryExpr) -> Self {
        Self {
            value: AddExpr::new_primary(value),
        }
    }

    pub fn new_number(num: Number) -> Self {
        Self {
            value: AddExpr::new_number(num),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddOp {
    Add,
    Sub,
}

#[derive(Debug, Clone)]
pub enum AddExpr {
    Expr {
        left: Box<MulExpr>,
        op: AddOp,
        right: Box<AddExpr>,
    },
    Mul(MulExpr),
}

impl AddExpr {
    pub fn new_add(left: MulExpr, op: AddOp, right: AddExpr) -> Self {
        Self::Expr {
            left: Box::new(left),
            op,
            right: Box::new(right),
        }
    }

    pub fn new_mul(expr: MulExpr) -> Self {
        Self::Mul(expr)
    }

    pub fn new_unary(expr: UnaryExpr) -> Self {
        Self::Mul(MulExpr::new_unary(expr))
    }

    pub fn new_primary(expr: PrimaryExpr) -> Self {
        Self::Mul(MulExpr::new_primary(expr))
    }

    pub fn new_number(num: Number) -> Self {
        Self::Mul(MulExpr::new_number(num))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MulOp {
    Mul,
    // TODO: TrueDiv,
    FloorDiv,
    Mod,
}

#[derive(Debug, Clone)]
pub enum MulExpr {
    Expr {
        left: Box<UnaryExpr>,
        op: MulOp,
        right: Box<MulExpr>,
    },
    Unary(UnaryExpr),
}

impl MulExpr {
    pub fn new_mul(left: UnaryExpr, op: MulOp, right: MulExpr) -> Self {
        Self::Expr {
            left: Box::new(left),
            op,
            right: Box::new(right),
        }
    }

    pub fn new_unary(expr: UnaryExpr) -> Self {
        Self::Unary(expr)
    }

    pub fn new_primary(expr: PrimaryExpr) -> Self {
        Self::Unary(UnaryExpr::new_primary(expr))
    }

    pub fn new_number(num: Number) -> Self {
        Self::Unary(UnaryExpr::new_number(num))
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
