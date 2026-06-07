use std::{str::FromStr, sync::Arc};

use num_bigint::BigInt;

use crate::{
    Interpreter,
    eval::ParseResult,
    lexer::{
        ast::{
            AddExpr, AddOp, EqExpr, EqOp, Expr, LAndExpr, LNotExpr, LOrExpr, MulExpr, MulOp,
            Number, PrimaryExpr, RelExpr, RelOp, UnaryExpr, UnaryOp,
        },
        tokenize::{Keyword, Operator, Separator, Token, TokenKind},
    },
};

fn parse_number(
    interpreter: Arc<Interpreter>,
    token: &[Token],
    idx: usize,
) -> Option<ParseResult<Number>> {
    if idx >= token.len() {
        return None;
    }

    if let TokenKind::Number(num) = &token[idx].value {
        Some(ParseResult::new(
            idx + 1,
            Number::new_int(interpreter, BigInt::from_str(num).unwrap()),
        ))
    } else {
        None
    }
}

fn parse_none(
    _interpreter: Arc<Interpreter>,
    token: &[Token],
    idx: usize,
) -> Option<ParseResult<()>> {
    if idx >= token.len() {
        return None;
    }

    if token[idx].value == TokenKind::Keyword(Keyword::None) {
        Some(ParseResult::new(idx + 1, ()))
    } else {
        None
    }
}

fn parse_string(
    _interpreter: Arc<Interpreter>,
    token: &[Token],
    idx: usize,
) -> Option<ParseResult<String>> {
    if idx >= token.len() {
        return None;
    }

    if let TokenKind::String(s) = &token[idx].value {
        Some(ParseResult::new(idx + 1, s.clone()))
    } else {
        None
    }
}

fn parse_primary_expr(
    interpreter: Arc<Interpreter>,
    tokens: &[Token],
    idx: usize,
) -> Option<ParseResult<PrimaryExpr>> {
    if idx >= tokens.len() {
        return None;
    }

    if parse_none(interpreter.clone(), tokens, idx).is_some() {
        return Some(ParseResult::new(idx + 1, PrimaryExpr::new_none()));
    }

    if let Some(string) = parse_string(interpreter.clone(), tokens, idx) {
        return Some(ParseResult::new(
            string.idx,
            PrimaryExpr::new_str(string.value),
        ));
    }

    if let Some(number) = parse_number(interpreter.clone(), tokens, idx) {
        return Some(ParseResult::new(
            number.idx,
            PrimaryExpr::new_number(number.value),
        ));
    }
    if idx + 2 < tokens.len()
        && tokens[idx].value == TokenKind::Separator(Separator::LeftParen)
        && let Some(expr) = parse_expr(interpreter, tokens, idx + 1)
        && tokens[expr.idx].value == TokenKind::Separator(Separator::RightParen)
    {
        return Some(ParseResult::new(
            expr.idx + 1,
            PrimaryExpr::new_expr(expr.value),
        ));
    }
    None
}

fn parse_unary_op(
    _interpreter: Arc<Interpreter>,
    token: &[Token],
    idx: usize,
) -> Option<ParseResult<UnaryOp>> {
    if idx >= token.len() {
        return None;
    }

    if let TokenKind::Operator(op) = &token[idx].value {
        match op {
            Operator::Add => Some(ParseResult::new(idx + 1, UnaryOp::Pos)),
            Operator::Sub => Some(ParseResult::new(idx + 1, UnaryOp::Neg)),
            Operator::Not => Some(ParseResult::new(idx + 1, UnaryOp::BitNot)),
            _ => None,
        }
    } else {
        None
    }
}

fn parse_unary_expr(
    interpreter: Arc<Interpreter>,
    tokens: &[Token],
    idx: usize,
) -> Option<ParseResult<UnaryExpr>> {
    if idx >= tokens.len() {
        return None;
    }

    if idx + 1 < tokens.len()
        && let Some(op) = parse_unary_op(interpreter.clone(), tokens, idx)
        && let Some(expr) = parse_unary_expr(interpreter.clone(), tokens, op.idx)
    {
        return Some(ParseResult::new(
            expr.idx,
            UnaryExpr::new_unary(op.value, expr.value),
        ));
    }

    if let Some(primary) = parse_primary_expr(interpreter, tokens, idx) {
        return Some(ParseResult::new(
            primary.idx,
            UnaryExpr::new_primary(primary.value),
        ));
    }

    None
}

