use std::{any::Any, fmt::Debug, sync::Arc};

use crate::types::PyType;

pub trait PyValue: Any {
    fn get_type(&self) -> Arc<PyType>;
}

impl<T: PyValue> PyValue for Box<T> {
    fn get_type(&self) -> Arc<PyType> {
        (**self).get_type()
    }
}

impl Debug for Box<dyn PyValue> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PyValue({})", self.get_type().get_name())
    }
}

impl dyn PyValue {
    pub fn as_any(&self) -> &dyn Any {
        self
    }
}
