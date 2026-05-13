use std::sync::Arc;

use crate::{
    Interpreter,
    error::InterpreterError,
    lexer::ast::{
        AddExpr, AddOp, EqExpr, EqOp, Expr, LAndExpr, LNotExpr, LOrExpr, MulExpr, MulOp, Number,
        PrimaryExpr, RelExpr, RelOp, UnaryExpr, UnaryOp,
    },
    types::tbool::PyBool,
    var::PyValue,
};

pub fn eval_number(
    _interpreter: Arc<Interpreter>,
    num: Number,
) -> Result<Box<dyn PyValue>, InterpreterError> {
    match num {
        Number::Int(num) => Ok(Box::new(num)),
    }
}

pub fn eval_primary_expr(
    interpreter: Arc<Interpreter>,
    expr: PrimaryExpr,
) -> Result<Box<dyn PyValue>, InterpreterError> {
    match expr {
        PrimaryExpr::Number(num) => eval_number(interpreter, num),
        PrimaryExpr::Expr(expr) => eval_expr(interpreter, *expr),
    }
}

pub fn eval_unary_expr(
    interpreter: Arc<Interpreter>,
    expr: UnaryExpr,
) -> Result<Box<dyn PyValue>, InterpreterError> {
    macro_rules! eval_unary {
        ($interpreter:ident, $expr:expr, $func:literal) => {{
            let expr_value = eval_unary_expr($interpreter.clone(), $expr)?;
            let func = expr_value
                .get_function($func)
                .ok_or_else(|| InterpreterError::new(format!("Type does not support {}", $func)))?;
            func.call($interpreter.clone(), vec![expr_value])
        }};
    }
    match expr {
        UnaryExpr::Primary(expr) => eval_primary_expr(interpreter, expr),
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

macro_rules! eval_binary {
    ($interpreter:ident, $lhs:expr, $eval_lhs:ident, $rhs:expr, $eval_rhs:ident, $func:literal) => {{
        let lhs = $eval_lhs($interpreter.clone(), $lhs)?;
        let rhs = $eval_rhs($interpreter.clone(), $rhs)?;
        let func = lhs
            .get_function($func)
            .ok_or_else(|| InterpreterError::new(format!("Type does not support {}", $func)))?;
        func.call($interpreter.clone(), vec![lhs, rhs])
    }};
}

pub fn eval_mul_expr(
    interpreter: Arc<Interpreter>,
    expr: crate::lexer::ast::MulExpr,
) -> Result<Box<dyn PyValue>, InterpreterError> {
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
        },
    }
}

pub fn eval_add_expr(
    interpreter: Arc<Interpreter>,
    expr: AddExpr,
) -> Result<Box<dyn PyValue>, InterpreterError> {
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
) -> Result<Box<dyn PyValue>, InterpreterError> {
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
) -> Result<Box<dyn PyValue>, InterpreterError> {
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
    value: Box<dyn PyValue>,
) -> Result<bool, InterpreterError> {
    let bool_func = value
        .get_function("__bool__")
        .ok_or_else(|| InterpreterError::new("Type does not support __bool__".to_string()))?;
    let bool_value = bool_func.call(interpreter.clone(), vec![value])?;
    let bool_value = bool_value
        .as_any()
        .downcast_ref::<PyBool>()
        .ok_or_else(|| InterpreterError::new("__bool__ did not return a bool".to_string()))?
        .get_value();
    Ok(bool_value)
}

pub fn eval_not_expr(
    interpreter: Arc<Interpreter>,
    expr: LNotExpr,
) -> Result<Box<dyn PyValue>, InterpreterError> {
    match expr {
        LNotExpr::Eq(expr) => eval_eq_expr(interpreter, expr),
        LNotExpr::Not(expr) => {
            let expr_value = eval_not_expr(interpreter.clone(), *expr)?;
            let bool_value = get_bool_value(interpreter.clone(), expr_value)?;
            Ok(Box::new(PyBool::new(interpreter, !bool_value)))
        }
    }
}

pub fn eval_and_expr(
    interpreter: Arc<Interpreter>,
    expr: LAndExpr,
) -> Result<Box<dyn PyValue>, InterpreterError> {
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

pub fn eval_or_expr(
    interpreter: Arc<Interpreter>,
    expr: LOrExpr,
) -> Result<Box<dyn PyValue>, InterpreterError> {
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
) -> Result<Box<dyn PyValue>, InterpreterError> {
    eval_or_expr(interpreter, expr.value)
}
