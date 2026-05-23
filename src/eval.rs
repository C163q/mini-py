use std::sync::Arc;

use crate::{
    Interpreter,
    error::InterpreterError,
    lexer::{self, tokenize::Token},
    types::tstr::PyStr,
};

pub mod expr;
pub mod output;

#[derive(Debug, Clone)]
pub struct ParseResult<T> {
    /// Next index to parse. It should be the index of the first token that is not parsed yet.
    pub idx: usize,
    pub value: T,
}

impl<T> ParseResult<T> {
    pub fn new(idx: usize, value: T) -> Self {
        Self { idx, value }
    }
}

pub fn eval_line(
    interpreter: Arc<Interpreter>,
    line: &str,
) -> Result<Option<PyStr>, InterpreterError> {
    let tokens = lexer::lex_line(interpreter.clone(), line)?;
    eval_line_from_token(interpreter, &tokens)
}

fn eval_line_from_token(
    interpreter: Arc<Interpreter>,
    tokens: &[Token],
) -> Result<Option<PyStr>, InterpreterError> {
    if tokens.is_empty() {
        return Ok(None);
    }
    parse_and_eval_line(interpreter, tokens)
}

/// Ok(PyStr) if the line is valid, Err(InterpreterError) otherwise.
///
/// Note that PyStr is only used for REPL. Any effect of the line should be applied to the
/// interpreter.
fn parse_and_eval_line(
    interpreter: Arc<Interpreter>,
    tokens: &[Token],
) -> Result<Option<PyStr>, InterpreterError> {
    let idx = 0;

    // <expr>
    if let Some(expr) = expr::parse_expr(interpreter.clone(), tokens, idx)
        && expr.idx == tokens.len()
    {
        let value = expr::eval_expr(interpreter.clone(), expr.value)?;
        let output = output::output_value(interpreter.clone(), value)?;
        return Ok(Some(output));
    }

    Err(InterpreterError::new("Invalid syntax".to_string()))
}
