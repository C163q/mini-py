use std::{fmt::Display, sync::Arc};

use num_bigint::BigInt;

use crate::{
    Interpreter, def_func_pair, get_type,
    types::{
        PyType,
        init::{self, ANY_TYPE_NAME},
        tbool::PyBool,
        tstr::PyStr,
    },
    var::PyValue,
};

const TYPE_NAME: &str = "int";

get_type!(TYPE_NAME);

pub fn init_type(interpreter: Arc<Interpreter>) {
    init::init_type(
        interpreter,
        TYPE_NAME,
        [
            def_func_pair!(__str__, PyInt, interpreter, &[TYPE_NAME]),
            def_func_pair!(__pos__, PyInt, interpreter, &[TYPE_NAME]),
            def_func_pair!(__neg__, PyInt, interpreter, &[TYPE_NAME]),
            def_func_pair!(__invert__, PyInt, interpreter, &[TYPE_NAME]),
            def_func_pair!(__bool__, PyInt, interpreter, &[TYPE_NAME]),
            def_func_pair!(__add__, PyInt, interpreter, &[TYPE_NAME, ANY_TYPE_NAME]),
            def_func_pair!(__sub__, PyInt, interpreter, &[TYPE_NAME, ANY_TYPE_NAME]),
            def_func_pair!(__mul__, PyInt, interpreter, &[TYPE_NAME, ANY_TYPE_NAME]),
            def_func_pair!(
                __floordiv__,
                PyInt,
                interpreter,
                &[TYPE_NAME, ANY_TYPE_NAME]
            ),
            def_func_pair!(__mod__, PyInt, interpreter, &[TYPE_NAME, ANY_TYPE_NAME]),
            def_func_pair!(__lt__, PyInt, interpreter, &[TYPE_NAME, ANY_TYPE_NAME]),
            def_func_pair!(__le__, PyInt, interpreter, &[TYPE_NAME, ANY_TYPE_NAME]),
            def_func_pair!(__gt__, PyInt, interpreter, &[TYPE_NAME, ANY_TYPE_NAME]),
            def_func_pair!(__ge__, PyInt, interpreter, &[TYPE_NAME, ANY_TYPE_NAME]),
            def_func_pair!(__eq__, PyInt, interpreter, &[TYPE_NAME, ANY_TYPE_NAME]),
            def_func_pair!(__ne__, PyInt, interpreter, &[TYPE_NAME, ANY_TYPE_NAME]),
        ],
    )
}

/// Int Value
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

macro_rules! def_binary_op {
    ($func_name:ident, $op:tt, $pyop:literal, $ret:tt) => {
        pub fn $func_name(&self, interpreter: Arc<Interpreter>, values: Vec<Box<dyn PyValue>>) -> $ret {
            if let Some(other_int) = values[0].as_any().downcast_ref::<PyInt>() {
                $ret::new(interpreter, self.value.clone() $op other_int.value.clone())
            } else {
                // TODO: Implement error handling
                panic!("Unsupported operand type(s) for {}: 'int' and '{}'", $pyop, values[0].get_type().get_name());
            }
        }
    }
}

impl PyInt {
    pub fn __str__(&self, interpreter: Arc<Interpreter>, _values: Vec<Box<dyn PyValue>>) -> PyStr {
        PyStr::new(interpreter, self.value.to_string())
    }

    pub fn __pos__(&self, interpreter: Arc<Interpreter>, _values: Vec<Box<dyn PyValue>>) -> PyInt {
        PyInt::new(interpreter, self.value.clone())
    }

    pub fn __neg__(&self, interpreter: Arc<Interpreter>, _values: Vec<Box<dyn PyValue>>) -> PyInt {
        PyInt::new(interpreter, -self.value.clone())
    }

    pub fn __invert__(
        &self,
        interpreter: Arc<Interpreter>,
        _values: Vec<Box<dyn PyValue>>,
    ) -> PyInt {
        PyInt::new(interpreter, !self.value.clone())
    }

    pub fn __bool__(
        &self,
        interpreter: Arc<Interpreter>,
        _values: Vec<Box<dyn PyValue>>,
    ) -> PyBool {
        PyBool::new(interpreter, self.value != BigInt::from(0))
    }

    def_binary_op!(__add__, +, "+", PyInt);
    def_binary_op!(__sub__, -, "-", PyInt);
    def_binary_op!(__mul__, *, "*", PyInt);
    def_binary_op!(__floordiv__, /, "//", PyInt);
    def_binary_op!(__mod__, %, "%", PyInt);
    def_binary_op!(__lt__, <, "<", PyBool);
    def_binary_op!(__le__, <=, "<=", PyBool);
    def_binary_op!(__gt__, >, ">", PyBool);
    def_binary_op!(__ge__, >=, ">=", PyBool);
    def_binary_op!(__eq__, ==, "==", PyBool);
    def_binary_op!(__ne__, !=, "!=", PyBool);
}
