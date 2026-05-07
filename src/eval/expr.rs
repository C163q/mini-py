use std::sync::Arc;

use crate::{
    Interpreter,
    error::InterpreterError,
    lexer::ast::{Expr, Number, PrimaryExpr, UnaryExpr, UnaryOp},
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
    match expr {
        UnaryExpr::Primary(expr) => eval_primary_expr(interpreter, expr),
        UnaryExpr::Expr { op, expr } => match op {
            UnaryOp::Pos => {
                let expr_value = eval_unary_expr(interpreter.clone(), *expr)?;
                let func = expr_value.get_function("__pos__").ok_or_else(|| {
                    InterpreterError::new("Type does not support __pos__".to_string())
                })?;
                func.call(interpreter.clone(), vec![expr_value])
            }
            UnaryOp::Neg => {
                let expr_value = eval_unary_expr(interpreter.clone(), *expr)?;
                let func = expr_value.get_function("__neg__").ok_or_else(|| {
                    InterpreterError::new("Type does not support __neg__".to_string())
                })?;
                func.call(interpreter.clone(), vec![expr_value])
            }
        },
    }
}

pub fn eval_expr(
    interpreter: Arc<Interpreter>,
    expr: Expr,
) -> Result<Box<dyn PyValue>, InterpreterError> {
    eval_unary_expr(interpreter, expr.value)
}
