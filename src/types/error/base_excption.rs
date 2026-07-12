use std::sync::{Arc, Mutex, MutexGuard};

use crate::{
    Interpreter, get_type,
    types::{
        PyType,
        init::{self, PyFunctionMapper},
        tstr::PyStr,
    },
    var::{PyValue, manager::VarManager},
};

const TYPE_NAME: &str = "BaseException";

get_type!(TYPE_NAME);

pub fn init_type(interpreter: Arc<Interpreter>) {
    type Current = PyBaseException;
    init::init_type(
        interpreter.clone(),
        TYPE_NAME,
        [PyFunctionMapper::from_method(
            "__str__",
            interpreter.clone(),
            Current::__str__,
        )],
    );
}

/// The base exception type, backing all error variants until dedicated types are implemented.
#[derive(Debug)]
pub struct PyBaseException {
    message: String,
    vars: Mutex<VarManager>,
    ty: Arc<PyType>,
}

impl PyBaseException {
    pub fn new(interpreter: Arc<Interpreter>, message: String) -> Self {
        Self {
            message,
            vars: Mutex::new(VarManager::new()),
            ty: get_type(interpreter),
        }
    }
}

impl PyValue for PyBaseException {
    fn get_type(&self) -> Arc<PyType> {
        self.ty.clone()
    }

    fn get_var_manager(&self) -> MutexGuard<'_, VarManager> {
        self.vars.lock().unwrap()
    }
}

impl PyBaseException {
    pub fn __str__(&self, interpreter: Arc<Interpreter>, _values: Vec<Arc<dyn PyValue>>) -> PyStr {
        PyStr::new(interpreter, self.message.clone())
    }
}
