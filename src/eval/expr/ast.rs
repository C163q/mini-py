use std::sync::Arc;

use num_bigint::BigInt;

use crate::{
    Interpreter,
    eval::basic::ast::LValue,
    types::{float::PyFloat, int::PyInt},
};

/// The top-level expression node, wrapping the operator-precedence hierarchy.
///
/// The hierarchy from lowest to highest precedence is:
/// `Expr` → `LOrExpr` → `LAndExpr` → `LNotExpr` → `EqExpr` → `RelExpr` → `AddExpr`
/// → `MulExpr` → `UnaryExpr` → `PowExpr` → `PrimaryExpr`.
#[derive(Debug, Clone)]
pub struct Expr {
    pub value: LOrExpr,
}

macro_rules! impl_expr {
    ($name:ident, $val_ty:ty, $($param_name:ident, $param_ty:ty),+) => {
        impl Expr {
            pub fn $name(value: $val_ty) -> Self {
                Self { value }
            }

            $(
                pub fn $param_name(value: $param_ty) -> Self {
                    Self {
                        value: <$val_ty>::$param_name(value),
                    }
                }
            )+
        }
    };
}

impl_expr!(
    new_or,
    LOrExpr,
    new_and,
    LAndExpr,
    new_not,
    LNotExpr,
    new_eq,
    EqExpr,
    new_rel,
    RelExpr,
    new_add,
    AddExpr,
    new_mul,
    MulExpr,
    new_unary,
    UnaryExpr,
    new_pow,
    PowExpr,
    new_primary,
    PrimaryExpr,
    new_number,
    Number
);

/// A logical-OR expression: `<a> or <b>`, or a bare `LAndExpr` if no `or` is present.
#[derive(Debug, Clone)]
pub enum LOrExpr {
    Or(Box<LAndExpr>, Box<LOrExpr>),
    And(LAndExpr),
}

impl LOrExpr {
    pub fn new_or(left: LAndExpr, right: LOrExpr) -> Self {
        Self::Or(Box::new(left), Box::new(right))
    }

    pub fn new_and(expr: LAndExpr) -> Self {
        Self::And(expr)
    }

    pub fn new_not(expr: LNotExpr) -> Self {
        Self::And(LAndExpr::new_not(expr))
    }

    pub fn new_eq(expr: EqExpr) -> Self {
        Self::And(LAndExpr::new_eq(expr))
    }

    pub fn new_rel(expr: RelExpr) -> Self {
        Self::And(LAndExpr::new_rel(expr))
    }

    pub fn new_add(expr: AddExpr) -> Self {
        Self::And(LAndExpr::new_add(expr))
    }

    pub fn new_mul(expr: MulExpr) -> Self {
        Self::And(LAndExpr::new_mul(expr))
    }

    pub fn new_unary(expr: UnaryExpr) -> Self {
        Self::And(LAndExpr::new_unary(expr))
    }

    pub fn new_pow(expr: PowExpr) -> Self {
        Self::And(LAndExpr::new_pow(expr))
    }

    pub fn new_primary(expr: PrimaryExpr) -> Self {
        Self::And(LAndExpr::new_primary(expr))
    }

    pub fn new_number(num: Number) -> Self {
        Self::And(LAndExpr::new_number(num))
    }
}

/// A logical-AND expression: `<a> and <b>`, or a bare `LNotExpr` if no `and` is present.
#[derive(Debug, Clone)]
pub enum LAndExpr {
    And(Box<LNotExpr>, Box<LAndExpr>),
    Not(LNotExpr),
}

impl LAndExpr {
    pub fn new_and(left: LNotExpr, right: LAndExpr) -> Self {
        Self::And(Box::new(left), Box::new(right))
    }

    pub fn new_not(expr: LNotExpr) -> Self {
        Self::Not(expr)
    }

    pub fn new_eq(expr: EqExpr) -> Self {
        Self::Not(LNotExpr::new_eq(expr))
    }

    pub fn new_rel(expr: RelExpr) -> Self {
        Self::Not(LNotExpr::new_rel(expr))
    }

    pub fn new_add(expr: AddExpr) -> Self {
        Self::Not(LNotExpr::new_add(expr))
    }

    pub fn new_mul(expr: MulExpr) -> Self {
        Self::Not(LNotExpr::new_mul(expr))
    }

    pub fn new_unary(expr: UnaryExpr) -> Self {
        Self::Not(LNotExpr::new_unary(expr))
    }

