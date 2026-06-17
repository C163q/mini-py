use std::sync::Arc;

use crate::{
    Interpreter,
    types::{error, tstr::PyStr},
    var::PyValue,
};

pub fn output_value(
    interpreter: Arc<Interpreter>,
    value: Arc<dyn PyValue>,
) -> Result<PyStr, Arc<dyn PyValue>> {
    if let Ok(repr_func) = value.get_var(interpreter.clone(), "__repr__") {
        let repr_value = crate::var::call::call(repr_func, interpreter.clone(), vec![value])?;
        if let Some(repr_str) = repr_value.as_any().downcast_ref::<PyStr>() {
            Ok(repr_str.clone())
        } else {
            Err(error::get_type_error(
                interpreter,
                "__repr__ did not return a string".to_string(),
            ))
        }
    } else if let Ok(str_func) = value.get_var(interpreter.clone(), "__str__") {
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
