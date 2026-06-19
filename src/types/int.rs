use std::{
    fmt::Display,
    sync::{Arc, Mutex, MutexGuard},
};

use num_bigint::{BigInt, Sign};

use num_rational::{BigRational, Ratio};
use num_traits::{Pow, Zero, cast::ToPrimitive};

use crate::{
    Interpreter, get_type,
    types::{
        PyType,
        float::PyFloat,
        init::{self, PyFunctionMapper},
        tbool::PyBool,
        tstr::PyStr,
    },
    var::{PyValue, manager::VarManager},
};

const TYPE_NAME: &str = "int";

get_type!(TYPE_NAME);

pub fn init_type(interpreter: Arc<Interpreter>) {
    type Current = PyInt;
    init::init_type(
        interpreter.clone(),
        TYPE_NAME,
        [
            PyFunctionMapper::from_method("__str__", interpreter.clone(), Current::__str__),
            PyFunctionMapper::from_method("__bool__", interpreter.clone(), Current::__bool__),
            PyFunctionMapper::from_method("__int__", interpreter.clone(), Current::__int__),
            PyFunctionMapper::from_method("__float__", interpreter.clone(), Current::__float__),
            PyFunctionMapper::from_method("__pos__", interpreter.clone(), Current::__pos__),
            PyFunctionMapper::from_method("__neg__", interpreter.clone(), Current::__neg__),
            PyFunctionMapper::from_method("__invert__", interpreter.clone(), Current::__invert__),
            PyFunctionMapper::from_method("__add__", interpreter.clone(), Current::__add__),
            PyFunctionMapper::from_method("__sub__", interpreter.clone(), Current::__sub__),
            PyFunctionMapper::from_method("__mul__", interpreter.clone(), Current::__mul__),
            PyFunctionMapper::from_method(
                "__floordiv__",
                interpreter.clone(),
                Current::__floordiv__,
            ),
            PyFunctionMapper::from_method("__truediv__", interpreter.clone(), Current::__truediv__),
            PyFunctionMapper::from_method("__mod__", interpreter.clone(), Current::__mod__),
            PyFunctionMapper::from_method("__pow__", interpreter.clone(), Current::__pow__),
            PyFunctionMapper::from_method("__lt__", interpreter.clone(), Current::__lt__),
            PyFunctionMapper::from_method("__le__", interpreter.clone(), Current::__le__),
            PyFunctionMapper::from_method("__gt__", interpreter.clone(), Current::__gt__),
            PyFunctionMapper::from_method("__ge__", interpreter.clone(), Current::__ge__),
            PyFunctionMapper::from_method("__eq__", interpreter.clone(), Current::__eq__),
            PyFunctionMapper::from_method("__ne__", interpreter.clone(), Current::__ne__),
        ],
    )
}

/// Int Value
#[derive(Debug)]
pub struct PyInt {
    ty: Arc<PyType>,
    vars: Mutex<VarManager>,
    value: BigInt,
}

impl PyInt {
    pub fn new(interpreter: Arc<Interpreter>, value: BigInt) -> Self {
        Self {
            ty: get_type(interpreter),
            vars: Mutex::new(VarManager::new()),
            value,
        }
    }
}

impl Clone for PyInt {
    fn clone(&self) -> Self {
        Self {
            ty: self.ty.clone(),
            vars: Mutex::new(self.vars.lock().unwrap().clone()),
            value: self.value.clone(),
        }
    }
}

impl PyValue for PyInt {
    fn get_type(&self) -> Arc<PyType> {
        self.ty.clone()
    }

    fn get_var_manager(&self) -> MutexGuard<'_, VarManager> {
        self.vars.lock().unwrap()
    }
}

impl Display for PyInt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

macro_rules! def_binary_op {
    ($func_name:ident, $op:tt, $pyop:literal) => {
        pub fn $func_name(&self, interpreter: Arc<Interpreter>, values: Vec<Arc<dyn PyValue>>) -> Arc<dyn PyValue> {
            if let Some(other_int) = values[0].as_any().downcast_ref::<PyInt>() {
                Arc::new(PyInt::new(interpreter, self.value.clone() $op other_int.value.clone()))
            } else if let Some(other_float) = values[0].as_any().downcast_ref::<PyFloat>() {
                Arc::new(PyFloat::new(interpreter, self.value.to_f64().unwrap() $op other_float.value()))
            } else {
                // TODO: Implement error handling
                panic!("Unsupported operand type(s) for {}: 'int' and '{}'", $pyop, values[0].get_type().get_name());
            }
        }
    }
}

