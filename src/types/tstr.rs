use std::{fmt::Display, sync::Arc};

use num_bigint::BigInt;

use crate::{
    Interpreter, def_func_pair, get_type,
    types::{PyType, float::PyFloat, init, int::PyInt, tbool::PyBool},
    var::PyValue,
};

const TYPE_NAME: &str = "str";

get_type!(TYPE_NAME);

pub fn init_type(interpreter: Arc<Interpreter>) {
    init::init_type(
        interpreter,
        TYPE_NAME,
        [
            def_func_pair!(__str__, PyStr, interpreter, &[TYPE_NAME]),
            def_func_pair!(__int__, PyStr, interpreter, &[TYPE_NAME]),
            def_func_pair!(__bool__, PyStr, interpreter, &[TYPE_NAME]),
            def_func_pair!(__float__, PyStr, interpreter, &[TYPE_NAME]),
            def_func_pair!(__add__, PyStr, interpreter, &[TYPE_NAME, TYPE_NAME]),
        ],
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
    pub fn __str__(&self, _: Arc<Interpreter>, _values: Vec<Arc<dyn PyValue>>) -> PyStr {
        self.clone()
    }

    pub fn __int__(&self, interpreter: Arc<Interpreter>, _values: Vec<Arc<dyn PyValue>>) -> PyInt {
        let int_value = str::parse::<BigInt>(self.value.trim()).unwrap_or_else(|_| {
            // TODO: Implement error handling
            panic!("Cannot convert string '{}' to int", self.value);
        });
        PyInt::new(interpreter, int_value)
    }

    pub fn __bool__(
        &self,
        interpreter: Arc<Interpreter>,
        _values: Vec<Arc<dyn PyValue>>,
    ) -> PyBool {
        PyBool::new(interpreter, !self.value.is_empty())
    }

    pub fn __float__(
        &self,
        interpreter: Arc<Interpreter>,
        _values: Vec<Arc<dyn PyValue>>,
    ) -> PyFloat {
        let s = self.value.trim();
        let float_value = if s.eq_ignore_ascii_case("inf")
            || s.eq_ignore_ascii_case("+inf")
            || s.eq_ignore_ascii_case("infinity")
            || s.eq_ignore_ascii_case("+infinity")
        {
            f64::INFINITY
        } else if s.eq_ignore_ascii_case("-inf") || s.eq_ignore_ascii_case("-infinity") {
            f64::NEG_INFINITY
        } else if s.eq_ignore_ascii_case("nan") {
            f64::NAN
        } else {
            str::parse::<f64>(s).unwrap_or_else(|_| {
                // TODO: Implement error handling
                panic!("Cannot convert string '{}' to float", self.value);
            })
        };
        PyFloat::new(interpreter, float_value)
    }

    pub fn __add__(&self, interpreter: Arc<Interpreter>, values: Vec<Arc<dyn PyValue>>) -> PyStr {
        let other = values[0].as_any().downcast_ref::<PyStr>().unwrap();
        PyStr::new(interpreter, format!("{}{}", self.value, other.value))
    }
}
