use std::sync::Arc;

use crate::{Interpreter, error::InterpreterError, lexer::tokenize::Token};

pub mod ast;
pub mod indent;
pub mod line;
pub mod tokenize;

pub fn lex_line(
    interpreter: Arc<Interpreter>,
    new_line: &str,
) -> Result<Vec<Token>, InterpreterError> {
    let line = line::get_line(&interpreter.line_context, new_line)?;
    if let Some(line) = line {
        return tokenize::tokenize(line);
    }
    Ok(vec![])
}
