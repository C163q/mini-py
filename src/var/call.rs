use std::sync::Arc;

use crate::{
    Interpreter,
    types::{error, function::PyFunction},
    var::PyValue,
};

fn call_inner(
    interpreter: Arc<Interpreter>,
    values: Vec<Arc<dyn PyValue>>,
) -> Result<Arc<dyn PyValue>, Arc<dyn PyValue>> {
    debug_assert!(!values.is_empty());

    if let Some(builtin) = values[0].as_any().downcast_ref::<PyFunction>() {
        return builtin.call(interpreter, values[1..].to_vec());
    }

    if let Ok(func) = values[0].get_binding(interpreter.clone(), "__call__") {
        return call_inner(
            interpreter,
            [func].into_iter().chain(values[1..].to_vec()).collect(),
        );
    }

    Err(error::get_type_error(
        interpreter,
        format!(
            "'{}' object is not callable",
            values[0].get_type().get_name()
        ),
    ))
}

pub fn call(
    func: Arc<dyn PyValue>,
    interpreter: Arc<Interpreter>,
    values: Vec<Arc<dyn PyValue>>,
) -> Result<Arc<dyn PyValue>, Arc<dyn PyValue>> {
    call_inner(interpreter, [func].into_iter().chain(values).collect())
}
