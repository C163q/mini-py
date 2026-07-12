use std::sync::Arc;

use crate::{
    Interpreter,
    types::{
        PyType, error, float,
        function::{self, PyFunction},
        int, none, tbool, tstr, ttype,
    },
    var::{IntoPyValueArcResult, PyValue},
};

/// A sentinel type name accepted by [`check_args`] or other functions to mean "any type is
/// acceptable".
///
/// [`check_args`]: crate::types::function::wrapper::check_args
pub const ANY_TYPE_NAME: &str = "$builtin_any";

/// Associates a method name with its [`PyFunction`] implementation, used when registering
/// methods on a type during initialization.
#[derive(Debug, Clone)]
pub struct PyFunctionMapper {
    pub name: &'static str,
    pub func: PyFunction,
}

impl PyFunctionMapper {
    /// Creates a `PyFunctionMapper` from a pre-built [`PyFunction`].
    pub fn new(name: &'static str, func: PyFunction) -> Self {
        Self { name, func }
    }

    /// Creates a `PyFunctionMapper` from a typed method, wrapping it via [`method_to_pyfunc`].
    ///
    /// [`method_to_pyfunc`]: crate::types::function::wrapper::method_to_pyfunc
    pub fn from_method<T, R, F>(name: &'static str, interpreter: Arc<Interpreter>, func: F) -> Self
    where
        T: PyValue,
        R: IntoPyValueArcResult,
        F: (Fn(&T, Arc<Interpreter>, Vec<Arc<dyn PyValue>>) -> R) + Send + Sync + 'static,
    {
        Self::new(
            name,
            function::wrapper::method_to_pyfunc(name, interpreter, move |value, interp, args| {
                func(value, interp, args).into_pyvalue_arc()
            }),
        )
    }
}

/// Registers a new type and attaches the given methods to it.
///
/// Note: `type` and `function` cannot use this helper because they must be initialized before
/// other types and have circular dependencies between each other. They have their own
/// `init_raw_type` / `init_type` paths.
pub(super) fn init_type<I>(interpreter: Arc<Interpreter>, type_name: &str, mapper: I)
where
    I: IntoIterator<Item = PyFunctionMapper>,
{
    let ty = interpreter
        .clone()
        .register_type(Arc::new(PyType::new(type_name, interpreter)))
        .expect("Failed to register type");

    for PyFunctionMapper { name, func } in mapper {
        ty.add_function(name, func);
    }
}

/// Registers all built-in types with the interpreter in the correct initialization order.
pub fn register_types(interpreter: Arc<Interpreter>) {
    // Special handling for `type`, since it has cyclic reference with itself and other types.
    ttype::init_raw_type(interpreter.clone());
    function::init_type(interpreter.clone());
    ttype::init_functions(interpreter.clone());

    // Normal types
    error::base_excption::init_type(interpreter.clone());
    none::init_type(interpreter.clone());
    int::init_type(interpreter.clone());
    tstr::init_type(interpreter.clone());
    tbool::init_type(interpreter.clone());
    float::init_type(interpreter.clone());
}

/// Registers all built-in functions (e.g. `print`) as global variables in the interpreter.
pub fn register_functions(interpreter: Arc<Interpreter>) -> Result<(), Arc<dyn PyValue>> {
    function::builtin::register_print(interpreter.clone())?;

    Ok(())
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
