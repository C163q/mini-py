use std::{str::FromStr, sync::Arc};

use num_bigint::BigInt;

use crate::{
    Interpreter,
    error::InterpreterError,
    eval,
    lexer::{
        ast::{
            AddExpr, AddOp, EqExpr, EqOp, Expr, MulExpr, MulOp, Number, PrimaryExpr, RelExpr,
            RelOp, UnaryExpr, UnaryOp,
        },
        tokenize::{Operator, Separator, Token, TokenKind},
    },
    types::tstr::PyStr,
};

#[derive(Debug, Clone)]
struct EvalResult<T> {
    /// Next index to parse. It should be the index of the first token that is not parsed yet.
    pub idx: usize,
    pub value: T,
}

impl<T> EvalResult<T> {
    pub fn new(idx: usize, value: T) -> Self {
        Self { idx, value }
    }
}

fn eval_number(
    interpreter: Arc<Interpreter>,
    token: &[Token],
    idx: usize,
) -> Option<EvalResult<Number>> {
    if idx >= token.len() {
        return None;
    }

    if let TokenKind::Number(num) = &token[idx].value {
        Some(EvalResult::new(
            idx + 1,
            Number::new_int(interpreter, BigInt::from_str(num).unwrap()),
        ))
    } else {
        None
    }
}

fn eval_primary_expr(
    interpreter: Arc<Interpreter>,
    tokens: &[Token],
    idx: usize,
) -> Option<EvalResult<PrimaryExpr>> {
    if idx >= tokens.len() {
        return None;
    }

    if let Some(number) = eval_number(interpreter.clone(), tokens, idx) {
        return Some(EvalResult::new(
            number.idx,
            PrimaryExpr::new_number(number.value),
        ));
    }
    if idx + 2 < tokens.len()
        && tokens[idx].value == TokenKind::Separator(Separator::LeftParen)
        && let Some(expr) = eval_expr(interpreter, tokens, idx + 1)
        && tokens[expr.idx].value == TokenKind::Separator(Separator::RightParen)
    {
        return Some(EvalResult::new(
            expr.idx + 1,
            PrimaryExpr::new_expr(expr.value),
        ));
    }
    None
}

fn eval_unary_op(
    _interpreter: Arc<Interpreter>,
    token: &[Token],
    idx: usize,
) -> Option<EvalResult<UnaryOp>> {
    if idx >= token.len() {
        return None;
    }

    if let TokenKind::Operator(op) = &token[idx].value {
        match op {
            Operator::Add => Some(EvalResult::new(idx + 1, UnaryOp::Pos)),
            Operator::Sub => Some(EvalResult::new(idx + 1, UnaryOp::Neg)),
            _ => None,
        }
    } else {
        None
    }
}

fn eval_unary_expr(
    interpreter: Arc<Interpreter>,
    tokens: &[Token],
    idx: usize,
) -> Option<EvalResult<UnaryExpr>> {
    if idx >= tokens.len() {
        return None;
    }

    if idx + 1 < tokens.len()
        && let Some(op) = eval_unary_op(interpreter.clone(), tokens, idx)
        && let Some(expr) = eval_unary_expr(interpreter.clone(), tokens, op.idx)
    {
        return Some(EvalResult::new(
            expr.idx,
            UnaryExpr::new_unary(op.value, expr.value),
        ));
    }

    if let Some(primary) = eval_primary_expr(interpreter, tokens, idx) {
        return Some(EvalResult::new(
            primary.idx,
            UnaryExpr::new_primary(primary.value),
        ));
    }

    None
}

fn eval_mul_op(
    _interpreter: Arc<Interpreter>,
    token: &[Token],
    idx: usize,
) -> Option<EvalResult<MulOp>> {
    if idx >= token.len() {
        return None;
    }

    if let TokenKind::Operator(op) = &token[idx].value {
        match op {
            Operator::Mul => Some(EvalResult::new(idx + 1, MulOp::Mul)),
            Operator::FloorDiv => Some(EvalResult::new(idx + 1, MulOp::FloorDiv)),
            Operator::Mod => Some(EvalResult::new(idx + 1, MulOp::Mod)),
            Operator::TrueDiv => unimplemented!(),
            _ => None,
        }
    } else {
        None
    }
}

fn eval_mul_expr(
    interpreter: Arc<Interpreter>,
    tokens: &[Token],
    idx: usize,
) -> Option<EvalResult<MulExpr>> {
    if idx >= tokens.len() {
        return None;
    }

    if idx + 2 < tokens.len()
        && let Some(left) = eval_unary_expr(interpreter.clone(), tokens, idx)
        && let Some(op) = eval_mul_op(interpreter.clone(), tokens, left.idx)
        && let Some(right) = eval_mul_expr(interpreter.clone(), tokens, op.idx)
    {
        return Some(EvalResult::new(
            right.idx,
            MulExpr::new_mul(left.value, op.value, right.value),
        ));
    }

    if let Some(unary) = eval_unary_expr(interpreter.clone(), tokens, idx) {
        return Some(EvalResult::new(unary.idx, MulExpr::new_unary(unary.value)));
    }

    None
}

fn eval_add_op(
    _interpreter: Arc<Interpreter>,
    token: &[Token],
    idx: usize,
) -> Option<EvalResult<AddOp>> {
    if idx >= token.len() {
        return None;
    }

    if let TokenKind::Operator(op) = &token[idx].value {
        match op {
            Operator::Add => Some(EvalResult::new(idx + 1, AddOp::Add)),
            Operator::Sub => Some(EvalResult::new(idx + 1, AddOp::Sub)),
            _ => None,
        }
    } else {
        None
    }
}

