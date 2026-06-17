use std::{
    fmt::{Debug, Display},
    sync::{Arc, Mutex, MutexGuard},
};

use crate::{
    Interpreter, get_type,
    types::{PyType, init::PyFunctionMapper, tstr::PyStr},
    var::{PyValue, manager::VarManager},
};

pub mod wrapper;

pub type BuiltinPyFunction = Arc<
    dyn Fn(Arc<Interpreter>, Vec<Arc<dyn PyValue>>) -> Result<Arc<dyn PyValue>, Arc<dyn PyValue>>
        + Send
        + Sync
        + 'static,
>;

const TYPE_NAME: &str = "function";

get_type!(TYPE_NAME);

pub fn init_type(interpreter: Arc<Interpreter>) {
    type Current = PyFunction;

    // NEVER Call init::init_type() here, it will cause circular dependency

    let ty = interpreter
        .clone()
        .register_type(Arc::new(PyType::new(TYPE_NAME, interpreter.clone())))
        .expect("Failed to register type");

    for PyFunctionMapper { name, func } in [
        PyFunctionMapper::new(
            "__call__",
            wrapper::to_pyfunc(
                interpreter.clone(),
                wrapper::method_to_func(TYPE_NAME, Box::new(Current::__call__)),
            ),
        ),
        PyFunctionMapper::from_method("__str__", interpreter.clone(), Current::__str__),
    ] {
        ty.add_function(name, func);
    }
}

/// Function Value
pub struct PyFunction {
    ty: Arc<PyType>,
    vars: Mutex<VarManager>,
    value: BuiltinPyFunction,
}

impl Debug for PyFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<class 'function'>")
    }
}

impl Clone for PyFunction {
    fn clone(&self) -> Self {
        Self {
            ty: self.ty.clone(),
            vars: Mutex::new(self.vars.lock().unwrap().clone()),
            value: self.value.clone(),
        }
    }
}

impl PyFunction {
    pub fn new(interpreter: Arc<Interpreter>, value: BuiltinPyFunction) -> Self {
        Self {
            ty: get_type(interpreter),
            vars: Mutex::new(VarManager::new()),
            value,
        }
    }
}

impl PyValue for PyFunction {
    fn get_type(&self) -> Arc<PyType> {
        self.ty.clone()
    }

    fn get_var_manager(&self) -> MutexGuard<'_, VarManager> {
        self.vars.lock().unwrap()
    }
}

impl Display for PyFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<class 'function'>")
    }
}

impl PyFunction {
    fn __str__(&self, interpreter: Arc<Interpreter>, _values: Vec<Arc<dyn PyValue>>) -> PyStr {
        PyStr::new(interpreter, String::from("<class 'function'>"))
    }

    fn __call__(
        &self,
        interpreter: Arc<Interpreter>,
        values: Vec<Arc<dyn PyValue>>,
    ) -> Result<Arc<dyn PyValue>, Arc<dyn PyValue>> {
        self.call(interpreter, values)
    }

    pub fn call(
        &self,
        interpreter: Arc<Interpreter>,
        values: Vec<Arc<dyn PyValue>>,
    ) -> Result<Arc<dyn PyValue>, Arc<dyn PyValue>> {
        (self.value)(interpreter, values)
    }
}
