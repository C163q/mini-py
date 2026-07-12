use std::sync::Arc;

use crate::{Interpreter, var::PyValue};

pub mod base_excption;

/// Returns a `TypeError`. Falls back to [`PyBaseException`] until `TypeError` is implemented.
///
/// [`PyBaseException`]: base_excption::PyBaseException
pub fn get_type_error(interpreter: Arc<Interpreter>, message: String) -> Arc<dyn PyValue> {
    Arc::new(base_excption::PyBaseException::new(interpreter, message))
}

/// Returns a `SyntaxError`. Falls back to [`PyBaseException`] until `SyntaxError` is implemented.
///
/// [`PyBaseException`]: base_excption::PyBaseException
pub fn get_syntax_error(interpreter: Arc<Interpreter>, message: String) -> Arc<dyn PyValue> {
    Arc::new(base_excption::PyBaseException::new(interpreter, message))
}

/// Returns an `AttributeError`. Falls back to [`PyBaseException`] until `AttributeError` is implemented.
///
/// [`PyBaseException`]: base_excption::PyBaseException
pub fn get_attribute_error(interpreter: Arc<Interpreter>, message: String) -> Arc<dyn PyValue> {
    Arc::new(base_excption::PyBaseException::new(interpreter, message))
}

/// Returns a `NameError`. Falls back to [`PyBaseException`] until `NameError` is implemented.
///
/// [`PyBaseException`]: base_excption::PyBaseException
pub fn get_name_error(interpreter: Arc<Interpreter>, message: String) -> Arc<dyn PyValue> {
    Arc::new(base_excption::PyBaseException::new(interpreter, message))
}

/// Returns a `RuntimeError`. Falls back to [`PyBaseException`] until `RuntimeError` is implemented.
///
/// [`PyBaseException`]: base_excption::PyBaseException
pub fn get_runtime_error(interpreter: Arc<Interpreter>, message: String) -> Arc<dyn PyValue> {
    Arc::new(base_excption::PyBaseException::new(interpreter, message))
}
