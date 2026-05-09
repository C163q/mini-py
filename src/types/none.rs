use std::sync::Arc;

use crate::{
    Interpreter, def_func_pair, get_type,
    types::{PyType, init, tstr::PyStr},
    var::PyValue,
};

const TYPE_NAME: &str = "none";

get_type!(TYPE_NAME);

pub fn init_type(interpreter: Arc<Interpreter>) {
    init::init_type(
        interpreter,
        TYPE_NAME,
        [def_func_pair!(__str__, PyNone, interpreter, &[TYPE_NAME])],
    );
}

/// None Value
#[derive(Debug, Clone)]
pub struct PyNone {
    ty: Arc<PyType>,
}

impl PyNone {
    pub fn new(interpreter: Arc<Interpreter>) -> Self {
        Self {
            ty: get_type(interpreter),
        }
    }
}

impl PyValue for PyNone {
    fn get_type(&self) -> Arc<PyType> {
        self.ty.clone()
    }
}

impl PyNone {
    pub fn __str__(&self, interpreter: Arc<Interpreter>, _values: Vec<Box<dyn PyValue>>) -> PyStr {
        PyStr::new(interpreter, "None".to_string())
    }
}
