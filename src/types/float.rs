use std::{
    fmt::Display,
    sync::{Arc, Mutex},
};

use num_bigint::BigInt;

use num_traits::cast::FromPrimitive;

use crate::{
    Interpreter, get_type,
    types::{
        PyType,
        init::{self, PyFunctionMapper},
        int::PyInt,
        tbool::PyBool,
        tstr::PyStr,
    },
    var::{PyValue, manager::VarManager},
};

const TYPE_NAME: &str = "float";

get_type!(TYPE_NAME);

pub fn init_type(interpreter: Arc<Interpreter>) {
    type Current = PyFloat;
    init::init_type(
        interpreter.clone(),
        TYPE_NAME,
        [
            PyFunctionMapper::from_method("__str__", interpreter.clone(), Current::__str__),
            PyFunctionMapper::from_method("__int__", interpreter.clone(), Current::__int__),
            PyFunctionMapper::from_method("__float__", interpreter.clone(), Current::__float__),
            PyFunctionMapper::from_method("__bool__", interpreter.clone(), Current::__bool__),
            PyFunctionMapper::from_method("__pos__", interpreter.clone(), Current::__pos__),
            PyFunctionMapper::from_method("__neg__", interpreter.clone(), Current::__neg__),
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
pub struct PyFloat {
    ty: Arc<PyType>,
    vars: Mutex<VarManager>,
    value: f64,
}

impl PyFloat {
    pub fn new(interpreter: Arc<Interpreter>, value: f64) -> Self {
        Self {
            ty: get_type(interpreter),
            vars: Mutex::new(VarManager::new()),
            value,
        }
    }
}

impl Clone for PyFloat {
    fn clone(&self) -> Self {
        Self {
            ty: self.ty.clone(),
            vars: Mutex::new(self.vars.lock().unwrap().clone()),
            value: self.value,
        }
    }
}

impl PyValue for PyFloat {
    fn get_type(&self) -> Arc<PyType> {
        self.ty.clone()
    }

    fn get_var_manager(&self) -> std::sync::MutexGuard<'_, VarManager> {
        self.vars.lock().unwrap()
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
