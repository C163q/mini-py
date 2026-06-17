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

const TYPE_NAME: &str = "object";

get_type!(TYPE_NAME);

pub fn init_type(interpreter: Arc<Interpreter>) {
    type Current = PyObject;
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

/// Object Value
#[derive(Debug)]
pub struct PyObject {
    ty: Arc<PyType>,
    vars: Mutex<VarManager>,
}

impl PyObject {
    pub fn new(interpreter: Arc<Interpreter>) -> Self {
        Self {
            ty: get_type(interpreter),
            vars: Mutex::new(VarManager::new()),
        }
    }
}

impl PyValue for PyObject {
    fn get_type(&self) -> Arc<PyType> {
        self.ty.clone()
    }

    fn get_var_manager(&self) -> MutexGuard<'_, VarManager> {
        self.vars.lock().unwrap()
    }
}

impl Clone for PyObject {
    fn clone(&self) -> Self {
        Self {
            ty: self.ty.clone(),
            vars: Mutex::new(self.vars.lock().unwrap().clone()),
        }
    }
}

impl PyObject {
    pub fn __str__(&self, interpreter: Arc<Interpreter>, _values: Vec<Arc<dyn PyValue>>) -> PyStr {
        PyStr::new(interpreter, "<Object>".to_string())
    }
}
