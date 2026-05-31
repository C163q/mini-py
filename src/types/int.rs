use std::{fmt::Display, sync::Arc};

use num_bigint::BigInt;

use num_rational::BigRational;
use num_traits::{Zero, cast::ToPrimitive};

use crate::{
    Interpreter, get_type,
    types::{
        PyType,
        float::PyFloat,
        init::{self, PyFunctionMapper},
        tbool::PyBool,
        tstr::PyStr,
    },
    var::PyValue,
};

const TYPE_NAME: &str = "int";

get_type!(TYPE_NAME);

pub fn init_type(interpreter: Arc<Interpreter>) {
    type Current = PyInt;
    init::init_type(
        interpreter.clone(),
        TYPE_NAME,
        [
            PyFunctionMapper::from_method("__str__", Current::__str__),
            PyFunctionMapper::from_method("__bool__", Current::__bool__),
            PyFunctionMapper::from_method("__int__", Current::__int__),
            PyFunctionMapper::from_method("__float__", Current::__float__),
            PyFunctionMapper::from_method("__pos__", Current::__pos__),
            PyFunctionMapper::from_method("__neg__", Current::__neg__),
            PyFunctionMapper::from_method("__invert__", Current::__invert__),
            PyFunctionMapper::from_method("__add__", Current::__add__),
            PyFunctionMapper::from_method("__sub__", Current::__sub__),
            PyFunctionMapper::from_method("__mul__", Current::__mul__),
            PyFunctionMapper::from_method("__floordiv__", Current::__floordiv__),
            PyFunctionMapper::from_method("__truediv__", Current::__truediv__),
            PyFunctionMapper::from_method("__mod__", Current::__mod__),
            PyFunctionMapper::from_method("__lt__", Current::__lt__),
            PyFunctionMapper::from_method("__le__", Current::__le__),
            PyFunctionMapper::from_method("__gt__", Current::__gt__),
            PyFunctionMapper::from_method("__ge__", Current::__ge__),
            PyFunctionMapper::from_method("__eq__", Current::__eq__),
            PyFunctionMapper::from_method("__ne__", Current::__ne__),
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
        pub fn $func_name(&self, interpreter: Arc<Interpreter>, values: Vec<Arc<dyn PyValue>>) -> $ret {
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
    pub fn __str__(&self, interpreter: Arc<Interpreter>, _values: Vec<Arc<dyn PyValue>>) -> PyStr {
        PyStr::new(interpreter, self.value.to_string())
    }

    pub fn __bool__(
        &self,
        interpreter: Arc<Interpreter>,
        _values: Vec<Arc<dyn PyValue>>,
    ) -> PyBool {
        PyBool::new(interpreter, self.value != BigInt::from(0))
    }

    pub fn __int__(&self, _interpreter: Arc<Interpreter>, _values: Vec<Arc<dyn PyValue>>) -> PyInt {
        self.clone()
    }

    pub fn __float__(
        &self,
        interpreter: Arc<Interpreter>,
        _values: Vec<Arc<dyn PyValue>>,
    ) -> PyFloat {
        // It never fails.
        let float_value = self.value.to_f64().unwrap();
        PyFloat::new(interpreter, float_value)
    }

    pub fn __pos__(&self, interpreter: Arc<Interpreter>, _values: Vec<Arc<dyn PyValue>>) -> PyInt {
        PyInt::new(interpreter, self.value.clone())
    }

    pub fn __neg__(&self, interpreter: Arc<Interpreter>, _values: Vec<Arc<dyn PyValue>>) -> PyInt {
        PyInt::new(interpreter, -self.value.clone())
    }

    pub fn __invert__(
        &self,
        interpreter: Arc<Interpreter>,
        _values: Vec<Arc<dyn PyValue>>,
    ) -> PyInt {
        PyInt::new(interpreter, !self.value.clone())
    }

    pub fn __truediv__(
        &self,
        interpreter: Arc<Interpreter>,
        values: Vec<Arc<dyn PyValue>>,
    ) -> PyFloat {
        if let Some(other_int) = values[0].as_any().downcast_ref::<PyInt>() {
            let other_value = other_int.value.clone();
            if other_value.is_zero() {
                // TODO: Implement error handling
                panic!("division by zero");
            }
            BigRational::new(self.value.clone(), other_int.value.clone())
                .to_f64()
                .map(|f| PyFloat::new(interpreter, f))
                // Never fails because None is only returned when f64 is NaN.
                // This case has been handled above.
                .unwrap()
        } else {
            // TODO: Implement error handling
            panic!(
                "Unsupported operand type(s) for /: 'int' and '{}'",
                values[0].get_type().get_name()
            );
        }
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
