use std::{fmt::Display, sync::Arc};

use crate::{
    Interpreter, def_func_pair, get_type,
    types::{PyType, init},
    var::PyValue,
};

const TYPE_NAME: &str = "str";

get_type!(TYPE_NAME);

pub fn init_type(interpreter: Arc<Interpreter>) {
    init::init_type(
        interpreter,
        TYPE_NAME,
        [def_func_pair!(__str__, PyStr, interpreter, &[TYPE_NAME])],
    );
}

/// String
#[derive(Debug, Clone)]
pub struct PyStr {
    ty: Arc<PyType>,
    value: String,
}

impl PyStr {
    pub fn new(interpreter: Arc<Interpreter>, value: String) -> Self {
        Self {
            ty: get_type(interpreter),
            value,
        }
    }
}

impl PyValue for PyStr {
    fn get_type(&self) -> Arc<PyType> {
        self.ty.clone()
    }
}

impl Display for PyStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl PyStr {
    pub fn __str__(&self, _: Arc<Interpreter>, _values: Vec<Box<dyn PyValue>>) -> PyStr {
        self.clone()
    }
}
