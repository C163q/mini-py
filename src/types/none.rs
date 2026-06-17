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

const TYPE_NAME: &str = "none";

get_type!(TYPE_NAME);

pub fn init_type(interpreter: Arc<Interpreter>) {
    type Current = PyNone;
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

/// None Value
#[derive(Debug)]
pub struct PyNone {
    ty: Arc<PyType>,
    vars: Mutex<VarManager>,
}

impl PyNone {
    pub fn new(interpreter: Arc<Interpreter>) -> Self {
        Self {
            ty: get_type(interpreter),
            vars: Mutex::new(VarManager::new()),
        }
    }
}

impl Clone for PyNone {
    fn clone(&self) -> Self {
        Self {
            ty: self.ty.clone(),
            vars: Mutex::new(self.vars.lock().unwrap().clone()),
        }
    }
}

impl PyValue for PyNone {
    fn get_type(&self) -> Arc<PyType> {
        self.ty.clone()
    }

    fn get_var_manager(&self) -> MutexGuard<'_, VarManager> {
        self.vars.lock().unwrap()
    }
}

impl PyNone {
    pub fn __str__(&self, interpreter: Arc<Interpreter>, _values: Vec<Arc<dyn PyValue>>) -> PyStr {
        PyStr::new(interpreter, "None".to_string())
    }
}