    pub fn new_pow(expr: PowExpr) -> Self {
        Self::Not(LNotExpr::new_pow(expr))
    }

    pub fn new_primary(expr: PrimaryExpr) -> Self {
        Self::Not(LNotExpr::new_primary(expr))
    }

    pub fn new_number(num: Number) -> Self {
        Self::Not(LNotExpr::new_number(num))
    }
}

/// A logical-NOT expression: `not <a>`, or a bare `EqExpr` if no `not` is present.
#[derive(Debug, Clone)]
pub enum LNotExpr {
    Not(Box<LNotExpr>),
    Eq(EqExpr),
}

impl LNotExpr {
    pub fn new_not(expr: LNotExpr) -> Self {
        Self::Not(Box::new(expr))
    }

    pub fn new_eq(expr: EqExpr) -> Self {
        Self::Eq(expr)
    }

    pub fn new_rel(expr: RelExpr) -> Self {
        Self::Eq(EqExpr::new_rel(expr))
    }

    pub fn new_add(expr: AddExpr) -> Self {
        Self::Eq(EqExpr::new_add(expr))
    }

    pub fn new_mul(expr: MulExpr) -> Self {
        Self::Eq(EqExpr::new_mul(expr))
    }

    pub fn new_unary(expr: UnaryExpr) -> Self {
        Self::Eq(EqExpr::new_unary(expr))
    }

    pub fn new_pow(expr: PowExpr) -> Self {
        Self::Eq(EqExpr::new_pow(expr))
    }

    pub fn new_primary(expr: PrimaryExpr) -> Self {
        Self::Eq(EqExpr::new_primary(expr))
    }

    pub fn new_number(num: Number) -> Self {
        Self::Eq(EqExpr::new_number(num))
    }
}

/// Equality comparison operators: `==` and `!=`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EqOp {
    Eq,
    NotEq,
}

/// An equality comparison: `<a> == <b>`, `<a> != <b>`, or a bare `RelExpr`.
#[derive(Debug, Clone)]
pub enum EqExpr {
    Expr {
        left: Box<RelExpr>,
        op: EqOp,
        right: Box<EqExpr>,
    },
    Rel(RelExpr),
}

impl EqExpr {
    pub fn new_eq(left: RelExpr, op: EqOp, right: EqExpr) -> Self {
        Self::Expr {
            left: Box::new(left),
            op,
            right: Box::new(right),
        }
    }

    pub fn new_rel(expr: RelExpr) -> Self {
        Self::Rel(expr)
    }

    pub fn new_add(expr: AddExpr) -> Self {
        Self::Rel(RelExpr::new_add(expr))
    }

    pub fn new_mul(expr: MulExpr) -> Self {
        Self::Rel(RelExpr::new_mul(expr))
    }

    pub fn new_unary(expr: UnaryExpr) -> Self {
        Self::Rel(RelExpr::new_unary(expr))
    }

    pub fn new_pow(expr: PowExpr) -> Self {
        Self::Rel(RelExpr::new_pow(expr))
    }

    pub fn new_primary(expr: PrimaryExpr) -> Self {
        Self::Rel(RelExpr::new_primary(expr))
    }

    pub fn new_number(num: Number) -> Self {
        Self::Rel(RelExpr::new_number(num))
    }
}

/// Relational comparison operators: `<`, `>`, `<=`, `>=`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelOp {
    Lt,
    Gt,
    Le,
    Ge,
}

/// A relational comparison: `<a> < <b>` etc., or a bare `AddExpr`.
#[derive(Debug, Clone)]
pub enum RelExpr {
    Expr {
        left: Box<AddExpr>,
        op: RelOp,
        right: Box<RelExpr>,
    },
    Add(AddExpr),
}

impl RelExpr {
    pub fn new_rel(left: AddExpr, op: RelOp, right: RelExpr) -> Self {
        Self::Expr {
            left: Box::new(left),
            op,
            right: Box::new(right),
        }
    }

    pub fn new_add(expr: AddExpr) -> Self {
        Self::Add(expr)
    }

    pub fn new_mul(expr: MulExpr) -> Self {
        Self::Add(AddExpr::new_mul(expr))
    }

    pub fn new_unary(expr: UnaryExpr) -> Self {
        Self::Add(AddExpr::new_unary(expr))
    }

    pub fn new_pow(expr: PowExpr) -> Self {
        Self::Add(AddExpr::new_pow(expr))
    }

