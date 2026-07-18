//! Expression evaluation — one `eval_*` function per AST node in the precedence hierarchy.
//! Each function dispatches on its enum variant, delegates to the next level down for the
//! pass-through case, and calls the appropriate dunder method for the operator case.

use std::sync::Arc;

use crate::{
    Interpreter,
    eval::ast::{
        AddExpr, AddOp, EqExpr, EqOp, Expr, LAndExpr, LNotExpr, LOrExpr, MulExpr, MulOp, Number,
        PowExpr, PrimaryExpr, RelExpr, RelOp, UnaryExpr, UnaryOp,
    },
    types::{error, none::PyNone, tbool::PyBool, tstr::PyStr},
    var::PyValue,
};

pub fn eval_number(
    _interpreter: Arc<Interpreter>,
    num: Number,
) -> Result<Arc<dyn PyValue>, Arc<dyn PyValue>> {
    match num {
        Number::Int(num) => Ok(Arc::new(num)),
        Number::Float(num) => Ok(Arc::new(num)),
    }
}

pub fn eval_primary_expr(
    interpreter: Arc<Interpreter>,
    expr: PrimaryExpr,
) -> Result<Arc<dyn PyValue>, Arc<dyn PyValue>> {
    match expr {
        PrimaryExpr::Number(num) => eval_number(interpreter, num),
        PrimaryExpr::Expr(expr) => eval_expr(interpreter, *expr),
        PrimaryExpr::None => Ok(Arc::new(PyNone::new(interpreter))),
        PrimaryExpr::Str(str) => Ok(Arc::new(PyStr::new(interpreter, str))),
        PrimaryExpr::LValue(lvalue) => interpreter.get_var(&lvalue.name),
    }
}

macro_rules! eval_binary {
    ($interpreter:ident, $lhs:expr, $eval_lhs:ident, $rhs:expr, $eval_rhs:ident, $func:literal) => {{
        let lhs = $eval_lhs($interpreter.clone(), $lhs)?;
        let rhs = $eval_rhs($interpreter.clone(), $rhs)?;
        let func = lhs.get_binding($interpreter.clone(), $func)?;
        $crate::var::call::call(func, $interpreter.clone(), vec![lhs, rhs])
    }};
}

pub fn eval_pow_expr(
    interpreter: Arc<Interpreter>,
    expr: PowExpr,
) -> Result<Arc<dyn PyValue>, Arc<dyn PyValue>> {
    match expr {
        PowExpr::Primary(expr) => eval_primary_expr(interpreter, expr),
        PowExpr::Expr { left, right } => {
            eval_binary!(
                interpreter,
                *left,
                eval_primary_expr,
                *right,
                eval_pow_expr,
                "__pow__"
            )
        }
    }
}

pub fn eval_unary_expr(
    interpreter: Arc<Interpreter>,
    expr: UnaryExpr,
) -> Result<Arc<dyn PyValue>, Arc<dyn PyValue>> {
    macro_rules! eval_unary {
        ($interpreter:ident, $expr:expr, $func:literal) => {{
            let expr_value = eval_unary_expr($interpreter.clone(), $expr)?;
            let func = expr_value.get_binding($interpreter.clone(), $func)?;
            $crate::var::call::call(func, $interpreter.clone(), vec![expr_value])
        }};
    }
    match expr {
        UnaryExpr::Pow(expr) => eval_pow_expr(interpreter, expr),
        UnaryExpr::Expr { op, expr } => match op {
            UnaryOp::Pos => {
                eval_unary!(interpreter, *expr, "__pos__")
            }
            UnaryOp::Neg => {
                eval_unary!(interpreter, *expr, "__neg__")
            }
            UnaryOp::BitNot => {
                eval_unary!(interpreter, *expr, "__invert__")
            }
        },
    }
}

pub fn eval_mul_expr(
    interpreter: Arc<Interpreter>,
    expr: MulExpr,
) -> Result<Arc<dyn PyValue>, Arc<dyn PyValue>> {
    match expr {
        MulExpr::Unary(expr) => eval_unary_expr(interpreter, expr),
        MulExpr::Expr { left, op, right } => match op {
            MulOp::Mul => {
                eval_binary!(
                    interpreter,
                    *left,
                    eval_unary_expr,
                    *right,
                    eval_mul_expr,
                    "__mul__"
                )
            }
            MulOp::FloorDiv => {
                eval_binary!(
                    interpreter,
                    *left,
                    eval_unary_expr,
                    *right,
                    eval_mul_expr,
                    "__floordiv__"
                )
            }
            MulOp::Mod => {
                eval_binary!(
                    interpreter,
                    *left,
                    eval_unary_expr,
                    *right,
                    eval_mul_expr,
                    "__mod__"
                )
            }
            MulOp::TrueDiv => {
                eval_binary!(
                    interpreter,
                    *left,
                    eval_unary_expr,
                    *right,
                    eval_mul_expr,
                    "__truediv__"
                )
            }
        },
    }
}

