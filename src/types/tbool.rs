use std::{fmt::Display, sync::Arc};

use crate::{
    Interpreter, get_type,
    types::{
        PyType,
        init::{self, PyFunctionMapper},
        tstr::PyStr,
    },
    var::PyValue,
};

const TYPE_NAME: &str = "bool";

get_type!(TYPE_NAME);

pub fn init_type(interpreter: Arc<Interpreter>) {
    type Current = PyBool;
    init::init_type(
        interpreter.clone(),
        TYPE_NAME,
        [
            PyFunctionMapper::from_method("__str__", interpreter.clone(), Current::__str__),
            PyFunctionMapper::from_method("__bool__", interpreter.clone(), Current::__bool__),
        ],
    );
}

// Bool Value
#[derive(Debug, Clone)]
pub struct PyBool {
    ty: Arc<PyType>,
    value: bool,
}

impl PyBool {
    pub fn new(interpreter: Arc<Interpreter>, value: bool) -> Self {
        Self {
            ty: get_type(interpreter),
            value,
        }
    }

    pub fn get_value(&self) -> bool {
        self.value
    }
}

impl PyValue for PyBool {
    fn get_type(&self) -> Arc<PyType> {
        self.ty.clone()
    }
}

impl Display for PyBool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.value {
            true => write!(f, "True"),
            false => write!(f, "False"),
        }
    }
}

impl PyBool {
    pub fn __str__(&self, interpreter: Arc<Interpreter>, _values: Vec<Arc<dyn PyValue>>) -> PyStr {
        PyStr::new(interpreter, self.to_string())
    }

    pub fn __bool__(&self, _: Arc<Interpreter>, _values: Vec<Arc<dyn PyValue>>) -> PyBool {
        self.clone()
    }
}
