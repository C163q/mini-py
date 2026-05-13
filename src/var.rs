use std::{any::Any, fmt::Debug, sync::Arc};

use dyn_clone::DynClone;

use crate::{func::PyFunction, types::PyType};

pub trait PyValue: Any + DynClone {
    fn get_type(&self) -> Arc<PyType>;

    fn get_function(&self, name: &str) -> Option<Arc<PyFunction>> {
        self.get_type().get_function(name)
    }
}

dyn_clone::clone_trait_object!(PyValue);

impl<T: PyValue + Clone> PyValue for Box<T> {
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
