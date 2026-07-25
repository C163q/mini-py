//! Expression parsing — one `parse_*` function per AST node in the precedence hierarchy.
//! Each function tries to match the current token position and returns `None` on mismatch,
//! letting the caller fall through to the next lower-precedence rule.

use std::{str::FromStr, sync::Arc};

use num_bigint::BigInt;

use crate::{
    Interpreter,
    eval::{
        Parse, ParseResult,
        basic::ast::LValue,
        expr::ast::{
            AddExpr, AddOp, EqExpr, EqOp, Expr, LAndExpr, LNotExpr, LOrExpr, MulExpr, MulOp,
            NoneExpr, Number, PowExpr, PrimaryExpr, RelExpr, RelOp, UnaryExpr, UnaryOp,
        },
    },
    lexer::tokenize::{Keyword, Operator, Separator, Token, TokenNode},
};

impl Parse for Number {
    fn parse(
        interpreter: Arc<Interpreter>,
        tokens: &[TokenNode],
        idx: usize,
    ) -> Option<ParseResult<Self>> {
        parse_number(interpreter, tokens, idx)
    }
}

impl Parse for NoneExpr {
    fn parse(
        interpreter: Arc<Interpreter>,
        tokens: &[TokenNode],
        idx: usize,
    ) -> Option<ParseResult<Self>> {
        parse_none(interpreter, tokens, idx)
    }
}

impl Parse for String {
    fn parse(
        interpreter: Arc<Interpreter>,
        tokens: &[TokenNode],
        idx: usize,
    ) -> Option<ParseResult<Self>> {
        parse_string(interpreter, tokens, idx)
    }
}

impl Parse for PowExpr {
    fn parse(
        interpreter: Arc<Interpreter>,
        tokens: &[TokenNode],
        idx: usize,
    ) -> Option<ParseResult<Self>> {
        parse_pow_expr(interpreter, tokens, idx)
    }
}

impl Parse for PrimaryExpr {
    fn parse(
        interpreter: Arc<Interpreter>,
        tokens: &[TokenNode],
        idx: usize,
    ) -> Option<ParseResult<Self>> {
        parse_primary_expr(interpreter, tokens, idx)
    }
}

impl Parse for UnaryOp {
    fn parse(
        interpreter: Arc<Interpreter>,
        tokens: &[TokenNode],
        idx: usize,
    ) -> Option<ParseResult<Self>> {
        parse_unary_op(interpreter, tokens, idx)
    }
}

impl Parse for UnaryExpr {
    fn parse(
        interpreter: Arc<Interpreter>,
        tokens: &[TokenNode],
        idx: usize,
    ) -> Option<ParseResult<Self>> {
        parse_unary_expr(interpreter, tokens, idx)
    }
}

impl Parse for MulOp {
    fn parse(
        interpreter: Arc<Interpreter>,
        tokens: &[TokenNode],
        idx: usize,
    ) -> Option<ParseResult<Self>> {
        parse_mul_op(interpreter, tokens, idx)
    }
}

impl Parse for MulExpr {
    fn parse(
        interpreter: Arc<Interpreter>,
        tokens: &[TokenNode],
        idx: usize,
    ) -> Option<ParseResult<Self>> {
        parse_mul_expr(interpreter, tokens, idx)
    }
}

impl Parse for AddOp {
    fn parse(
        interpreter: Arc<Interpreter>,
        tokens: &[TokenNode],
        idx: usize,
    ) -> Option<ParseResult<Self>> {
        parse_add_op(interpreter, tokens, idx)
    }
}

impl Parse for AddExpr {
    fn parse(
        interpreter: Arc<Interpreter>,
        tokens: &[TokenNode],
        idx: usize,
    ) -> Option<ParseResult<Self>> {
        parse_add_expr(interpreter, tokens, idx)
    }
}

impl Parse for RelOp {
    fn parse(
        interpreter: Arc<Interpreter>,
        tokens: &[TokenNode],
        idx: usize,
    ) -> Option<ParseResult<Self>> {
        parse_rel_op(interpreter, tokens, idx)
    }
}

impl Parse for RelExpr {
    fn parse(
        interpreter: Arc<Interpreter>,
        tokens: &[TokenNode],
        idx: usize,
    ) -> Option<ParseResult<Self>> {
        parse_rel_expr(interpreter, tokens, idx)
    }
}

impl Parse for EqOp {
    fn parse(
        interpreter: Arc<Interpreter>,
        tokens: &[TokenNode],
        idx: usize,
    ) -> Option<ParseResult<Self>> {
        parse_eq_op(interpreter, tokens, idx)
    }
}

