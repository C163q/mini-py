use std::sync::Arc;

use crate::{Interpreter, var::PyValue};

pub mod base_excption;

pub fn get_type_error(interpreter: Arc<Interpreter>, message: String) -> Arc<dyn PyValue> {
    // TypeError is not implemented yet, so we return BaseException instead
    Arc::new(base_excption::PyBaseException::new(interpreter, message))
}

pub fn get_syntax_error(interpreter: Arc<Interpreter>, message: String) -> Arc<dyn PyValue> {
    // SyntaxError is not implemented yet, so we return BaseException instead
    Arc::new(base_excption::PyBaseException::new(interpreter, message))
}

pub fn get_attribute_error(interpreter: Arc<Interpreter>, message: String) -> Arc<dyn PyValue> {
    // AttributeError is not implemented yet, so we return BaseException instead
    Arc::new(base_excption::PyBaseException::new(interpreter, message))
}

pub fn get_name_error(interpreter: Arc<Interpreter>, message: String) -> Arc<dyn PyValue> {
    // NameError is not implemented yet, so we return BaseException instead
    Arc::new(base_excption::PyBaseException::new(interpreter, message))
}
