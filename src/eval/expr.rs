//! Expression evaluation.
//!
//! [`Eval::eval`] returns `Result<Option<Arc<dyn PyValue>>, _>` because statements have no
//! result, but an [`Expr`] always evaluates to a value — the `Option` would always be `Some`.
//! To spare callers an unwrap they know will never fail, this module (and its private `eval`
//! submodule, which implements the actual per-precedence-level `eval_*` functions) instead
//! exposes plain `Result<Arc<dyn PyValue>, _>` functions such as [`eval_expr`]. Other statement
//! modules (`assign`, `sif`, `swhile`, ...) call [`eval_expr`] directly to evaluate a
//! sub-expression, rather than going through [`Eval::eval`]/[`Eval::eval_with_state`].
//!
//! [`Eval::eval`]: crate::eval::Eval::eval
//! [`Eval::eval_with_state`]: crate::eval::Eval::eval_with_state

use std::sync::Arc;

use crate::{
    Interpreter,
    eval::{Eval, expr::ast::Expr},
    var::PyValue,
};

pub mod ast;
mod eval;
mod parse;

impl Eval for Expr {
    fn eval(
        self: Box<Self>,
        interpreter: Arc<Interpreter>,
    ) -> Result<Option<Arc<dyn PyValue>>, Arc<dyn PyValue>> {
        eval::eval_expr(interpreter, *self).map(Some)
    }
}

/// Evaluates an [`Expr`] and returns the resulting Python value.
pub fn eval_expr(
    interpreter: Arc<Interpreter>,
    expr: Expr,
) -> Result<Arc<dyn PyValue>, Arc<dyn PyValue>> {
    eval::eval_expr(interpreter, expr)
}