impl Parse for EqExpr {
    fn parse(
        interpreter: Arc<Interpreter>,
        tokens: &[TokenNode],
        idx: usize,
    ) -> Option<ParseResult<Self>> {
        parse_eq_expr(interpreter, tokens, idx)
    }
}

impl Parse for LNotExpr {
    fn parse(
        interpreter: Arc<Interpreter>,
        tokens: &[TokenNode],
        idx: usize,
    ) -> Option<ParseResult<Self>> {
        parse_not_expr(interpreter, tokens, idx)
    }
}

impl Parse for LAndExpr {
    fn parse(
        interpreter: Arc<Interpreter>,
        tokens: &[TokenNode],
        idx: usize,
    ) -> Option<ParseResult<Self>> {
        parse_and_expr(interpreter, tokens, idx)
    }
}

impl Parse for LOrExpr {
    fn parse(
        interpreter: Arc<Interpreter>,
        tokens: &[TokenNode],
        idx: usize,
    ) -> Option<ParseResult<Self>> {
        parse_or_expr(interpreter, tokens, idx)
    }
}

impl Parse for Expr {
    fn parse(
        interpreter: Arc<Interpreter>,
        tokens: &[TokenNode],
        idx: usize,
    ) -> Option<ParseResult<Self>> {
        parse_expr(interpreter, tokens, idx)
    }
}

fn parse_number(
    interpreter: Arc<Interpreter>,
    token: &[TokenNode],
    idx: usize,
) -> Option<ParseResult<Number>> {
    if idx >= token.len() {
        return None;
    }

    if let Token::Number(num) = &token[idx].value {
        if num.contains('.') {
            Some(ParseResult::new(
                idx + 1,
                Number::new_float(interpreter, num.parse::<f64>().unwrap()),
            ))
        } else {
            Some(ParseResult::new(
                idx + 1,
                Number::new_int(interpreter, BigInt::from_str(num).unwrap()),
            ))
        }
    } else {
        None
    }
}

fn parse_none(
    _interpreter: Arc<Interpreter>,
    token: &[TokenNode],
    idx: usize,
) -> Option<ParseResult<NoneExpr>> {
    if idx >= token.len() {
        return None;
    }

    if token[idx].value == Token::Keyword(Keyword::None) {
        Some(ParseResult::new(idx + 1, NoneExpr))
    } else {
        None
    }
}

fn parse_string(
    _interpreter: Arc<Interpreter>,
    token: &[TokenNode],
    idx: usize,
) -> Option<ParseResult<String>> {
    if idx >= token.len() {
        return None;
    }

    if let Token::String(s) = &token[idx].value {
        Some(ParseResult::new(idx + 1, s.clone()))
    } else {
        None
    }
}

fn parse_primary_expr(
    interpreter: Arc<Interpreter>,
    tokens: &[TokenNode],
    idx: usize,
) -> Option<ParseResult<PrimaryExpr>> {
    if idx >= tokens.len() {
        return None;
    }

    if NoneExpr::parse(interpreter.clone(), tokens, idx).is_some() {
        return Some(ParseResult::new(idx + 1, PrimaryExpr::new_none()));
    }

    if let Some(string) = String::parse(interpreter.clone(), tokens, idx) {
        return Some(ParseResult::new(
            string.idx,
            PrimaryExpr::new_str(string.value),
        ));
    }

    if let Some(number) = Number::parse(interpreter.clone(), tokens, idx) {
        return Some(ParseResult::new(
            number.idx,
            PrimaryExpr::new_number(number.value),
        ));
    }

    if let Some(lvalue) = LValue::parse(interpreter.clone(), tokens, idx) {
        return Some(ParseResult::new(
            lvalue.idx,
            PrimaryExpr::new_lvalue(lvalue.value),
        ));
    }

    if idx + 2 < tokens.len()
        && tokens[idx].value == Token::Separator(Separator::LeftParen)
        && let Some(expr) = Expr::parse(interpreter, tokens, idx + 1)
        && tokens[expr.idx].value == Token::Separator(Separator::RightParen)
    {
        return Some(ParseResult::new(
            expr.idx + 1,
            PrimaryExpr::new_expr(expr.value),
        ));
    }
    None
}

