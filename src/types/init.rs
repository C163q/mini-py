use std::sync::Arc;

use crate::{
    Interpreter,
    error::InterpreterError,
    func::PyFunction,
    types::{PyType, int, none, tbool, tstr},
    var::PyValue,
};

pub const ANY_TYPE_NAME: &str = "$builtin_any";

pub(super) fn init_type<I>(interpreter: Arc<Interpreter>, type_name: &str, mapper: I)
where
    I: IntoIterator<Item = (&'static str, PyFunction)>,
{
    let ty = interpreter
        .register_type(Arc::new(PyType::new(type_name)))
        .expect("Failed to register type");

    for (name, func) in mapper {
        ty.add_function(name, func);
    }
}

pub fn check_args(expected: &[&str], got: &[Box<dyn PyValue>]) -> Result<(), InterpreterError> {
    if expected.len() != got.len() {
        return Err(InterpreterError::new(format!(
            "Expected {} arguments, got {}",
            expected.len(),
            got.len()
        )));
    }

    for (i, (&expected_type, arg)) in expected.iter().zip(got.iter()).enumerate() {
        if expected_type != ANY_TYPE_NAME && arg.get_type().get_name() != expected_type {
            return Err(InterpreterError::new(format!(
                "Argument {}: expected type '{}', got '{}'",
                i + 1,
                expected_type,
                arg.get_type().get_name()
            )));
        }
    }

    Ok(())
}

pub fn register_types(interpreter: Arc<Interpreter>) {
    none::init_type(interpreter.clone());
    int::init_type(interpreter.clone());
    tstr::init_type(interpreter.clone());
    tbool::init_type(interpreter.clone());
}

#[macro_export]
macro_rules! get_type {
    ($ty_name:expr) => {
        pub fn get_type(interpreter: Arc<Interpreter>) -> Arc<PyType> {
            interpreter
                .get_type($ty_name)
                .expect("Type not registered")
                .clone()
        }
    };
}

#[macro_export]
macro_rules! def_func_pair {
    ($name:tt, $type:ty, $interpreter:ident, $expected:expr) => {
        (
            stringify!($name),
            $crate::func::PyFunction::new_builtin(|$interpreter, mut values| {
                $crate::types::init::check_args($expected, &values)?;
                let other_values = values.split_off(1);
                match values[0].as_any().downcast_ref::<$type>() {
                    None => {
                        unreachable!(
                            "This should never happen, type checking should have caught this"
                        )
                    }
                    Some(s) => Ok(Box::new(s.$name($interpreter, other_values))),
                }
            }),
        )
    };
}
