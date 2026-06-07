use std::sync::Arc;

use crate::{
    Interpreter,
    eval::ParseResult,
    lexer::{ast::Expr, tokenize::Token},
    var::PyValue,
};

mod eval;
mod parse;

pub fn parse_expr(
    interpreter: Arc<Interpreter>,
    tokens: &[Token],
    idx: usize,
) -> Option<ParseResult<Expr>> {
    parse::parse_expr(interpreter, tokens, idx)
}

pub fn eval_expr(
    interpreter: Arc<Interpreter>,
    expr: Expr,
) -> Result<Arc<dyn PyValue>, Arc<dyn PyValue>> {
    eval::eval_expr(interpreter, expr)
}
