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

/// BaseException Value
#[derive(Debug, Clone)]
pub struct PyBaseException {
    message: String,
    ty: Arc<PyType>,
}

impl PyBaseException {
    pub fn new(interpreter: Arc<Interpreter>, message: String) -> Self {
        Self {
            message,
            ty: get_type(interpreter),
        }
    }
}

impl PyValue for PyBaseException {
    fn get_type(&self) -> Arc<PyType> {
        self.ty.clone()
    }
}

impl PyBaseException {
    pub fn __str__(&self, interpreter: Arc<Interpreter>, _values: Vec<Arc<dyn PyValue>>) -> PyStr {
        PyStr::new(interpreter, self.message.clone())
    }
}