fn parse_mul_op(
    _interpreter: Arc<Interpreter>,
    token: &[Token],
    idx: usize,
) -> Option<ParseResult<MulOp>> {
    if idx >= token.len() {
        return None;
    }

    if let TokenKind::Operator(op) = &token[idx].value {
        match op {
            Operator::Mul => Some(ParseResult::new(idx + 1, MulOp::Mul)),
            Operator::FloorDiv => Some(ParseResult::new(idx + 1, MulOp::FloorDiv)),
            Operator::Mod => Some(ParseResult::new(idx + 1, MulOp::Mod)),
            Operator::TrueDiv => Some(ParseResult::new(idx + 1, MulOp::TrueDiv)),
            _ => None,
        }
    } else {
        None
    }
}

fn parse_mul_expr(
    interpreter: Arc<Interpreter>,
    tokens: &[Token],
    idx: usize,
) -> Option<ParseResult<MulExpr>> {
    if idx >= tokens.len() {
        return None;
    }

    if idx + 2 < tokens.len()
        && let Some(left) = parse_unary_expr(interpreter.clone(), tokens, idx)
        && let Some(op) = parse_mul_op(interpreter.clone(), tokens, left.idx)
        && let Some(right) = parse_mul_expr(interpreter.clone(), tokens, op.idx)
    {
        return Some(ParseResult::new(
            right.idx,
            MulExpr::new_mul(left.value, op.value, right.value),
        ));
    }

    if let Some(unary) = parse_unary_expr(interpreter.clone(), tokens, idx) {
        return Some(ParseResult::new(unary.idx, MulExpr::new_unary(unary.value)));
    }

    None
}

fn parse_add_op(
    _interpreter: Arc<Interpreter>,
    token: &[Token],
    idx: usize,
) -> Option<ParseResult<AddOp>> {
    if idx >= token.len() {
        return None;
    }

    if let TokenKind::Operator(op) = &token[idx].value {
        match op {
            Operator::Add => Some(ParseResult::new(idx + 1, AddOp::Add)),
            Operator::Sub => Some(ParseResult::new(idx + 1, AddOp::Sub)),
            _ => None,
        }
    } else {
        None
    }
}

fn parse_add_expr(
    interpreter: Arc<Interpreter>,
    tokens: &[Token],
    idx: usize,
) -> Option<ParseResult<AddExpr>> {
    if idx >= tokens.len() {
        return None;
    }

    if idx + 2 < tokens.len()
        && let Some(left) = parse_mul_expr(interpreter.clone(), tokens, idx)
        && let Some(op) = parse_add_op(interpreter.clone(), tokens, left.idx)
        && let Some(right) = parse_add_expr(interpreter.clone(), tokens, op.idx)
    {
        return Some(ParseResult::new(
            right.idx,
            AddExpr::new_add(left.value, op.value, right.value),
        ));
    }

    if let Some(mul) = parse_mul_expr(interpreter.clone(), tokens, idx) {
        return Some(ParseResult::new(mul.idx, AddExpr::new_mul(mul.value)));
    }

    None
}

fn parse_rel_op(
    _interpreter: Arc<Interpreter>,
    tokens: &[Token],
    idx: usize,
) -> Option<ParseResult<RelOp>> {
    if idx >= tokens.len() {
        return None;
    }

    if let TokenKind::Operator(op) = &tokens[idx].value {
        match op {
            Operator::Less => Some(ParseResult::new(idx + 1, RelOp::Lt)),
            Operator::Greater => Some(ParseResult::new(idx + 1, RelOp::Gt)),
            Operator::LessEqual => Some(ParseResult::new(idx + 1, RelOp::Le)),
            Operator::GreaterEqual => Some(ParseResult::new(idx + 1, RelOp::Ge)),
            _ => None,
        }
    } else {
        None
    }
}

fn parse_rel_expr(
    interpreter: Arc<Interpreter>,
    tokens: &[Token],
    idx: usize,
) -> Option<ParseResult<RelExpr>> {
    if idx >= tokens.len() {
        return None;
    }

    if idx + 2 < tokens.len()
        && let Some(left) = parse_add_expr(interpreter.clone(), tokens, idx)
        && let Some(op) = parse_rel_op(interpreter.clone(), tokens, left.idx)
        && let Some(right) = parse_rel_expr(interpreter.clone(), tokens, op.idx)
    {
        return Some(ParseResult::new(
            right.idx,
            RelExpr::new_rel(left.value, op.value, right.value),
        ));
    }

    if let Some(add) = parse_add_expr(interpreter.clone(), tokens, idx) {
        return Some(ParseResult::new(add.idx, RelExpr::new_add(add.value)));
    }

    None
}