pub fn eval_add_expr(
    interpreter: Arc<Interpreter>,
    expr: AddExpr,
) -> Result<Arc<dyn PyValue>, Arc<dyn PyValue>> {
    match expr {
        AddExpr::Mul(expr) => eval_mul_expr(interpreter, expr),
        AddExpr::Expr { left, op, right } => match op {
            AddOp::Add => {
                eval_binary!(
                    interpreter,
                    *left,
                    eval_mul_expr,
                    *right,
                    eval_add_expr,
                    "__add__"
                )
            }
            AddOp::Sub => {
                eval_binary!(
                    interpreter,
                    *left,
                    eval_mul_expr,
                    *right,
                    eval_add_expr,
                    "__sub__"
                )
            }
        },
    }
}

pub fn eval_rel_expr(
    interpreter: Arc<Interpreter>,
    expr: RelExpr,
) -> Result<Arc<dyn PyValue>, Arc<dyn PyValue>> {
    match expr {
        RelExpr::Add(expr) => eval_add_expr(interpreter, expr),
        RelExpr::Expr { left, op, right } => match op {
            RelOp::Lt => {
                eval_binary!(
                    interpreter,
                    *left,
                    eval_add_expr,
                    *right,
                    eval_rel_expr,
                    "__lt__"
                )
            }
            RelOp::Gt => {
                eval_binary!(
                    interpreter,
                    *left,
                    eval_add_expr,
                    *right,
                    eval_rel_expr,
                    "__gt__"
                )
            }
            RelOp::Le => {
                eval_binary!(
                    interpreter,
                    *left,
                    eval_add_expr,
                    *right,
                    eval_rel_expr,
                    "__le__"
                )
            }
            RelOp::Ge => {
                eval_binary!(
                    interpreter,
                    *left,
                    eval_add_expr,
                    *right,
                    eval_rel_expr,
                    "__ge__"
                )
            }
        },
    }
}

pub fn eval_eq_expr(
    interpreter: Arc<Interpreter>,
    expr: EqExpr,
) -> Result<Arc<dyn PyValue>, Arc<dyn PyValue>> {
    match expr {
        EqExpr::Rel(expr) => eval_rel_expr(interpreter, expr),
        EqExpr::Expr { left, op, right } => match op {
            EqOp::Eq => {
                eval_binary!(
                    interpreter,
                    *left,
                    eval_rel_expr,
                    *right,
                    eval_eq_expr,
                    "__eq__"
                )
            }
            EqOp::NotEq => {
                eval_binary!(
                    interpreter,
                    *left,
                    eval_rel_expr,
                    *right,
                    eval_eq_expr,
                    "__ne__"
                )
            }
        },
    }
}

fn get_bool_value(
    interpreter: Arc<Interpreter>,
    value: Arc<dyn PyValue>,
) -> Result<bool, Arc<dyn PyValue>> {
    let bool_func = value.get_binding(interpreter.clone(), "__bool__")?;
    let bool_value = crate::var::call::call(bool_func, interpreter.clone(), vec![value])?;
    let bool_value = bool_value
        .as_any()
        .downcast_ref::<PyBool>()
        .ok_or_else(|| {
            error::get_type_error(interpreter, "__bool__ did not return a bool".to_string())
        })?
        .get_value();
    Ok(bool_value)
}

pub fn eval_not_expr(
    interpreter: Arc<Interpreter>,
    expr: LNotExpr,
) -> Result<Arc<dyn PyValue>, Arc<dyn PyValue>> {
    match expr {
        LNotExpr::Eq(expr) => eval_eq_expr(interpreter, expr),
        LNotExpr::Not(expr) => {
            let expr_value = eval_not_expr(interpreter.clone(), *expr)?;
            let bool_value = get_bool_value(interpreter.clone(), expr_value)?;
            Ok(Arc::new(PyBool::new(interpreter, !bool_value)))
        }
    }
}

pub fn eval_and_expr(
    interpreter: Arc<Interpreter>,
    expr: LAndExpr,
) -> Result<Arc<dyn PyValue>, Arc<dyn PyValue>> {
    match expr {
        LAndExpr::Not(expr) => eval_not_expr(interpreter, expr),
        LAndExpr::And(left, right) => {
            let lhs_value = eval_not_expr(interpreter.clone(), *left)?;
            let lhs_bool = get_bool_value(interpreter.clone(), lhs_value.clone())?;
            if lhs_bool {
                Ok(eval_and_expr(interpreter.clone(), *right)?)
            } else {
                Ok(lhs_value)
            }
        }
    }
}

fn eval_or_expr(
    interpreter: Arc<Interpreter>,
    expr: LOrExpr,
) -> Result<Arc<dyn PyValue>, Arc<dyn PyValue>> {
    match expr {
        LOrExpr::And(expr) => eval_and_expr(interpreter, expr),
        LOrExpr::Or(left, right) => {
            let lhs_value = eval_and_expr(interpreter.clone(), *left)?;
            let lhs_bool = get_bool_value(interpreter.clone(), lhs_value.clone())?;
            if lhs_bool {
                Ok(lhs_value)
            } else {
                Ok(eval_or_expr(interpreter.clone(), *right)?)
            }
        }
    }
}

pub fn eval_expr(
    interpreter: Arc<Interpreter>,
    expr: Expr,
) -> Result<Arc<dyn PyValue>, Arc<dyn PyValue>> {
    eval_or_expr(interpreter, expr.value)
}
