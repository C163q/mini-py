use std::sync::Arc;

use crate::{
    Interpreter, get_type,
    types::{
        PyType,
        init::{self, PyFunctionMapper},
        tstr::PyStr,
    },
    var::PyValue,
};

const TYPE_NAME: &str = "object";

get_type!(TYPE_NAME);

pub fn init_type(interpreter: Arc<Interpreter>) {
    type Current = PyObject;
    init::init_type(
        interpreter.clone(),
        TYPE_NAME,
        [PyFunctionMapper::from_method("__str__", Current::__str__)],
    );
}

/// Object Value
#[derive(Debug, Clone)]
pub struct PyObject {
    ty: Arc<PyType>,
}

impl PyObject {
    pub fn new(interpreter: Arc<Interpreter>) -> Self {
        Self {
            ty: get_type(interpreter),
        }
    }
}

impl PyValue for PyObject {
    fn get_type(&self) -> Arc<PyType> {
        self.ty.clone()
    }
}

impl PyObject {
    pub fn __str__(&self, interpreter: Arc<Interpreter>, _values: Vec<Arc<dyn PyValue>>) -> PyStr {
        PyStr::new(interpreter, "None".to_string())
    }
}
