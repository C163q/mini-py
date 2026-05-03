use std::{fmt::Debug, sync::Arc};

use crate::{Interpreter, error::InterpreterError, var::PyValue};

pub type BuiltinPyFunction = Box<
    dyn Fn(Arc<Interpreter>, Vec<Box<dyn PyValue>>) -> Result<Box<dyn PyValue>, InterpreterError>
        + Send
        + Sync
        + 'static,
>;

pub enum PyFunction {
    Builtin(BuiltinPyFunction),
    // TODO: User-defined functions
}

impl Debug for PyFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PyFunction::Builtin(_) => write!(f, "BuiltinFunction"),
            // PyFunction::UserDefined(_) => write!(f, "UserDefinedFunction"),
        }
    }
}

impl PyFunction {
    pub fn new_builtin<F>(func: F) -> Self
    where
        F: Fn(
                Arc<Interpreter>,
                Vec<Box<dyn PyValue>>,
            ) -> Result<Box<dyn PyValue>, InterpreterError>
            + Send
            + Sync
            + 'static,
    {
        Self::Builtin(Box::new(func))
    }
}
