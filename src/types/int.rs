use std::{fmt::Display, sync::Arc};

use num_bigint::BigInt;

use crate::{
    Interpreter, def_func_pair, get_type,
    types::{PyType, init, tstr::PyStr},
    var::PyValue,
};

const TYPE_NAME: &str = "int";

get_type!(TYPE_NAME);

pub fn init_type(interpreter: Arc<Interpreter>) {
    init::init_type(
        interpreter,
        TYPE_NAME,
        [def_func_pair!(__str__, PyInt, interpreter, &[TYPE_NAME])],
    )
}

/// None Value
#[derive(Debug, Clone)]
pub struct PyInt {
    ty: Arc<PyType>,
    value: BigInt,
}

impl PyInt {
    pub fn new(interpreter: Arc<Interpreter>, value: BigInt) -> Self {
        Self {
            ty: get_type(interpreter),
            value,
        }
    }
}

impl PyValue for PyInt {
    fn get_type(&self) -> Arc<PyType> {
        self.ty.clone()
    }
}

impl Display for PyInt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl PyInt {
    pub fn __str__(&self, interpreter: Arc<Interpreter>) -> PyStr {
        PyStr::new(interpreter, self.value.to_string())
    }
}