fn eval_add_expr(
    interpreter: Arc<Interpreter>,
    tokens: &[Token],
    idx: usize,
) -> Option<EvalResult<AddExpr>> {
    if idx >= tokens.len() {
        return None;
    }

    if idx + 2 < tokens.len()
        && let Some(left) = eval_mul_expr(interpreter.clone(), tokens, idx)
        && let Some(op) = eval_add_op(interpreter.clone(), tokens, left.idx)
        && let Some(right) = eval_add_expr(interpreter.clone(), tokens, op.idx)
    {
        return Some(EvalResult::new(
            right.idx,
            AddExpr::new_add(left.value, op.value, right.value),
        ));
    }

    if let Some(mul) = eval_mul_expr(interpreter.clone(), tokens, idx) {
        return Some(EvalResult::new(mul.idx, AddExpr::new_mul(mul.value)));
    }

    None
}

fn eval_rel_op(
    _interpreter: Arc<Interpreter>,
    tokens: &[Token],
    idx: usize,
) -> Option<EvalResult<RelOp>> {
    if idx >= tokens.len() {
        return None;
    }

    if let TokenKind::Operator(op) = &tokens[idx].value {
        match op {
            Operator::Less => Some(EvalResult::new(idx + 1, RelOp::Lt)),
            Operator::Greater => Some(EvalResult::new(idx + 1, RelOp::Gt)),
            Operator::LessEqual => Some(EvalResult::new(idx + 1, RelOp::Le)),
            Operator::GreaterEqual => Some(EvalResult::new(idx + 1, RelOp::Ge)),
            _ => None,
        }
    } else {
        None
    }
}

fn eval_rel_expr(
    interpreter: Arc<Interpreter>,
    tokens: &[Token],
    idx: usize,
) -> Option<EvalResult<RelExpr>> {
    if idx >= tokens.len() {
        return None;
    }

    if idx + 2 < tokens.len()
        && let Some(left) = eval_add_expr(interpreter.clone(), tokens, idx)
        && let Some(op) = eval_rel_op(interpreter.clone(), tokens, left.idx)
        && let Some(right) = eval_rel_expr(interpreter.clone(), tokens, op.idx)
    {
        return Some(EvalResult::new(
            right.idx,
            RelExpr::new_rel(left.value, op.value, right.value),
        ));
    }

    if let Some(add) = eval_add_expr(interpreter.clone(), tokens, idx) {
        return Some(EvalResult::new(add.idx, RelExpr::new_add(add.value)));
    }

    None
}

fn eval_eq_op(
    _interpreter: Arc<Interpreter>,
    tokens: &[Token],
    idx: usize,
) -> Option<EvalResult<EqOp>> {
    if idx >= tokens.len() {
        return None;
    }

    if let TokenKind::Operator(op) = &tokens[idx].value {
        match op {
            Operator::Equal => Some(EvalResult::new(idx + 1, EqOp::Eq)),
            Operator::NotEqual => Some(EvalResult::new(idx + 1, EqOp::NotEq)),
            _ => None,
        }
    } else {
        None
    }
}

fn eval_eq_expr(
    interpreter: Arc<Interpreter>,
    tokens: &[Token],
    idx: usize,
) -> Option<EvalResult<EqExpr>> {
    if idx >= tokens.len() {
        return None;
    }

    if idx + 2 < tokens.len()
        && let Some(left) = eval_rel_expr(interpreter.clone(), tokens, idx)
        && let Some(op) = eval_eq_op(interpreter.clone(), tokens, left.idx)
        && let Some(right) = eval_eq_expr(interpreter.clone(), tokens, op.idx)
    {
        return Some(EvalResult::new(
            right.idx,
            EqExpr::new_eq(left.value, op.value, right.value),
        ));
    }

    if let Some(rel) = eval_rel_expr(interpreter.clone(), tokens, idx) {
        return Some(EvalResult::new(rel.idx, EqExpr::new_rel(rel.value)));
    }

    None
}

fn eval_expr(
    interpreter: Arc<Interpreter>,
    tokens: &[Token],
    idx: usize,
) -> Option<EvalResult<Expr>> {
    if idx >= tokens.len() {
        return None;
    }

    if let Some(eq) = eval_eq_expr(interpreter, tokens, idx) {
        return Some(EvalResult::new(eq.idx, Expr::new_eq(eq.value)));
    }

    None
}

/// Ok(PyStr) if the line is valid, Err(InterpreterError) otherwise.
///
/// Note that PyStr is only used for REPL. Any effect of the line should be applied to the
/// interpreter.
pub fn eval_line(
    interpreter: Arc<Interpreter>,
    tokens: &[Token],
) -> Result<Option<PyStr>, InterpreterError> {
    let idx = 0;

    // <expr>
    if let Some(expr) = eval_expr(interpreter.clone(), tokens, idx)
        && expr.idx == tokens.len()
    {
        let value = eval::expr::eval_expr(interpreter.clone(), expr.value)?;
        let output = eval::output_value(interpreter.clone(), value)?;
        return Ok(Some(output));
    }

    Err(InterpreterError::new("Invalid syntax".to_string()))
}
