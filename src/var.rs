use std::{any::Any, sync::Arc};

use crate::types::{PyType, function::PyFunction};

pub trait PyValue: Any {
    fn get_type(&self) -> Arc<PyType>;

    fn get_function(&self, name: &str) -> Option<Arc<PyFunction>> {
        self.get_type().get_function(name)
    }
}

impl<T: PyValue + Clone> PyValue for Box<T> {
    fn get_type(&self) -> Arc<PyType> {
        (**self).get_type()
    }

    fn get_function(&self, name: &str) -> Option<Arc<PyFunction>> {
        (**self).get_function(name)
    }
}

impl dyn PyValue {
    pub fn as_any(&self) -> &dyn Any {
        self
    }
}