    pub fn new_primary(expr: PrimaryExpr) -> Self {
        Self::Add(AddExpr::new_primary(expr))
    }

    pub fn new_number(num: Number) -> Self {
        Self::Add(AddExpr::new_number(num))
    }
}

/// Additive operators: `+` and `-`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddOp {
    Add,
    Sub,
}

/// An additive expression: `<a> + <b>`, `<a> - <b>`, or a bare `MulExpr`.
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

    pub fn new_pow(expr: PowExpr) -> Self {
        Self::Mul(MulExpr::new_pow(expr))
    }

    pub fn new_primary(expr: PrimaryExpr) -> Self {
        Self::Mul(MulExpr::new_primary(expr))
    }

    pub fn new_number(num: Number) -> Self {
        Self::Mul(MulExpr::new_number(num))
    }
}

/// Multiplicative operators: `*`, `/`, `//`, `%`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MulOp {
    Mul,
    TrueDiv,
    FloorDiv,
    Mod,
}

/// A multiplicative expression: `<a> * <b>` etc., or a bare `UnaryExpr`.
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

    pub fn new_pow(expr: PowExpr) -> Self {
        Self::Unary(UnaryExpr::new_pow(expr))
    }

    pub fn new_primary(expr: PrimaryExpr) -> Self {
        Self::Unary(UnaryExpr::new_primary(expr))
    }

    pub fn new_number(num: Number) -> Self {
        Self::Unary(UnaryExpr::new_number(num))
    }
}

/// Unary operators: `+`, `-`, `~`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Pos,
    Neg,
    BitNot,
    // TODO
}

/// A unary expression: `+<a>`, `-<a>`, `~<a>`, or a bare `PowExpr`.
#[derive(Debug, Clone)]
pub enum UnaryExpr {
    Expr { op: UnaryOp, expr: Box<UnaryExpr> },
    Pow(PowExpr),
}

impl UnaryExpr {
    pub fn new_unary(op: UnaryOp, expr: UnaryExpr) -> Self {
        Self::Expr {
            op,
            expr: Box::new(expr),
        }
    }

    pub fn new_pow(expr: PowExpr) -> Self {
        Self::Pow(expr)
    }

    pub fn new_primary(expr: PrimaryExpr) -> Self {
        Self::Pow(PowExpr::new_primary(expr))
    }

    pub fn new_number(num: Number) -> Self {
        Self::Pow(PowExpr::new_number(num))
    }
}

/// An exponentiation expression: `<a> ** <b>`, or a bare `PrimaryExpr`.
#[derive(Debug, Clone)]
pub enum PowExpr {
    Expr {
        left: Box<PrimaryExpr>,
        right: Box<PowExpr>,
    },
    Primary(PrimaryExpr),
}

impl PowExpr {
    pub fn new_pow(left: PrimaryExpr, right: PowExpr) -> Self {
        Self::Expr {
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    pub fn new_primary(expr: PrimaryExpr) -> Self {
        Self::Primary(expr)
    }

    pub fn new_number(num: Number) -> Self {
        Self::Primary(PrimaryExpr::new_number(num))
    }
}

/// The highest-precedence expression form: a parenthesised expression, a numeric literal,
/// `None`, a string literal, or a variable name.
#[derive(Debug, Clone)]
pub enum PrimaryExpr {
    Expr(Box<Expr>),
    Number(Number),
    None,
    Str(String),
    LValue(LValue),
}

impl PrimaryExpr {
    pub fn new_expr(expr: Expr) -> Self {
        Self::Expr(Box::new(expr))
    }

    pub fn new_number(num: Number) -> Self {
        Self::Number(num)
    }

    pub fn new_none() -> Self {
        Self::None
    }

    pub fn new_str(s: String) -> Self {
        Self::Str(s)
    }

    pub fn new_lvalue(lvalue: LValue) -> Self {
        Self::LValue(lvalue)
    }
}

/// A numeric literal: either an integer or a floating-point value.
#[derive(Debug, Clone)]
pub enum Number {
    Int(PyInt),
    Float(PyFloat),
}

impl Number {
    pub fn new_int(interpreter: Arc<Interpreter>, num: BigInt) -> Self {
        Self::Int(PyInt::new(interpreter, num))
    }

    pub fn new_float(interpreter: Arc<Interpreter>, num: f64) -> Self {
        Self::Float(PyFloat::new(interpreter, num))
    }
}

/// A `None` literal.
pub struct NoneExpr;
