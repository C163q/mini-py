use std::sync::Arc;

use crate::{
    Interpreter,
    eval::{Eval, ParseResult, ast::Expr},
    lexer::tokenize::TokenNode,
    var::PyValue,
};

mod eval;
mod parse;

pub(super) use parse::parse_lvalue;

impl Eval for Expr {
    fn eval(
        self: Box<Self>,
        interpreter: Arc<Interpreter>,
    ) -> Result<Option<Arc<dyn PyValue>>, Arc<dyn PyValue>> {
        eval::eval_expr(interpreter, *self).map(Some)
    }
}

/// Attempts to parse an [`Expr`] from `tokens` starting at `idx`.
///
/// Returns `None` if no valid expression begins at that position.
pub fn parse_expr(
    interpreter: Arc<Interpreter>,
    tokens: &[TokenNode],
    idx: usize,
) -> Option<ParseResult<Expr>> {
    parse::parse_expr(interpreter, tokens, idx)
}

/// Evaluates an [`Expr`] and returns the resulting Python value.
pub fn eval_expr(
    interpreter: Arc<Interpreter>,
    expr: Expr,
) -> Result<Arc<dyn PyValue>, Arc<dyn PyValue>> {
    eval::eval_expr(interpreter, expr)
}
