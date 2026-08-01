use std::sync::Arc;

use crate::{
    Interpreter,
    error::PyError,
    eval::{
        Eval, Parse, ParseResult,
        basic::ast::LValue,
        expr::{self, ast::Expr},
    },
    lexer::tokenize::{Operator, TokenNode},
    var::PyValue,
};

/// An assignment statement: `<lvalue> = <expr>`.
#[derive(Debug, Clone)]
pub struct Assign {
    pub target: LValue,
    pub value: Expr,
}

impl Assign {
    pub fn new(target: LValue, value: Expr) -> Self {
        Self { target, value }
    }
}

impl Eval for Assign {
    fn eval(
        self: Box<Self>,
        interpreter: Arc<Interpreter>,
    ) -> Result<Option<Arc<dyn PyValue>>, PyError> {
        eval_assign(interpreter, *self)
            .map(|_| None)
            .map_err(PyError::new_exception)
    }
}

impl Parse for Assign {
    fn parse(
        interpreter: Arc<Interpreter>,
        tokens: &[TokenNode],
        idx: usize,
    ) -> Option<ParseResult<Assign>> {
        parse_assign(interpreter, tokens, idx)
    }
}

/// Attempts to parse `<lvalue> = <expr>` from `tokens` starting at `idx`.
///
/// Returns `None` if the token sequence does not match an assignment.
fn parse_assign(
    interpreter: Arc<Interpreter>,
    tokens: &[TokenNode],
    idx: usize,
) -> Option<ParseResult<Assign>> {
    if idx >= tokens.len() {
        return None;
    }

    if let Some(lvalue) = LValue::parse(interpreter.clone(), tokens, idx)
        && let Some(assign_op) = Operator::parse(interpreter.clone(), tokens, lvalue.idx)
        && assign_op.value == Operator::Assign
        && let Some(rvalue) = Expr::parse(interpreter.clone(), tokens, assign_op.idx)
    {
        return Some(ParseResult::new(
            rvalue.idx,
            Assign::new(lvalue.value, rvalue.value),
        ));
    }

    None
}

/// Evaluates an [`Assign`] node: evaluates the right-hand side and binds it to the target name.
fn eval_assign(interpreter: Arc<Interpreter>, assign: Assign) -> Result<(), Arc<dyn PyValue>> {
    let value = expr::eval_expr(interpreter.clone(), assign.value)?;
    interpreter.set_var(&assign.target.name, value)
}
