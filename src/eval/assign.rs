use std::sync::Arc;

use crate::{
    Interpreter,
    eval::{ParseResult, ast::Assign},
    lexer::tokenize::{Operator, Token, TokenKind},
    var::PyValue,
};

use super::expr::parse_lvalue;

pub fn parse_assign(
    interpreter: Arc<Interpreter>,
    tokens: &[Token],
    idx: usize,
) -> Option<ParseResult<Assign>> {
    if idx >= tokens.len() {
        return None;
    }

    if let Some(lvalue) = parse_lvalue(interpreter.clone(), tokens, idx)
        && lvalue.idx < tokens.len()
        && tokens[lvalue.idx].value == TokenKind::Operator(Operator::Assign)
        && let Some(rvalue) = super::expr::parse_expr(interpreter.clone(), tokens, lvalue.idx + 1)
    {
        return Some(ParseResult::new(
            rvalue.idx,
            Assign::new(lvalue.value, rvalue.value),
        ));
    }

    None
}

pub fn eval_assign(interpreter: Arc<Interpreter>, assign: Assign) -> Result<(), Arc<dyn PyValue>> {
    let value = super::expr::eval_expr(interpreter.clone(), assign.value)?;
    interpreter.set_var(&assign.target.name, value)
}
