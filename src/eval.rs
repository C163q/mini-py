use std::sync::Arc;

use crate::{
    Interpreter,
    error::InterpreterError,
    lexer::{self, tokenize::Token},
    types::tstr::PyStr,
    var::PyValue,
};

pub mod expr;

pub fn eval_line(
    interpreter: Arc<Interpreter>,
    line: &str,
) -> Result<Option<PyStr>, InterpreterError> {
    let tokens = lexer::lex_line(interpreter.clone(), line)?;
    eval_line_from_token(interpreter, &tokens)
}

pub fn output_value(
    interpreter: Arc<Interpreter>,
    value: Box<dyn PyValue>,
) -> Result<PyStr, InterpreterError> {
    if let Some(repr_func) = value.get_function("__repr__") {
        let repr_value = repr_func.call(interpreter.clone(), vec![value])?;
        if let Some(repr_str) = repr_value.as_any().downcast_ref::<PyStr>() {
            Ok(repr_str.clone())
        } else {
            Err(InterpreterError::new(
                "__repr__ did not return a string".to_string(),
            ))
        }
    } else if let Some(str_func) = value.get_function("__str__") {
        let str_value = str_func.call(interpreter.clone(), vec![value])?;
        if let Some(str_str) = str_value.as_any().downcast_ref::<PyStr>() {
            Ok(str_str.clone())
        } else {
            Err(InterpreterError::new(
                "__str__ did not return a string".to_string(),
            ))
        }
    } else {
        Err(InterpreterError::new(
            "Type does not support __repr__ or __str__".to_string(),
        ))
    }
}

fn eval_line_from_token(
    interpreter: Arc<Interpreter>,
    tokens: &[Token],
) -> Result<Option<PyStr>, InterpreterError> {
    if tokens.is_empty() {
        return Ok(None);
    }
    lexer::parser::eval_line(interpreter, tokens)
}
