use std::sync::Arc;

use crate::{Interpreter, error::InterpreterError, types::tstr::PyStr, var::PyValue};

pub fn output_value(
    interpreter: Arc<Interpreter>,
    value: Arc<dyn PyValue>,
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
