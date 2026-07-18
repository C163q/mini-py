use std::{collections::HashMap, sync::Arc};

use crate::{
    Interpreter,
    var::{PyValue, getset::PyGetSetDef},
};

/// A named variable entry that stores a [`PyValue`] together with its [`PyGetSetDef`] access descriptor.
///
/// Reading and writing the stored value always goes through the descriptor, which allows
/// individual entries to be read-only, write-only, or computed on access.
#[derive(Debug, Clone)]
pub struct Binding {
    value: Arc<dyn PyValue>,
    pub getset: PyGetSetDef,
}

impl Binding {
    /// Creates a new `Binding` with the given value and access descriptor.
    pub fn new(value: Arc<dyn PyValue>, getset: PyGetSetDef) -> Self {
        Self { value, getset }
    }

    /// Reads the stored value through the getter defined in [`PyGetSetDef`].
    ///
    /// Returns an `AttributeError` if this entry has no getter.
    pub fn get(&self, interpreter: Arc<Interpreter>) -> Result<Arc<dyn PyValue>, Arc<dyn PyValue>> {
        self.getset.get(interpreter, &self.value)
    }

    /// Writes a new value through the setter defined in [`PyGetSetDef`].
    ///
    /// Returns an `AttributeError` if this entry has no setter.
    pub fn set(
        &mut self,
        interpreter: Arc<Interpreter>,
        value: Arc<dyn PyValue>,
    ) -> Result<(), Arc<dyn PyValue>> {
        self.getset.set(interpreter, &mut self.value, value)
    }
}

/// Holds a mapping from attribute/variable names to their [`Binding`] entries.
///
/// Every [`PyValue`] implementation owns a `Namespace` (behind a `Mutex`) to store its
/// instance attributes. [`PyType`] also uses one to store the type's methods.
///
/// [`PyType`]: crate::types::PyType
#[derive(Debug, Clone)]
pub struct Namespace {
    pub map: HashMap<String, Binding>,
}

impl Default for Namespace {
    fn default() -> Self {
        Self::new()
    }
}

impl Namespace {
    /// Creates an empty `Namespace`.
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// Returns a shared reference to the underlying name-to-[`Binding`] map.
    pub fn get_mapper(&self) -> &HashMap<String, Binding> {
        &self.map
    }

    /// Returns a mutable reference to the underlying name-to-[`Binding`] map.
    pub fn get_mapper_mut(&mut self) -> &mut HashMap<String, Binding> {
        &mut self.map
    }
}
