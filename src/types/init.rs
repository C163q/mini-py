use std::sync::Arc;

use crate::{
    Interpreter,
    types::{
        PyType, error, float, function::{self, PyFunction}, int, none, tbool, tstr
    },
    var::PyValue,
};

pub const ANY_TYPE_NAME: &str = "$builtin_any";

#[derive(Debug, Clone)]
pub struct PyFunctionMapper {
    pub name: &'static str,
    pub func: PyFunction,
}

impl PyFunctionMapper {
    pub fn new(name: &'static str, func: PyFunction) -> Self {
        Self { name, func }
    }

    pub fn from_method<T, R, F>(name: &'static str, interpreter: Arc<Interpreter>, func: F) -> Self
    where
        T: PyValue,
        R: PyValue,
        F: (Fn(&T, Arc<Interpreter>, Vec<Arc<dyn PyValue>>) -> R) + Send + Sync + 'static,
    {
        Self::new(
            name,
            function::wrapper::method_to_pyfunc(name, interpreter, Box::new(func)),
        )
    }
}

/// TODO: init_type() only register type, and add functions in their own module, to avoid circular
/// dependency
pub(super) fn init_type<I>(interpreter: Arc<Interpreter>, type_name: &str, mapper: I)
where
    I: IntoIterator<Item = PyFunctionMapper>,
{
    let ty = interpreter
        .register_type(Arc::new(PyType::new(type_name)))
        .expect("Failed to register type");

    for PyFunctionMapper { name, func } in mapper {
        ty.add_function(name, func);
    }
}

pub fn register_types(interpreter: Arc<Interpreter>) {
    function::init_type(interpreter.clone());
    error::base_excption::init_type(interpreter.clone());
    none::init_type(interpreter.clone());
    int::init_type(interpreter.clone());
    tstr::init_type(interpreter.clone());
    tbool::init_type(interpreter.clone());
    float::init_type(interpreter.clone());
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
