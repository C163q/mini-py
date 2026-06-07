use std::sync::Arc;

use crate::{
    Interpreter,
    types::{
        self, error,
        function::{BuiltinPyFunction, PyFunction},
    },
    var::PyValue,
};

type ArcValue = Arc<dyn PyValue>;
type ValueArgs = Vec<ArcValue>;
type FuncResult<T> = Result<T, Arc<dyn PyValue>>;

type BasicFunc<T> = Box<dyn Fn(Arc<Interpreter>, ValueArgs) -> T + Send + Sync + 'static>;
type ResultFunc<T> =
    Box<dyn Fn(Arc<Interpreter>, ValueArgs) -> FuncResult<T> + Send + Sync + 'static>;
type BasicMethodFunc<T, R> =
    Box<dyn Fn(&T, Arc<Interpreter>, ValueArgs) -> R + Send + Sync + 'static>;
type MethodFunc<T> =
    Box<dyn Fn(&T, Arc<Interpreter>, ValueArgs) -> FuncResult<ArcValue> + Send + Sync + 'static>;

pub fn check_args(
    interpreter: Arc<Interpreter>,
    expected: &[&str],
    got: &[Arc<dyn PyValue>],
) -> Result<(), Arc<dyn PyValue>> {
    if expected.len() != got.len() {
        return Err(error::get_type_error(
            interpreter,
            format!("Expected {} arguments, got {}", expected.len(), got.len()),
        ));
    }

    for (i, (&expected_type, arg)) in expected.iter().zip(got.iter()).enumerate() {
        if expected_type != types::init::ANY_TYPE_NAME && arg.get_type().get_name() != expected_type
        {
            return Err(error::get_type_error(
                interpreter,
                format!(
                    "Argument {}: expected type '{}', got '{}'",
                    i + 1,
                    expected_type,
                    arg.get_type().get_name()
                ),
            ));
        }
    }

    Ok(())
}

pub fn method_to_func<T: PyValue>(
    type_name: &'static str,
    func: MethodFunc<T>,
) -> ResultFunc<ArcValue> {
    Box::new(move |interpreter, mut values| {
        if values.is_empty() {
            return Err(error::get_type_error(
                interpreter,
                format!("Expected type {}, got no arguments", type_name),
            ));
        }
        let other_values = values.split_off(1);
        let value = values[0].as_any().downcast_ref::<T>().ok_or_else(|| {
            error::get_type_error(
                interpreter.clone(),
                format!(
                    "Expected first argument to be of type {}, got {}",
                    type_name,
                    values[0].get_type().get_name()
                ),
            )
        })?;
        func(value, interpreter, other_values)
    })
}

pub fn with_result_handler<T: 'static>(func: BasicFunc<T>) -> ResultFunc<T> {
    Box::new(move |interpreter, value| Ok(func(interpreter, value)))
}

pub fn to_arc_func(func: ResultFunc<ArcValue>) -> BuiltinPyFunction {
    Arc::from(func)
}

pub fn to_pyfunc(interpreter: Arc<Interpreter>, func: ResultFunc<ArcValue>) -> PyFunction {
    PyFunction::new(interpreter, to_arc_func(func))
}

pub fn method_to_pyfunc<T: PyValue, R: PyValue>(
    type_name: &'static str,
    interpreter_arc: Arc<Interpreter>,
    func: BasicMethodFunc<T, R>,
) -> PyFunction {
    to_pyfunc(
        interpreter_arc,
        method_to_func(
            type_name,
            Box::new(move |self_ref, interpreter, values| {
                Ok(Arc::new(func(self_ref, interpreter, values)))
            }),
        ),
    )
}
