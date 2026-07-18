pub mod call;
pub mod getset;
pub mod namespace;

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
        namespace::{Binding, Namespace},
    },
};

/// Implemented by every value that can appear in a mini-Python program.
///
/// In Python every entity — including types themselves — is an object, so this trait is the
/// universal value interface. For example, `int` is represented as `PyInt`, which holds an
/// [`Arc<PyType>`] for type information, a [`Mutex<Namespace>`] for its attributes/methods,
/// and the underlying Rust value.
///
/// [`Arc<PyType>`]: crate::types::PyType
/// [`Mutex<Namespace>`]: crate::var::namespace::Namespace
pub trait PyValue: Any + Send + Sync {
    /// Returns the Python type of this value.
    ///
    /// The returned [`Arc<PyType>`] must be consistent for all instances of the same type and must
    /// match the name registered in the interpreter.
    fn get_type(&self) -> Arc<PyType>;

    /// Returns the namespace holding this value's attributes or methods.
    ///
    /// For a type object, this contains the type's methods. For an instance, this contains the
    /// instance's attributes.
    fn get_namespace(&self) -> MutexGuard<'_, Namespace>;

    /// Looks up an attribute by name.
    ///
    /// The default implementation searches the instance's own [`Namespace`] first, then fall back
    /// to the type's [`Namespace`]. Returns an `AttributeError` if the name is not found in either.
    fn get_binding(
        &self,
        interpreter: Arc<Interpreter>,
        name: &str,
    ) -> Result<Arc<dyn PyValue>, Arc<dyn PyValue>> {
        // From var
        if let Some(var) = self.get_namespace().get_mapper().get(name) {
            return var.get(interpreter);
        }

        // From type
        if let Some(var) = self.get_type().get_namespace().get_mapper().get(name) {
            return var.get(interpreter);
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

    /// Sets an attribute by name.
    ///
    /// The default implementation updates the existing entry in the instance's [`Namespace`] if
    /// the attribute already exists. If the name is new, a fresh entry is inserted and `Ok(())`
    /// is returned, allowing dynamic attribute assignment.
    fn set_binding(
        &self,
        interpreter: Arc<Interpreter>,
        name: &str,
        value: Arc<dyn PyValue>,
    ) -> Result<(), Arc<dyn PyValue>> {
        // From var
        if let Some(var) = self.get_namespace().get_mapper_mut().get_mut(name) {
            return var.set(interpreter, value);
        }

        {
            self.get_namespace().get_mapper_mut().insert(
                name.to_string(),
                Binding::new(value, PyGetSetDef::default()),
            );
        }

        Ok(())
    }
}

impl<T: PyValue + Clone> PyValue for Box<T> {
    fn get_type(&self) -> Arc<PyType> {
        (**self).get_type()
    }

    fn get_namespace(&self) -> MutexGuard<'_, Namespace> {
        (**self).get_namespace()
    }
}

impl dyn PyValue {
    /// Returns a reference to this value as `dyn Any`, enabling downcasting to a concrete type.
    pub fn as_any(&self) -> &(dyn Any + Send + Sync) {
        self
    }

    /// Converts `Arc<dyn PyValue>` into `Arc<dyn Any>`, enabling downcasting to a concrete type.
    pub fn as_arc_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync> {
        self
    }
}

impl Debug for dyn PyValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<{} object>", self.get_type().get_name())
    }
}

/// Converts a value into `Result<Arc<dyn PyValue>, Arc<dyn PyValue>>`, the standard return type
/// for built-in functions and attribute accessors.
pub trait IntoPyValueArcResult {
    fn into_pyvalue_arc(self) -> Result<Arc<dyn PyValue>, Arc<dyn PyValue>>;
}

impl<T> IntoPyValueArcResult for T
where
    T: PyValue + 'static,
{
    fn into_pyvalue_arc(self) -> Result<Arc<dyn PyValue>, Arc<dyn PyValue>> {
        Ok(Arc::new(self))
    }
}

impl IntoPyValueArcResult for Arc<dyn PyValue> {
    fn into_pyvalue_arc(self) -> Result<Arc<dyn PyValue>, Arc<dyn PyValue>> {
        Ok(self)
    }
}

impl IntoPyValueArcResult for Result<Arc<dyn PyValue>, Arc<dyn PyValue>> {
    fn into_pyvalue_arc(self) -> Result<Arc<dyn PyValue>, Arc<dyn PyValue>> {
        self
    }
}