fn parse_eq_op(
    _interpreter: Arc<Interpreter>,
    tokens: &[Token],
    idx: usize,
) -> Option<ParseResult<EqOp>> {
    if idx >= tokens.len() {
        return None;
    }

    if let TokenKind::Operator(op) = &tokens[idx].value {
        match op {
            Operator::Equal => Some(ParseResult::new(idx + 1, EqOp::Eq)),
            Operator::NotEqual => Some(ParseResult::new(idx + 1, EqOp::NotEq)),
            _ => None,
        }
    } else {
        None
    }
}

fn parse_eq_expr(
    interpreter: Arc<Interpreter>,
    tokens: &[Token],
    idx: usize,
) -> Option<ParseResult<EqExpr>> {
    if idx >= tokens.len() {
        return None;
    }

    if idx + 2 < tokens.len()
        && let Some(left) = parse_rel_expr(interpreter.clone(), tokens, idx)
        && let Some(op) = parse_eq_op(interpreter.clone(), tokens, left.idx)
        && let Some(right) = parse_eq_expr(interpreter.clone(), tokens, op.idx)
    {
        return Some(ParseResult::new(
            right.idx,
            EqExpr::new_eq(left.value, op.value, right.value),
        ));
    }

    if let Some(rel) = parse_rel_expr(interpreter.clone(), tokens, idx) {
        return Some(ParseResult::new(rel.idx, EqExpr::new_rel(rel.value)));
    }

    None
}

fn parse_not_expr(
    interpreter: Arc<Interpreter>,
    tokens: &[Token],
    idx: usize,
) -> Option<ParseResult<LNotExpr>> {
    if idx >= tokens.len() {
        return None;
    }

    if idx + 1 < tokens.len()
        && tokens[idx].value == TokenKind::Keyword(Keyword::Not)
        && let Some(expr) = parse_not_expr(interpreter.clone(), tokens, idx + 1)
    {
        return Some(ParseResult::new(expr.idx, LNotExpr::new_not(expr.value)));
    }

    if let Some(eq) = parse_eq_expr(interpreter, tokens, idx) {
        return Some(ParseResult::new(eq.idx, LNotExpr::new_eq(eq.value)));
    }

    None
}

fn parse_and_expr(
    interpreter: Arc<Interpreter>,
    tokens: &[Token],
    idx: usize,
) -> Option<ParseResult<LAndExpr>> {
    if idx >= tokens.len() {
        return None;
    }

    if let Some(left) = parse_not_expr(interpreter.clone(), tokens, idx)
        && left.idx + 1 < tokens.len()
        && tokens[left.idx].value == TokenKind::Keyword(Keyword::And)
        && let Some(right) = parse_and_expr(interpreter.clone(), tokens, left.idx + 1)
    {
        return Some(ParseResult::new(
            right.idx,
            LAndExpr::new_and(left.value, right.value),
        ));
    }

    if let Some(not) = parse_not_expr(interpreter.clone(), tokens, idx) {
        return Some(ParseResult::new(not.idx, LAndExpr::new_not(not.value)));
    }

    None
}

fn parse_or_expr(
    interpreter: Arc<Interpreter>,
    tokens: &[Token],
    idx: usize,
) -> Option<ParseResult<LOrExpr>> {
    if idx >= tokens.len() {
        return None;
    }

    if let Some(left) = parse_and_expr(interpreter.clone(), tokens, idx)
        && left.idx + 1 < tokens.len()
        && tokens[left.idx].value == TokenKind::Keyword(Keyword::Or)
        && let Some(right) = parse_or_expr(interpreter.clone(), tokens, left.idx + 1)
    {
        return Some(ParseResult::new(
            right.idx,
            LOrExpr::new_or(left.value, right.value),
        ));
    }

    if let Some(and) = parse_and_expr(interpreter.clone(), tokens, idx) {
        return Some(ParseResult::new(and.idx, LOrExpr::new_and(and.value)));
    }

    None
}

pub fn parse_expr(
    interpreter: Arc<Interpreter>,
    tokens: &[Token],
    idx: usize,
) -> Option<ParseResult<Expr>> {
    if idx >= tokens.len() {
        return None;
    }

    if let Some(lor) = parse_or_expr(interpreter, tokens, idx) {
        return Some(ParseResult::new(lor.idx, Expr::new_or(lor.value)));
    }

    None
}
