use std::{any::Any, fmt::Debug, sync::Arc};

use crate::{func::PyFunction, types::PyType};

pub trait PyValue: Any {
    fn get_type(&self) -> Arc<PyType>;

    fn get_function(&self, name: &str) -> Option<Arc<PyFunction>> {
        self.get_type().get_function(name)
    }
}

impl<T: PyValue> PyValue for Box<T> {
    fn get_type(&self) -> Arc<PyType> {
        (**self).get_type()
    }

    fn get_function(&self, name: &str) -> Option<Arc<PyFunction>> {
        (**self).get_function(name)
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