macro_rules! def_binary_cmp {
    ($func_name:ident, $op:tt, $pyop:literal) => {
        pub fn $func_name(&self, interpreter: Arc<Interpreter>, values: Vec<Arc<dyn PyValue>>) -> PyBool {
            if let Some(other_int) = values[0].as_any().downcast_ref::<PyInt>() {
                PyBool::new(interpreter, self.value.clone() $op other_int.value.clone())
            } else if let Some(other_float) = values[0].as_any().downcast_ref::<PyFloat>() {
                PyBool::new(interpreter, self.value.to_f64().unwrap() $op other_float.value())
            } else {
                // TODO: Implement error handling
                panic!("Unsupported operand type(s) for {}: 'int' and '{}'", $pyop, values[0].get_type().get_name());
            }
        }
    }
}

impl PyInt {
    pub fn value(&self) -> &BigInt {
        &self.value
    }

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
        } else if let Some(other_float) = values[0].as_any().downcast_ref::<PyFloat>() {
            let other_value = other_float.value();
            if other_value.is_zero() {
                // TODO: Implement error handling
                panic!("division by zero");
            }
            PyFloat::new(
                interpreter,
                self.value.to_f64().unwrap() / other_float.value(),
            )
        } else {
            // TODO: Implement error handling
            panic!(
                "Unsupported operand type(s) for /: 'int' and '{}'",
                values[0].get_type().get_name()
            );
        }
    }

    pub fn __floordiv__(
        &self,
        interpreter: Arc<Interpreter>,
        values: Vec<Arc<dyn PyValue>>,
    ) -> Arc<dyn PyValue> {
        if let Some(other_int) = values[0].as_any().downcast_ref::<PyInt>() {
            let other_value = other_int.value.clone();
            if other_value.is_zero() {
                // TODO: Implement error handling
                panic!("division by zero");
            }
            Arc::new(PyInt::new(interpreter, self.value.clone() / other_value))
        } else if let Some(other_float) = values[0].as_any().downcast_ref::<PyFloat>() {
            let other_value = other_float.value();
            if other_value.is_zero() {
                // TODO: Implement error handling
                panic!("division by zero");
            }
            Arc::new(PyFloat::new(
                interpreter,
                (self.value.to_f64().unwrap() / other_float.value()).floor(),
            ))
        } else {
            panic!(
                "Unsupported operand type(s) for //: 'int' and '{}'",
                values[0].get_type().get_name()
            );
        }
    }

    pub fn __pow__(
        &self,
        interpreter: Arc<Interpreter>,
        values: Vec<Arc<dyn PyValue>>,
    ) -> Arc<dyn PyValue> {
        if let Some(other_int) = values[0].as_any().downcast_ref::<PyInt>() {
            let (sign, value) = other_int.value.clone().into_parts();
            match sign {
                Sign::Plus | Sign::NoSign => Arc::new(PyInt::new(
                    interpreter.clone(),
                    Pow::pow(self.value.clone(), value),
                )),
                Sign::Minus => {
                    if self.value.is_zero() {
                        panic!("division by zero");
                    }
                    let base = Ratio::new(BigInt::from(1), self.value.clone());
                    Arc::new(PyFloat::new(
                        interpreter,
                        Pow::pow(base, value).to_f64().unwrap(),
                    ))
                }
            }
        } else {
            // TODO: Implement error handling
            panic!(
                "Unsupported operand type(s) for {}: 'int' and '{}'",
                "**",
                values[0].get_type().get_name()
            );
        }
    }

    def_binary_op!(__add__, +, "+");
    def_binary_op!(__sub__, -, "-");
    def_binary_op!(__mul__, *, "*");
    def_binary_op!(__mod__, %, "%");
    def_binary_cmp!(__lt__, <, "<");
    def_binary_cmp!(__le__, <=, "<=");
    def_binary_cmp!(__gt__, >, ">");
    def_binary_cmp!(__ge__, >=, ">=");
    def_binary_cmp!(__eq__, ==, "==");
    def_binary_cmp!(__ne__, !=, "!=");
}
