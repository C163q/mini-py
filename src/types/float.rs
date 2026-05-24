use std::{fmt::Display, sync::Arc};

use num_bigint::BigInt;

use num_traits::cast::FromPrimitive;

use crate::{
    Interpreter, def_func_pair, get_type,
    types::{
        PyType,
        init::{self, ANY_TYPE_NAME},
        int::PyInt,
        tbool::PyBool,
        tstr::PyStr,
    },
    var::PyValue,
};

const TYPE_NAME: &str = "float";

get_type!(TYPE_NAME);

pub fn init_type(interpreter: Arc<Interpreter>) {
    init::init_type(
        interpreter,
        TYPE_NAME,
        [
            def_func_pair!(__str__, PyFloat, interpreter, &[TYPE_NAME]),
            def_func_pair!(__int__, PyFloat, interpreter, &[TYPE_NAME]),
            def_func_pair!(__float__, PyFloat, interpreter, &[TYPE_NAME]),
            def_func_pair!(__bool__, PyFloat, interpreter, &[TYPE_NAME]),
            def_func_pair!(__pos__, PyFloat, interpreter, &[TYPE_NAME]),
            def_func_pair!(__neg__, PyFloat, interpreter, &[TYPE_NAME]),
            def_func_pair!(__add__, PyFloat, interpreter, &[TYPE_NAME, ANY_TYPE_NAME]),
            def_func_pair!(__sub__, PyFloat, interpreter, &[TYPE_NAME, ANY_TYPE_NAME]),
            def_func_pair!(__mul__, PyFloat, interpreter, &[TYPE_NAME, ANY_TYPE_NAME]),
            def_func_pair!(
                __floordiv__,
                PyFloat,
                interpreter,
                &[TYPE_NAME, ANY_TYPE_NAME]
            ),
            def_func_pair!(
                __truediv__,
                PyFloat,
                interpreter,
                &[TYPE_NAME, ANY_TYPE_NAME]
            ),
            def_func_pair!(__mod__, PyFloat, interpreter, &[TYPE_NAME, ANY_TYPE_NAME]),
            def_func_pair!(__lt__, PyFloat, interpreter, &[TYPE_NAME, ANY_TYPE_NAME]),
            def_func_pair!(__le__, PyFloat, interpreter, &[TYPE_NAME, ANY_TYPE_NAME]),
            def_func_pair!(__gt__, PyFloat, interpreter, &[TYPE_NAME, ANY_TYPE_NAME]),
            def_func_pair!(__ge__, PyFloat, interpreter, &[TYPE_NAME, ANY_TYPE_NAME]),
            def_func_pair!(__eq__, PyFloat, interpreter, &[TYPE_NAME, ANY_TYPE_NAME]),
            def_func_pair!(__ne__, PyFloat, interpreter, &[TYPE_NAME, ANY_TYPE_NAME]),
        ],
    )
}

/// Int Value
#[derive(Debug, Clone)]
pub struct PyFloat {
    ty: Arc<PyType>,
    value: f64,
}

impl PyFloat {
    pub fn new(interpreter: Arc<Interpreter>, value: f64) -> Self {
        Self {
            ty: get_type(interpreter),
            value,
        }
    }
}

impl PyValue for PyFloat {
    fn get_type(&self) -> Arc<PyType> {
        self.ty.clone()
    }
}

impl Display for PyFloat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

macro_rules! def_binary_op {
    ($func_name:ident, $op:tt, $pyop:literal, $ret:tt) => {
        pub fn $func_name(&self, interpreter: Arc<Interpreter>, values: Vec<Arc<dyn PyValue>>) -> $ret {
            if let Some(other_float) = values[0].as_any().downcast_ref::<PyFloat>() {
                $ret::new(interpreter, self.value $op other_float.value)
            } else {
                // TODO: Implement error handling
                panic!(
                    "Unsupported operand type(s) for {}: 'float' and '{}'",
                    $pyop,
                    values[0].get_type().get_name()
                );
            }
        }
    }
}

impl PyFloat {
    pub fn __str__(&self, interpreter: Arc<Interpreter>, _values: Vec<Arc<dyn PyValue>>) -> PyStr {
        PyStr::new(interpreter, self.value.to_string())
    }

    pub fn __int__(&self, interpreter: Arc<Interpreter>, _values: Vec<Arc<dyn PyValue>>) -> PyInt {
        let int_value = BigInt::from_f64(self.value).unwrap_or_else(|| {
            // TODO: Implement error handling
            panic!("Cannot convert float {} to int", self.value);
        });
        PyInt::new(interpreter, int_value)
    }

    pub fn __float__(
        &self,
        _interpreter: Arc<Interpreter>,
        _values: Vec<Arc<dyn PyValue>>,
    ) -> PyFloat {
        self.clone()
    }

    pub fn __bool__(
        &self,
        interpreter: Arc<Interpreter>,
        _values: Vec<Arc<dyn PyValue>>,
    ) -> PyBool {
        PyBool::new(interpreter, self.value != 0.0)
    }

    pub fn __pos__(
        &self,
        interpreter: Arc<Interpreter>,
        _values: Vec<Arc<dyn PyValue>>,
    ) -> PyFloat {
        PyFloat::new(interpreter, self.value)
    }

    pub fn __neg__(
        &self,
        interpreter: Arc<Interpreter>,
        _values: Vec<Arc<dyn PyValue>>,
    ) -> PyFloat {
        PyFloat::new(interpreter, -self.value)
    }

    pub fn __floordiv__(
        &self,
        interpreter: Arc<Interpreter>,
        values: Vec<Arc<dyn PyValue>>,
    ) -> PyFloat {
        if let Some(other_float) = values[0].as_any().downcast_ref::<PyFloat>() {
            PyFloat::new(interpreter, (self.value / other_float.value).floor())
        } else {
            // TODO: Implement error handling
            panic!(
                "Unsupported operand type(s) for //: 'float' and '{}'",
                values[0].get_type().get_name()
            );
        }
    }

    def_binary_op!(__add__, +, "+", PyFloat);
    def_binary_op!(__sub__, -, "-", PyFloat);
    def_binary_op!(__mul__, *, "*", PyFloat);
    def_binary_op!(__truediv__, /, "/", PyFloat);
    def_binary_op!(__mod__, %, "%", PyFloat);
    def_binary_op!(__lt__, <, "<", PyBool);
    def_binary_op!(__le__, <=, "<=", PyBool);
    def_binary_op!(__gt__, >, ">", PyBool);
    def_binary_op!(__ge__, >=, ">=", PyBool);
    def_binary_op!(__eq__, ==, "==", PyBool);
    def_binary_op!(__ne__, !=, "!=", PyBool);
}