fn parse_pow_expr(
    interpreter: Arc<Interpreter>,
    tokens: &[TokenNode],
    idx: usize,
) -> Option<ParseResult<PowExpr>> {
    if idx >= tokens.len() {
        return None;
    }

    if let Some(primary) = PrimaryExpr::parse(interpreter.clone(), tokens, idx) {
        if primary.idx + 1 < tokens.len()
            && tokens[primary.idx].value == Token::Operator(Operator::Pow)
            && let Some(right) = PowExpr::parse(interpreter.clone(), tokens, primary.idx + 1)
        {
            return Some(ParseResult::new(
                right.idx,
                PowExpr::new_pow(primary.value, right.value),
            ));
        }

        return Some(ParseResult::new(
            primary.idx,
            PowExpr::new_primary(primary.value),
        ));
    }

    None
}

fn parse_unary_op(
    _interpreter: Arc<Interpreter>,
    token: &[TokenNode],
    idx: usize,
) -> Option<ParseResult<UnaryOp>> {
    if idx >= token.len() {
        return None;
    }

    if let Token::Operator(op) = &token[idx].value {
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
    tokens: &[TokenNode],
    idx: usize,
) -> Option<ParseResult<UnaryExpr>> {
    if idx >= tokens.len() {
        return None;
    }

    if idx + 1 < tokens.len()
        && let Some(op) = UnaryOp::parse(interpreter.clone(), tokens, idx)
        && let Some(expr) = UnaryExpr::parse(interpreter.clone(), tokens, op.idx)
    {
        return Some(ParseResult::new(
            expr.idx,
            UnaryExpr::new_unary(op.value, expr.value),
        ));
    }

    if let Some(pow) = PowExpr::parse(interpreter, tokens, idx) {
        return Some(ParseResult::new(pow.idx, UnaryExpr::new_pow(pow.value)));
    }

    None
}

fn parse_mul_op(
    _interpreter: Arc<Interpreter>,
    token: &[TokenNode],
    idx: usize,
) -> Option<ParseResult<MulOp>> {
    if idx >= token.len() {
        return None;
    }

    if let Token::Operator(op) = &token[idx].value {
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
    tokens: &[TokenNode],
    idx: usize,
) -> Option<ParseResult<MulExpr>> {
    if idx >= tokens.len() {
        return None;
    }

    if idx + 2 < tokens.len()
        && let Some(left) = UnaryExpr::parse(interpreter.clone(), tokens, idx)
        && let Some(op) = MulOp::parse(interpreter.clone(), tokens, left.idx)
        && let Some(right) = MulExpr::parse(interpreter.clone(), tokens, op.idx)
    {
        return Some(ParseResult::new(
            right.idx,
            MulExpr::new_mul(left.value, op.value, right.value),
        ));
    }

    if let Some(unary) = UnaryExpr::parse(interpreter.clone(), tokens, idx) {
        return Some(ParseResult::new(unary.idx, MulExpr::new_unary(unary.value)));
    }

    None
}

fn parse_add_op(
    _interpreter: Arc<Interpreter>,
    tokens: &[TokenNode],
    idx: usize,
) -> Option<ParseResult<AddOp>> {
    if idx >= tokens.len() {
        return None;
    }

    if let Token::Operator(op) = &tokens[idx].value {
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
    tokens: &[TokenNode],
    idx: usize,
) -> Option<ParseResult<AddExpr>> {
    if idx >= tokens.len() {
        return None;
    }

    if idx + 2 < tokens.len()
        && let Some(left) = MulExpr::parse(interpreter.clone(), tokens, idx)
        && let Some(op) = AddOp::parse(interpreter.clone(), tokens, left.idx)
        && let Some(right) = AddExpr::parse(interpreter.clone(), tokens, op.idx)
    {
        return Some(ParseResult::new(
            right.idx,
            AddExpr::new_add(left.value, op.value, right.value),
        ));
    }

    if let Some(mul) = MulExpr::parse(interpreter.clone(), tokens, idx) {
        return Some(ParseResult::new(mul.idx, AddExpr::new_mul(mul.value)));
    }

    None
}

fn parse_rel_op(
    _interpreter: Arc<Interpreter>,
    tokens: &[TokenNode],
    idx: usize,
) -> Option<ParseResult<RelOp>> {
    if idx >= tokens.len() {
        return None;
    }

    if let Token::Operator(op) = &tokens[idx].value {
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
    tokens: &[TokenNode],
    idx: usize,
) -> Option<ParseResult<RelExpr>> {
    if idx >= tokens.len() {
        return None;
    }

    if idx + 2 < tokens.len()
        && let Some(left) = AddExpr::parse(interpreter.clone(), tokens, idx)
        && let Some(op) = RelOp::parse(interpreter.clone(), tokens, left.idx)
        && let Some(right) = RelExpr::parse(interpreter.clone(), tokens, op.idx)
    {
        return Some(ParseResult::new(
            right.idx,
            RelExpr::new_rel(left.value, op.value, right.value),
        ));
    }

    if let Some(add) = AddExpr::parse(interpreter.clone(), tokens, idx) {
        return Some(ParseResult::new(add.idx, RelExpr::new_add(add.value)));
    }

    None
}

fn parse_eq_op(
    _interpreter: Arc<Interpreter>,
    tokens: &[TokenNode],
    idx: usize,
) -> Option<ParseResult<EqOp>> {
    if idx >= tokens.len() {
        return None;
    }

    if let Token::Operator(op) = &tokens[idx].value {
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
    tokens: &[TokenNode],
    idx: usize,
) -> Option<ParseResult<EqExpr>> {
    if idx >= tokens.len() {
        return None;
    }

    if idx + 2 < tokens.len()
        && let Some(left) = RelExpr::parse(interpreter.clone(), tokens, idx)
        && let Some(op) = EqOp::parse(interpreter.clone(), tokens, left.idx)
        && let Some(right) = EqExpr::parse(interpreter.clone(), tokens, op.idx)
    {
        return Some(ParseResult::new(
            right.idx,
            EqExpr::new_eq(left.value, op.value, right.value),
        ));
    }

    if let Some(rel) = RelExpr::parse(interpreter.clone(), tokens, idx) {
        return Some(ParseResult::new(rel.idx, EqExpr::new_rel(rel.value)));
    }

    None
}

fn parse_not_expr(
    interpreter: Arc<Interpreter>,
    tokens: &[TokenNode],
    idx: usize,
) -> Option<ParseResult<LNotExpr>> {
    if idx >= tokens.len() {
        return None;
    }

    if idx + 1 < tokens.len()
        && tokens[idx].value == Token::Keyword(Keyword::Not)
        && let Some(expr) = LNotExpr::parse(interpreter.clone(), tokens, idx + 1)
    {
        return Some(ParseResult::new(expr.idx, LNotExpr::new_not(expr.value)));
    }

    if let Some(eq) = EqExpr::parse(interpreter, tokens, idx) {
        return Some(ParseResult::new(eq.idx, LNotExpr::new_eq(eq.value)));
    }

    None
}

fn parse_and_expr(
    interpreter: Arc<Interpreter>,
    tokens: &[TokenNode],
    idx: usize,
) -> Option<ParseResult<LAndExpr>> {
    if idx >= tokens.len() {
        return None;
    }

    if let Some(left) = LNotExpr::parse(interpreter.clone(), tokens, idx)
        && left.idx + 1 < tokens.len()
        && tokens[left.idx].value == Token::Keyword(Keyword::And)
        && let Some(right) = LAndExpr::parse(interpreter.clone(), tokens, left.idx + 1)
    {
        return Some(ParseResult::new(
            right.idx,
            LAndExpr::new_and(left.value, right.value),
        ));
    }

    if let Some(not) = LNotExpr::parse(interpreter.clone(), tokens, idx) {
        return Some(ParseResult::new(not.idx, LAndExpr::new_not(not.value)));
    }

    None
}

fn parse_or_expr(
    interpreter: Arc<Interpreter>,
    tokens: &[TokenNode],
    idx: usize,
) -> Option<ParseResult<LOrExpr>> {
    if idx >= tokens.len() {
        return None;
    }

    if let Some(left) = LAndExpr::parse(interpreter.clone(), tokens, idx)
        && left.idx + 1 < tokens.len()
        && tokens[left.idx].value == Token::Keyword(Keyword::Or)
        && let Some(right) = LOrExpr::parse(interpreter.clone(), tokens, left.idx + 1)
    {
        return Some(ParseResult::new(
            right.idx,
            LOrExpr::new_or(left.value, right.value),
        ));
    }

    if let Some(and) = LAndExpr::parse(interpreter.clone(), tokens, idx) {
        return Some(ParseResult::new(and.idx, LOrExpr::new_and(and.value)));
    }

    None
}

fn parse_expr(
    interpreter: Arc<Interpreter>,
    tokens: &[TokenNode],
    idx: usize,
) -> Option<ParseResult<Expr>> {
    if idx >= tokens.len() {
        return None;
    }

    if let Some(lor) = LOrExpr::parse(interpreter, tokens, idx) {
        return Some(ParseResult::new(lor.idx, Expr::new_or(lor.value)));
    }

    None
}
