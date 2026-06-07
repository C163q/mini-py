use std::sync::Arc;

use crate::{Interpreter, lexer::tokenize::Token, types::error, var::PyValue};

pub mod ast;
pub mod indent;
pub mod line;
pub mod tokenize;

pub fn lex_line(
    interpreter: Arc<Interpreter>,
    new_line: &str,
) -> Result<Vec<Token>, Arc<dyn PyValue>> {
    let line = line::get_line(&interpreter.line_context, new_line)
        .map_err(|e| error::get_syntax_error(interpreter.clone(), e.to_string()))?;
    if let Some(line) = line {
        return tokenize::tokenize(line)
            .map_err(|e| error::get_syntax_error(interpreter, e.to_string()));
    }
    Ok(vec![])
}
