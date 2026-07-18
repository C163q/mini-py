use std::sync::Arc;

use crate::{
    Interpreter,
    error::InterpreterError,
    types::{error, tstr::PyStr},
    var::PyValue,
};

/// Converts `value` to a [`PyStr`] for display by calling `__repr__`, falling back to `__str__`.
///
/// Returns `Err(TypeError)` if neither method is available or if either method does not return
/// a string.
pub fn output_value(
    interpreter: Arc<Interpreter>,
    value: Arc<dyn PyValue>,
) -> Result<PyStr, Arc<dyn PyValue>> {
    if let Ok(repr_func) = value.get_binding(interpreter.clone(), "__repr__") {
        let repr_value = crate::var::call::call(repr_func, interpreter.clone(), vec![value])?;
        if let Some(repr_str) = repr_value.as_any().downcast_ref::<PyStr>() {
            Ok(repr_str.clone())
        } else {
            Err(error::get_type_error(
                interpreter,
                "__repr__ did not return a string".to_string(),
            ))
        }
    } else if let Ok(str_func) = value.get_binding(interpreter.clone(), "__str__") {
        let str_value = crate::var::call::call(str_func, interpreter.clone(), vec![value])?;
        if let Some(str_str) = str_value.as_any().downcast_ref::<PyStr>() {
            Ok(str_str.clone())
        } else {
            Err(error::get_type_error(
                interpreter,
                "__str__ did not return a string".to_string(),
            ))
        }
    } else {
        Err(error::get_type_error(
            interpreter,
            "Type does not support __repr__ or __str__".to_string(),
        ))
    }
}

/// Like [`output_value`], but converts errors into [`InterpreterError`] instead of a Python
/// exception — used when formatting an [`PyValue`] that is not allowed to raise an exception.
pub fn output_err_value(
    interpreter: Arc<Interpreter>,
    value: Arc<dyn PyValue>,
) -> Result<PyStr, InterpreterError> {
    if let Ok(repr_func) = value.get_binding(interpreter.clone(), "__repr__") {
        let repr_value = crate::var::call::call(repr_func, interpreter.clone(), vec![value])
            .map_err(|e| {
                InterpreterError::new_unhandled(format!(
                    "When handling error, another error occurred: {}",
                    e.get_type().get_name()
                ))
            })?;
        if let Some(repr_str) = repr_value.as_any().downcast_ref::<PyStr>() {
            Ok(repr_str.clone())
        } else {
            Err(InterpreterError::new_unhandled(
                "__repr__ did not return a string".to_string(),
            ))
        }
    } else if let Ok(str_func) = value.get_binding(interpreter.clone(), "__str__") {
        let str_value = crate::var::call::call(str_func, interpreter.clone(), vec![value])
            .map_err(|e| {
                InterpreterError::new_unhandled(format!(
                    "When handling error, another error occurred: {}",
                    e.get_type().get_name()
                ))
            })?;
        if let Some(str_str) = str_value.as_any().downcast_ref::<PyStr>() {
            Ok(str_str.clone())
        } else {
            Err(InterpreterError::new_unhandled(
                "__str__ did not return a string".to_string(),
            ))
        }
    } else {
        Err(InterpreterError::new_unhandled(
            "Type does not support __repr__ or __str__".to_string(),
        ))
    }
}
