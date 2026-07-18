use std::{
    fmt::Debug,
    sync::{Arc, LazyLock},
};

use crate::{Interpreter, types::error, var::PyValue};

/// A thread-safe getter function that reads a [`PyValue`] from a [`Binding`] entry.
///
/// [`Binding`]: crate::var::namespace::Binding
pub type Getter = Arc<
    dyn Fn(Arc<Interpreter>, &Arc<dyn PyValue>) -> Result<Arc<dyn PyValue>, Arc<dyn PyValue>>
        + 'static
        + Send
        + Sync,
>;

/// A thread-safe setter function that writes a [`PyValue`] into a [`Binding`] entry.
///
/// [`Binding`]: crate::var::namespace::Binding
pub type Setter = Arc<
    dyn Fn(
            Arc<Interpreter>,
            &mut Arc<dyn PyValue>,
            Arc<dyn PyValue>,
        ) -> Result<(), Arc<dyn PyValue>>
        + 'static
        + Send
        + Sync,
>;

fn default_getter(
    _interpreter: Arc<Interpreter>,
    target: &Arc<dyn PyValue>,
) -> Result<Arc<dyn PyValue>, Arc<dyn PyValue>> {
    Ok(target.clone())
}

fn default_setter(
    _interpreter: Arc<Interpreter>,
    target: &mut Arc<dyn PyValue>,
    value: Arc<dyn PyValue>,
) -> Result<(), Arc<dyn PyValue>> {
    *target = value;
    Ok(())
}

static DEFAULT_GETTER: LazyLock<Getter> = LazyLock::new(|| Arc::new(default_getter));
static DEFAULT_SETTER: LazyLock<Setter> = LazyLock::new(|| Arc::new(default_setter));

/// Descriptor that controls how a [`Binding`] entry is read and written.
///
/// Both getter and setter are optional. An entry with no getter returns an `AttributeError` on
/// read; an entry with no setter returns an `AttributeError` on write. The [`Default`]
/// implementation provides a plain pass-through getter and a direct-assignment setter.
///
/// [`Binding`]: crate::var::namespace::Binding
#[derive(Clone)]
pub struct PyGetSetDef {
    getter: Option<Getter>,
    setter: Option<Setter>,
}

impl Default for PyGetSetDef {
    fn default() -> Self {
        Self {
            getter: Some(DEFAULT_GETTER.clone()),
            setter: Some(DEFAULT_SETTER.clone()),
        }
    }
}

impl Debug for PyGetSetDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "<PyGetSetDef getter={:?} setter={:?}>",
            self.getter.is_some(),
            self.setter.is_some()
        )
    }
}

impl PyGetSetDef {
    /// Creates a `PyGetSetDef` with the given getter and setter. Pass `None` to make the entry
    /// write-only or read-only respectively.
    pub fn new(getter: Option<Getter>, setter: Option<Setter>) -> Self {
        Self { getter, setter }
    }

    /// Invokes the getter to read `target`. Returns an `AttributeError` if this entry has no getter.
    pub fn get(
        &self,
        interpreter: Arc<Interpreter>,
        target: &Arc<dyn PyValue>,
    ) -> Result<Arc<dyn PyValue>, Arc<dyn PyValue>> {
        match &self.getter {
            Some(getter) => getter(interpreter, target),
            None => Err(error::get_attribute_error(
                interpreter,
                "Attribute is not readable".to_string(),
            )),
        }
    }

    /// Invokes the setter to write `value` into `target`. Returns an `AttributeError` if this
    /// entry has no setter.
    pub fn set(
        &self,
        interpreter: Arc<Interpreter>,
        target: &mut Arc<dyn PyValue>,
        value: Arc<dyn PyValue>,
    ) -> Result<(), Arc<dyn PyValue>> {
        match &self.setter {
            Some(setter) => setter(interpreter, target, value),
            None => Err(error::get_attribute_error(
                interpreter,
                "Attribute is not writable".to_string(),
            )),
        }
    }
}
