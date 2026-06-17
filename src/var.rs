pub mod call;
pub mod getset;
pub mod manager;

use std::{
    any::Any,
    fmt::Debug,
    sync::{Arc, MutexGuard},
};

use crate::{
    Interpreter,
    types::{PyType, error},
    var::{
        getset::PyGetSetDef,
        manager::{Var, VarManager},
    },
};

pub trait PyValue: Any + Send + Sync {
    fn get_type(&self) -> Arc<PyType>;

    fn get_var_manager(&self) -> MutexGuard<'_, VarManager>;

    fn get_var(
        &self,
        interpreter: Arc<Interpreter>,
        name: &str,
    ) -> Result<Arc<dyn PyValue>, Arc<dyn PyValue>> {
        // From var
        if let Some(var) = self.get_var_manager().map.get(name) {
            return var.getset.get(interpreter, &var.value);
        }

        // From type
        if let Some(var) = self.get_type().get_var_manager().map.get(name) {
            return var.getset.get(interpreter, &var.value);
        }

        Err(error::get_attribute_error(
            interpreter,
            format!(
                "'{}' object has no attribute '{}'",
                self.get_type().get_name(),
                name
            ),
        ))
    }

    fn set_var(
        &self,
        interpreter: Arc<Interpreter>,
        name: &str,
        value: Arc<dyn PyValue>,
    ) -> Result<(), Arc<dyn PyValue>> {
        // From var
        if let Some(var) = self.get_var_manager().map.get_mut(name) {
            return var.getset.set(interpreter, &mut var.value, value);
        }

        {
            self.get_var_manager()
                .map
                .insert(name.to_string(), Var::new(value, PyGetSetDef::default()));
        }

        Err(error::get_attribute_error(
            interpreter,
            format!(
                "'{}' object has no attribute '{}'",
                self.get_type().get_name(),
                name
            ),
        ))
    }
}

impl<T: PyValue + Clone> PyValue for Box<T> {
    fn get_type(&self) -> Arc<PyType> {
        (**self).get_type()
    }

    fn get_var_manager(&self) -> MutexGuard<'_, VarManager> {
        (**self).get_var_manager()
    }
}

impl dyn PyValue {
    pub fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Debug for dyn PyValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<{} object>", self.get_type().get_name())
    }
}
