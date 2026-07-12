use std::sync::Arc;

use crate::{
    Interpreter,
    eval::{Eval, ParseResult, ast::Assign},
    lexer::tokenize::{Operator, Token, TokenNode},
    var::PyValue,
};

use super::expr::parse_lvalue;

impl Eval for Assign {
    fn eval(
        self: Box<Self>,
        interpreter: Arc<Interpreter>,
    ) -> Result<Option<Arc<dyn PyValue>>, Arc<dyn PyValue>> {
        eval_assign(interpreter, *self).map(|_| None)
    }
}

/// Attempts to parse `<lvalue> = <expr>` from `tokens` starting at `idx`.
///
/// Returns `None` if the token sequence does not match an assignment.
pub fn parse_assign(
    interpreter: Arc<Interpreter>,
    tokens: &[TokenNode],
    idx: usize,
) -> Option<ParseResult<Assign>> {
    if idx >= tokens.len() {
        return None;
    }

    if let Some(lvalue) = parse_lvalue(interpreter.clone(), tokens, idx)
        && lvalue.idx < tokens.len()
        && tokens[lvalue.idx].value == Token::Operator(Operator::Assign)
        && let Some(rvalue) = super::expr::parse_expr(interpreter.clone(), tokens, lvalue.idx + 1)
    {
        return Some(ParseResult::new(
            rvalue.idx,
            Assign::new(lvalue.value, rvalue.value),
        ));
    }

    None
}

/// Evaluates an [`Assign`] node: evaluates the right-hand side and binds it to the target name.
pub fn eval_assign(interpreter: Arc<Interpreter>, assign: Assign) -> Result<(), Arc<dyn PyValue>> {
    let value = super::expr::eval_expr(interpreter.clone(), assign.value)?;
    interpreter.set_var(&assign.target.name, value)
}
